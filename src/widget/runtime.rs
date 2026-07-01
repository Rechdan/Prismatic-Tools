//! `widget-runtime`: run a widget's Lua in a restricted VM and map its
//! declarative `render(state)` output onto reactor elements.
//!
//! The widget's chunk runs once and must return a module table `{ state, render }`
//! (see the `widget-runtime` spec). We hold owned handles to the `state` table
//! (source of truth, held so a future feature can serialize it) and the `render`
//! function, then call `render(state)` on every reactor render.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use mlua::{Function, Lua, LuaOptions, StdLib, Table, Value};
use windows_reactor::*;

use super::WidgetError;
use super::package::WidgetSource;

/// A loaded widget: its sandboxed VM plus owned handles to the module's `state`
/// table and `render` function. Held across renders in a reactor `use_ref`.
pub struct LoadedWidget {
    id: String,
    // The manifest-declared display name (`widget.toml` `name`). Surfaced in the
    // shell's navigation column; distinct from `id` (the folder name).
    name: String,
    // Kept alive for the widget's lifetime; the handles below reference this VM.
    _lua: Lua,
    state: Table,
    render: Function,
    // Set by a button callback when its Lua errors; drained on the next render so
    // the error is surfaced (a callback runs outside the render's error path).
    last_error: Rc<RefCell<Option<String>>>,
}

/// Load a widget source into a sandboxed VM and resolve its `{ state, render }`.
// `Result` is written fully-qualified: `use windows_reactor::*` glob-imports
// reactor's one-arg `Result` alias, which would shadow the std two-arg form.
pub fn load(source: &WidgetSource) -> std::result::Result<LoadedWidget, WidgetError> {
    let lua = make_sandbox(&source.dir).map_err(|e| lua_err(&source.id, e))?;
    install_ui_api(&lua).map_err(|e| lua_err(&source.id, e))?;

    let module: Value = lua
        .load(&source.source)
        .set_name(&source.id)
        .eval()
        .map_err(|e| lua_err(&source.id, e))?;
    let module = match module {
        Value::Table(t) => t,
        _ => {
            return Err(WidgetError::Lua {
                id: source.id.clone(),
                problem: "widget must return a table { state, render }".into(),
            });
        }
    };

    let render = match module.get::<Option<Function>>("render") {
        Ok(Some(f)) => f,
        _ => {
            return Err(WidgetError::Lua {
                id: source.id.clone(),
                problem: "widget module has no 'render' function".into(),
            });
        }
    };
    let state = match module.get::<Option<Table>>("state") {
        Ok(Some(t)) => t,
        Ok(None) => lua.create_table().map_err(|e| lua_err(&source.id, e))?,
        Err(_) => {
            return Err(WidgetError::Lua {
                id: source.id.clone(),
                problem: "'state' must be a table".into(),
            });
        }
    };

    Ok(LoadedWidget {
        id: source.id.clone(),
        name: source.manifest.name.clone(),
        _lua: lua,
        state,
        render,
        last_error: Rc::new(RefCell::new(None)),
    })
}

impl LoadedWidget {
    /// The widget's manifest-declared display name, for the navigation label.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Call `render(state)` and map the result to a reactor element. `set_tick`
    /// bumps a reactor state cell after any callback so the UI re-renders; `tick`
    /// is the current value (a callback stores `tick + 1`).
    pub fn render(&self, set_tick: &SetState<u32>, tick: u32) -> Element {
        // A callback that raised an error since the last render: surface it.
        if let Some(err) = self.last_error.borrow_mut().take() {
            return error_text(&format!("Widget '{}': {err}", self.id));
        }
        match self.render.call::<Value>(self.state.clone()) {
            Ok(node) => node_to_element(&node, set_tick, tick, &self.last_error),
            Err(e) => error_text(&format!("Widget '{}': {e}", self.id)),
        }
    }
}

/// Build a widget VM opening only safe libraries, then harden it.
fn make_sandbox(dir: &Path) -> mlua::Result<Lua> {
    // base is always loaded; open only the safe extras (no os/io/debug).
    let lua = Lua::new_with(
        StdLib::TABLE
            | StdLib::STRING
            | StdLib::MATH
            | StdLib::COROUTINE
            | StdLib::UTF8
            | StdLib::PACKAGE,
        LuaOptions::default(),
    )?;
    harden(&lua, dir)?;
    Ok(lua)
}

/// Strip the filesystem/native escape hatches and scope `require` to the widget.
fn harden(lua: &Lua, dir: &Path) -> mlua::Result<()> {
    let g = lua.globals();
    // Base file-reading functions bypass the scoped `require`; remove them.
    g.set("dofile", Value::Nil)?;
    g.set("loadfile", Value::Nil)?;
    // These libs are not opened, but nil the globals defensively.
    g.set("os", Value::Nil)?;
    g.set("io", Value::Nil)?;
    g.set("debug", Value::Nil)?;

    let package: Table = g.get("package")?;
    // `require` resolves only inside the widget's own folder; no native modules.
    package.set("path", format!("{}/?.lua", dir.display()))?;
    package.set("cpath", "")?;
    package.set("loadlib", Value::Nil)?;
    // Truncate searchers to [preload, Lua]; dropping the C / all-in-one loaders.
    // `require` iterates searchers with ipairs, which stops at the first nil.
    let searchers: Table = package.get("searchers")?;
    searchers.set(3, Value::Nil)?;
    searchers.set(4, Value::Nil)?;
    Ok(())
}

/// Inject the declarative UI builder globals. Stacks tag the caller's table
/// in place (hash keys stay as props, array entries as children); `text`/`button`
/// build fresh tagged tables.
fn install_ui_api(lua: &Lua) -> mlua::Result<()> {
    let g = lua.globals();
    g.set(
        "vstack",
        lua.create_function(|_, props: Table| {
            props.set("kind", "vstack")?;
            Ok(props)
        })?,
    )?;
    g.set(
        "hstack",
        lua.create_function(|_, props: Table| {
            props.set("kind", "hstack")?;
            Ok(props)
        })?,
    )?;
    g.set(
        "text",
        lua.create_function(|lua, value: String| {
            let t = lua.create_table()?;
            t.set("kind", "text")?;
            t.set("value", value)?;
            Ok(t)
        })?,
    )?;
    g.set(
        "button",
        lua.create_function(|lua, (label, on_click): (String, Option<Function>)| {
            let t = lua.create_table()?;
            t.set("kind", "button")?;
            t.set("label", label)?;
            if let Some(f) = on_click {
                t.set("on_click", f)?;
            }
            Ok(t)
        })?,
    )?;
    Ok(())
}

/// Map one tagged node table to a reactor element. Infallible: any malformed or
/// unrecognized node becomes an inline error element so siblings still render.
fn node_to_element(
    node: &Value,
    set_tick: &SetState<u32>,
    tick: u32,
    last_error: &Rc<RefCell<Option<String>>>,
) -> Element {
    let Some(t) = node.as_table() else {
        return error_text("invalid node (not a table)");
    };
    let kind: String = t.get("kind").unwrap_or_default();
    match kind.as_str() {
        "vstack" | "hstack" => {
            let children: Vec<Element> = t
                .clone()
                .sequence_values::<Value>()
                .filter_map(|child| child.ok())
                .map(|child| node_to_element(&child, set_tick, tick, last_error))
                .collect();
            let spacing = t.get::<Option<f64>>("spacing").ok().flatten();
            let stack = if kind == "vstack" {
                vstack(children)
            } else {
                hstack(children)
            };
            match spacing {
                Some(sp) => stack.spacing(sp).into(),
                None => stack.into(),
            }
        }
        "text" => text_block(t.get::<String>("value").unwrap_or_default()).into(),
        "button" => {
            let label = t.get::<String>("label").unwrap_or_default();
            let b = button(label);
            match t.get::<Option<Function>>("on_click").ok().flatten() {
                Some(f) => {
                    let setter = set_tick.clone();
                    let next = tick.wrapping_add(1);
                    let err = Rc::clone(last_error);
                    b.on_click(move || {
                        if let Err(e) = f.call::<()>(()) {
                            *err.borrow_mut() = Some(e.to_string());
                        }
                        setter.call(next);
                    })
                    .into()
                }
                None => b.into(),
            }
        }
        "" => error_text("node missing 'kind'"),
        other => error_text(&format!("unsupported node '{other}'")),
    }
}

/// An inline error element shown in place of a bad node.
fn error_text(msg: &str) -> Element {
    text_block(format!("⚠ {msg}")).into()
}

fn lua_err(id: &str, e: mlua::Error) -> WidgetError {
    WidgetError::Lua {
        id: id.to_string(),
        problem: e.to_string(),
    }
}
