## 1. Window size → 1024 × 768 (`src/window.rs`)

- [x] 1.1 Bump `MIN_INNER_SIZE` from `(800.0, 600.0)` to `(1024.0, 768.0)`, and add a sibling `INIT_INNER_SIZE: (f64, f64) = (1024.0, 768.0)` const with a doc comment noting it is the opening size (distinct from the resize floor).
- [x] 1.2 In `run`, add `.inner_size(INIT_INNER_SIZE.0, INIT_INNER_SIZE.1)` to the `App` builder alongside the existing `.inner_constraints(..)` (now using the bumped min), keeping the CBT hook armed first so startup stays flash-free.
- [x] 1.3 Update the `MIN_INNER_SIZE` doc comment and any 800×600 references in module docs to 1024×768. Verify on Windows via `python3 scripts/winrun.py`: window opens at 1024×768 and will not drag below it.

## 2. `border` node + `nav` contract + display-only mapper (`src/widget/runtime.rs`)

- [x] 2.1 Add a `border` builder to `install_ui_api` producing a tagged node (`kind = "border"`) carrying its child and props (`corner_radius`, `border_thickness`, `padding`, `background`, `border_color`).
- [x] 2.2 In `node_to_element`, map `border`: wrap the child in reactor's `border(..)` — one array child used directly, multiple children wrapped in an implicit `vstack`, zero → empty. Apply recognized props — numbers → `corner_radius`/`border_thickness`(uniform `Thickness`)/`padding`(uniform `Thickness`); colors (`background`/`border_color`) parsed leniently: up to 4 numeric entries, clamp 0–255, missing alpha → 255, <3 entries or non-numeric → ignore the prop. Ignore unrecognized props. Reuse in both the main and nav mappers.
- [x] 2.3 Add a display-only nav mapper (a variant of `node_to_element` with no `set_tick`/`tick`/`last_error`): maps `vstack`/`hstack`/`text`/`border`; a `button` node → inline error (`⚠ buttons not supported in nav`).
- [x] 2.4 Add `nav: Option<Function>` to `LoadedWidget`, resolved in `load` (`module.get::<Option<Function>>("nav")`; absent/`Nil` → `None`, no error).
- [x] 2.5 Add `LoadedWidget::nav_render(&self) -> Option<NavCard>` (`NavCard { style, content }`): `None` when `nav` is absent; otherwise call `nav(self.state.clone())` (the same `state` handle `render` uses) and map via the display-only mapper. When the `nav` root is a `border`, `style` = its visual props and `content` = its mapped child; otherwise `style` = default card and `content` = the mapped whole tree. Render-time errors surface inline as `content` (no `last_error` cell — nav has no callbacks).

## 3. Widget count (`src/widget/package.rs`, `src/widget/mod.rs`)

- [x] 3.1 Add `package::count_widgets() -> usize`: count immediate subdirectories of `widgets/` (same `path.is_dir()` filter as `discover_first`); `0` when the folder is absent/unreadable.
- [x] 3.2 Re-export as `widget::count()` from `mod.rs`.

## 4. Nav card + animated hover (`src/shell.rs`)

- [x] 4.1 Resolve the `NavCard` for the loaded widget: use `nav_render()` when `Some`, else a default-style card with `text_block(name)` as content. Default card style = `ThemeRef::CardBackground` + `ThemeRef::CardStroke` (1px) + `corner_radius(6)` + `padding(10)`; a widget-supplied border root's props override the style.
- [x] 4.2 Add a `hovered` `use_state(bool)`. Assemble the card: an outer `border` styled by the resolved style, whose child is `grid((hover_fill, content))` — `hover_fill` = a `ThemeRef::SubtleFill` border (matched corner radius) with `.opacity(if hovered {1.0} else {0.0}).with_opacity_transition(Duration::from_millis(150))`, `content` in the same cell above it.
- [x] 4.3 Apply host-owned modifiers last (not Lua-overridable): `.on_pointer_entered(set_hovered→true)`, `.on_pointer_exited(set_hovered→false)`, `.on_tapped(set_show_config.setter(false))` (activate widget view), `.horizontal_alignment(HorizontalAlignment::Stretch)` (fill the 200-DIP column).
- [x] 4.4 Keep the fallbacks: no widget / failed load → heading only, no card; borrow the `slot`/`LoadedWidget` once so nav and the right-container widget rendering don't double-borrow the `RefCell`.

## 5. Reload action (`src/shell.rs`)

- [x] 5.1 Change `config_view` to take the reload label + callback (e.g. `config_view(reload_label: String, on_reload: impl Fn() + 'static)`), rendering a reload `button(reload_label)` wired to it beneath the placeholder text.
- [x] 5.2 In `tool_surface`, compute the label from `widget::count()` (`count == 1 ? "Reload widget" : "Reload widgets"`) and build the reload closure capturing a clone of the widget `use_ref` slot and a clone of `set_tick`: on click, `*slot.borrow_mut() = Some(widget::load_first())` then bump `set_tick`. Do **not** change `show_config` (stay on config).
- [x] 5.3 Verify on Windows via `python3 scripts/winrun.py`: editing a widget's `.lua` then reloading picks up the change; a now-broken widget shows the error (no nav card); state resets; the label pluralizes with the number of widget folders.

## 6. Exercise the new nav path (`widgets/counter/`)

- [x] 6.1 Add a `nav = function(state) ... end` to `widgets/counter/main.lua` returning a `border` preview (e.g. `border{ padding = 8, corner_radius = 6, vstack { text('Counter'), text('clicks: ' .. state.count) } }`) to demonstrate the preview + shared state + `border` props. Leave the widget working unchanged if `nav` were omitted.

## 7. Build + verification

- [x] 7.1 Keep the Linux host stub green: `cargo build` and `cargo test` succeed (Windows UI code is `cfg(windows)`-gated; these only compile the stub).
- [x] 7.2 Full Windows smoke test via `python3 scripts/winrun.py`: window opens at 1024×768 with a 1024×768 floor; the nav card fills the column, shows a hover fill (spanning the whole card) that fades in/out (~150ms), and clicking it returns from config to the widget; the counter's `nav` preview mirrors the main view live (shared state) and shows its border styling; the config reload button reloads from disk, resets state, keeps the config view active, and pluralizes its label by widget count.
