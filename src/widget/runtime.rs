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
    // Optional display-only preview render for the navigation card. Called with
    // the same `state` handle as `render`, so its preview stays live as the main
    // view mutates state. Absent for widgets that ship no `nav` function.
    nav: Option<Function>,
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

    // Optional: absent, nil, or a non-function value all degrade to `None` (nav is
    // an opt-in preview, not required).
    let nav = module.get::<Option<Function>>("nav").ok().flatten();

    Ok(LoadedWidget {
        id: source.id.clone(),
        name: source.manifest.name.clone(),
        _lua: lua,
        state,
        render,
        nav,
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

    /// Call `nav(state)` and resolve the navigation card. Returns `None` when the
    /// widget ships no `nav`. Shares the one `state` handle with `render`, so the
    /// preview reflects main-view mutations. Display-only: no callbacks are wired
    /// (a `button` in the tree becomes an inline error), so it needs no `set_tick`
    /// or error cell — render-time errors surface synchronously as the content.
    pub fn nav_render(&self) -> Option<NavCard> {
        let nav = self.nav.as_ref()?;
        match nav.call::<Value>(self.state.clone()) {
            Ok(node) => Some(nav_card_from_node(&node)),
            Err(e) => Some(NavCard {
                style: BorderStyle::default_card(),
                content: error_text(&format!("Widget '{}': {e}", self.id)),
            }),
        }
    }
}

/// The navigation card for a loaded widget: the resolved outer-border style plus
/// the preview content the shell places inside it. When a widget's `nav` root is a
/// `border`, its visual props are merged over the default card; otherwise the
/// default card wraps the whole preview tree.
pub struct NavCard {
    pub style: BorderStyle,
    pub content: Element,
}

fn nav_card_from_node(node: &Value) -> NavCard {
    if let Some(t) = node.as_table() {
        let kind: String = t.get("kind").unwrap_or_default();
        if kind == "border" {
            return NavCard {
                style: BorderStyle::from_table(t).over(BorderStyle::default_card()),
                content: single_child(map_children(t, Mapper::Nav)),
            };
        }
    }
    NavCard {
        style: BorderStyle::default_card(),
        content: map_node(node, Mapper::Nav),
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
    // Like the stacks, `border` tags the caller's table in place: hash keys stay as
    // visual props (corner_radius/border_thickness/padding/background/border_color),
    // array entries are the wrapped child(ren).
    g.set(
        "border",
        lua.create_function(|_, props: Table| {
            props.set("kind", "border")?;
            Ok(props)
        })?,
    )?;
    Ok(())
}

/// Rendering mode for [`map_node`]. The main `render` path wires interactive
/// callbacks; the nav preview is display-only, so a `button` there is an inline
/// error rather than an interactive control. Copyable so it threads cheaply
/// through the recursion.
#[derive(Clone, Copy)]
enum Mapper<'a> {
    Interactive {
        set_tick: &'a SetState<u32>,
        tick: u32,
        last_error: &'a Rc<RefCell<Option<String>>>,
    },
    Nav,
}

/// Map one tagged node table to a reactor element. Infallible: any malformed or
/// unrecognized node becomes an inline error element so siblings still render.
fn map_node(node: &Value, m: Mapper) -> Element {
    let Some(t) = node.as_table() else {
        return error_text("invalid node (not a table)");
    };
    let kind: String = t.get("kind").unwrap_or_default();
    match kind.as_str() {
        "vstack" | "hstack" => {
            let children = map_children(t, m);
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
        "border" => BorderStyle::from_table(t)
            .apply(border(single_child(map_children(t, m))))
            .into(),
        "button" => match m {
            Mapper::Interactive {
                set_tick,
                tick,
                last_error,
            } => {
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
            Mapper::Nav => error_text("buttons not supported in nav"),
        },
        "" => error_text("node missing 'kind'"),
        other => error_text(&format!("unsupported node '{other}'")),
    }
}

/// Interactive mapping wrapper used by the main `render` path.
fn node_to_element(
    node: &Value,
    set_tick: &SetState<u32>,
    tick: u32,
    last_error: &Rc<RefCell<Option<String>>>,
) -> Element {
    map_node(
        node,
        Mapper::Interactive {
            set_tick,
            tick,
            last_error,
        },
    )
}

/// Map a node's array entries (children) with the given mapper.
fn map_children(t: &Table, m: Mapper) -> Vec<Element> {
    t.clone()
        .sequence_values::<Value>()
        .filter_map(|child| child.ok())
        .map(|child| map_node(&child, m))
        .collect()
}

/// Reduce a child list to the single child a `border` wraps: one entry as-is,
/// several stacked in a `vstack`, none → an empty text node.
fn single_child(children: Vec<Element>) -> Element {
    match children.len() {
        0 => text_block("").into(),
        1 => children.into_iter().next().unwrap(),
        _ => vstack(children).into(),
    }
}

/// A brush for a `border` node / nav card: a fixed RGBA color parsed from Lua, or
/// a theme-aware brush (used by the default card so it tracks light/dark Mica).
/// `ThemeRef` is `Clone` but not `Copy`, so neither is this.
#[derive(Clone)]
enum Brush {
    Rgba(Color),
    Theme(ThemeRef),
}

/// Visual props for a `border` node and the navigation card. Every prop is
/// optional; `None` leaves the reactor/theme default (or, when merged onto the
/// default card via [`BorderStyle::over`], the default-card value).
#[derive(Clone)]
pub struct BorderStyle {
    corner_radius: Option<f64>,
    border_thickness: Option<f64>,
    padding: Option<f64>,
    background: Option<Brush>,
    border_color: Option<Brush>,
}

impl BorderStyle {
    /// The theme-aware default navigation card: a card fill with a hairline stroke,
    /// rounded and padded. Tracks light/dark Mica via `ThemeRef` brushes.
    pub fn default_card() -> Self {
        Self {
            corner_radius: Some(6.0),
            border_thickness: Some(1.0),
            padding: Some(10.0),
            background: Some(Brush::Theme(ThemeRef::CardBackground)),
            border_color: Some(Brush::Theme(ThemeRef::CardStroke)),
        }
    }

    /// Parse a `border` node's visual props from its Lua table (unset → `None`).
    fn from_table(t: &Table) -> Self {
        Self {
            corner_radius: t.get::<Option<f64>>("corner_radius").ok().flatten(),
            border_thickness: t.get::<Option<f64>>("border_thickness").ok().flatten(),
            padding: t.get::<Option<f64>>("padding").ok().flatten(),
            background: parse_color(t, "background").map(Brush::Rgba),
            border_color: parse_color(t, "border_color").map(Brush::Rgba),
        }
    }

    /// Overlay `self`'s set props onto `base`; `self` wins where present. Used to
    /// merge a widget's nav `border` props over the default card.
    fn over(self, base: Self) -> Self {
        Self {
            corner_radius: self.corner_radius.or(base.corner_radius),
            border_thickness: self.border_thickness.or(base.border_thickness),
            padding: self.padding.or(base.padding),
            background: self.background.or(base.background),
            border_color: self.border_color.or(base.border_color),
        }
    }

    /// Corner radius, if set — the nav card matches its hover overlay to this.
    pub fn corner_radius(&self) -> Option<f64> {
        self.corner_radius
    }

    /// Padding, if set — the nav card relocates this onto its content (not the
    /// frame) so the card's background layers span the whole card.
    pub fn padding(&self) -> Option<f64> {
        self.padding
    }

    /// Apply the frame props (corner radius, stroke, background) to a `Border`,
    /// but NOT padding. The nav card uses this and pads its content instead, so
    /// the base fill and hover fill cover the full card.
    pub fn apply_frame(&self, mut b: Border) -> Border {
        if let Some(cr) = self.corner_radius {
            b = b.corner_radius(cr);
        }
        if let Some(th) = self.border_thickness {
            b = b.border_thickness(Thickness::uniform(th));
        }
        if let Some(bg) = &self.background {
            b = match bg {
                Brush::Rgba(c) => b.background(*c),
                Brush::Theme(t) => b.background(t.clone()),
            };
        }
        if let Some(bc) = &self.border_color {
            b = match bc {
                Brush::Rgba(c) => b.border_brush(*c),
                Brush::Theme(t) => b.border_brush(t.clone()),
            };
        }
        b
    }

    /// Apply all visual props including padding — used by the general `border`
    /// node in a widget's main render, where padding insets the child as usual.
    pub fn apply(&self, b: Border) -> Border {
        let mut b = self.apply_frame(b);
        if let Some(p) = self.padding {
            b = b.padding(Thickness::uniform(p));
        }
        b
    }
}

/// Parse a color prop as `{r, g, b}` or `{r, g, b, a}`: up to four numeric array
/// entries, each clamped to 0–255, missing alpha defaulting to opaque. Fewer than
/// three entries, or a non-table / non-numeric value, yields `None` (prop ignored).
fn parse_color(t: &Table, key: &str) -> Option<Color> {
    let arr: Table = t.get::<Option<Table>>(key).ok().flatten()?;
    let nums: Vec<u8> = arr
        .sequence_values::<f64>()
        .filter_map(|v| v.ok())
        .take(4)
        .map(|v| v.clamp(0.0, 255.0) as u8)
        .collect();
    if nums.len() < 3 {
        return None;
    }
    Some(Color {
        r: nums[0],
        g: nums[1],
        b: nums[2],
        a: nums.get(3).copied().unwrap_or(255),
    })
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
