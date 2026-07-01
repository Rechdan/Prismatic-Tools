## Why

The window currently stacks the header directly on top of the hosted widget in a
single padded column — there is no distinct "main" region, no persistent
navigation, and nowhere for a settings surface to live. To grow beyond a single
always-visible widget we need a real content layout: a fixed header on top, a
main region that fills the rest of the window, a persistent left navigation
column, and a right container whose content can swap between the active widget
and a config surface.

## What Changes

- Restructure the window body from `vstack(header, widget)` into a header on top
  plus a **main** region that fills the remaining vertical space (a two-row grid:
  `Auto` header row, `Star` main row).
- Make the main region a **persistent two-pane layout**: a fixed **200 DIP** left
  navigation column beside a flexible right container (a two-column grid:
  `Pixel(200)` + `Star`). Both panes stay visible at all times.
- The right container shows a **selected active view**: the **active widget by
  default**, or a **config placeholder view**. The header **Configs** button
  activates the config view; the widget's nav entry activates the widget view.
  Selection is explicit (not a single toggle). The left nav stays visible in both.
- The left navigation column shows the loaded widget's **manifest name** (e.g.
  `Counter`) as a **clickable entry** that activates the widget view. When no
  widget loads (or it errors), the nav shows its heading only and the right
  container shows the existing notice/error.
- Extend `LoadedWidget` to expose its manifest name via a `name()` accessor —
  consuming the previously-unused, manifest-declared `name` field.
- Move the hosted widget from directly-below-the-header into the right container.
- All work is in Windows-only (`cfg(windows)`) code — `src/shell.rs` render
  functions plus a small additive accessor in `src/widget/runtime.rs`. No change
  to the cross-build/staging path (`scripts/winrun.py`) or the dev-only Node
  layer. No new dependencies.

## Capabilities

### New Capabilities
- `main-content-layout`: the window body below the header — a main region that
  fills the remaining space; a persistent two-pane layout (fixed 200 DIP nav
  column + flexible right container); the right container swapping between the
  active widget and a config placeholder; and the nav column showing the active
  widget's name.

### Modified Capabilities
- `tool-surface`: the loaded widget is now rendered inside the right container of
  the main region's two-pane layout (when the config view is not active), not
  directly below the header.
- `app-header`: the **Configs** button changes from a no-op placeholder to a
  toggle that swaps the right container between the active widget and the config
  view — a spec-level behavior change to the "Configs button is a placeholder"
  requirement.

## Impact

- `src/shell.rs`: `tool_surface` splits into a header + main layout; new render
  helpers/expressions for the persistent two-pane layout, the left navigation
  column (a clickable widget entry), and the config placeholder view; a
  `show_config` view-mode `use_state` selecting the right container's content;
  the Configs `on_click` activates config (`setter(true)`) and the widget nav
  entry activates the widget (`setter(false)`).
- `src/widget/runtime.rs`: `LoadedWidget` gains a stored `name` (from
  `source.manifest.name`) and a `pub fn name(&self) -> &str` accessor. Additive —
  no change to the widget contract or existing render behavior.
- Uses existing reactor primitives only — `grid` with `.rows`/`.columns`
  (`GridLength::Auto`/`Pixel`/`Star`) and `grid_row`/`grid_column`. No API
  additions, no new crates.
- Specs: new `openspec/specs/main-content-layout/`; deltas to
  `openspec/specs/tool-surface/` and `openspec/specs/app-header/`.

## Non-goals

- No real multi-widget selection — the nav has a single clickable entry for the
  one currently-loaded widget (per-widget custom nav rendering, and listing
  multiple widgets, are deferred to a future change).
- No real settings — the config view is a visual placeholder that reads and
  writes nothing.
- No config entry in the nav — config is reached only from the header Configs
  button.
- No scroll containers, and no changes to window chrome, minimum size, tray
  behavior, theming, padding amounts, or the widget runtime contract.
