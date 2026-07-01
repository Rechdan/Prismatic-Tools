## Context

Today `shell::tool_surface` renders `vstack((header_grid, content)).spacing(12).padding(16)`:
the header (a two-column grid — app name `Star`, buttons `Auto`) sits above the
loaded widget in a single vertical stack. There is no dedicated "main" region, no
persistent navigation, and nowhere for settings to live. The Configs button is a
no-op, and the loaded widget's manifest `name` is parsed then discarded.

This change restructures the body into a header on top plus a main region that
fills the rest of the window. The main region is a **persistent two-pane
layout**: a fixed 200 DIP left navigation column and a flexible right container.
The right container shows the active widget by default and swaps to a config
placeholder when the header Configs button is clicked (and back on a second
click); the left nav stays visible throughout and shows the loaded widget's
manifest name. All layout is declarative reactor UI in `src/shell.rs`; the only
non-shell change is a small additive accessor in `src/widget/runtime.rs`.

Confirmed reactor primitives: `grid(children)` with `.rows(..)` / `.columns(..)`
taking `GridLength::{Auto, Pixel(f64), Star(f64)}`, plus the `.grid_row(i)` /
`.grid_column(i)` element modifiers (already used for the header). Grid children
default to Stretch alignment, so a `Star` cell's child fills the cell.

## Goals / Non-Goals

**Goals:**
- Header on top sized to content; a main region below that fills all remaining
  vertical space and tracks window resizing.
- A persistent two-pane main region: fixed 200 DIP left nav column + flexible
  right container; both panes always visible.
- Right container shows the active widget by default and the config placeholder
  when toggled, without hiding the nav column.
- Header Configs toggles the right container's content between widget and config.
- Left nav shows the loaded widget's manifest name (static); heading-only when no
  widget/error.
- Keep the existing window-edge padding and the widget load/render/error path
  intact.

**Non-Goals:**
- No multi-widget list or selection logic — the nav label is a static name, not a
  clickable selector; per-widget custom nav rendering is deferred.
- No config entry in the nav — config is reached only from the header Configs
  button.
- No real settings storage or config UI beyond a placeholder; no scroll
  containers.
- No changes to `window.rs`, the tray, theming, min-size, or the widget contract.

## Decisions

### Config model: nav persists, config fills the right container
Config does **not** replace the whole main region. The two-pane layout is the
permanent structure of the main region; only the right container's content swaps
between the active widget and the config placeholder. The left nav stays visible
in both states. This was chosen over a "config replaces everything" model so the
navigation surface is always present and the layout never collapses — and it is
future-proof for when the nav lists multiple tools.

**Active-view selection, not a toggle** (supersedes the original grilling
decision that Configs toggled and the nav label was static): the right container
shows a *selected* active view. The header Configs button activates the config
view (`set_show_config(true)`); the widget's nav entry activates the widget view
(`set_show_config(false)`). Clicking Configs while config is already active is a
no-op re-select (it does not toggle back). The way back to the widget is
selecting the widget in the nav — so the widget entry must be interactive, not a
static label. This matches the "active view" mental model (each destination is a
thing you select) rather than one control flipping between two states.

### Body layout: two-row grid, not a vstack
Replace the outer `vstack` with `grid` using `.rows([GridLength::Auto,
GridLength::Star(1.0)])`: the header goes in row 0 (`Auto` → sizes to content),
the main region in row 1 (`Star` → absorbs the rest of the height). A `vstack`
(`StackPanel`) only measures to its children's natural height and top-packs; it
cannot make one child fill the leftover space, which is exactly the "main fills
the rest" requirement. The outer grid keeps `.padding(16.0)` (preserving
`app-header`'s inset requirement) and uses `.row_spacing(12.0)` in place of the
old `vstack` spacing. The header keeps its existing inner two-column grid, now
carrying `.grid_row(0)`.

### Main region: persistent two-column grid `Pixel(200)` + `Star`
The main region (row 1) is `grid((left_nav.grid_column(0),
right.grid_column(1)))` with `.columns([GridLength::Pixel(200.0),
GridLength::Star(1.0)])` and `.column_spacing(12.0)`. `Pixel(200.0)` fixes the
nav at 200 DIP; `Star` gives the right container all remaining width; both
stretch to fill the main row vertically. This grid is always rendered — the
view-mode toggle only changes what goes into the right cell.

### Right container content: a single boolean `use_state`, set explicitly
Add `let (show_config, set_show_config) = cx.use_state(false);` at the top of
`tool_surface`. The right cell is `(if show_config { config_view() } else { content
}).grid_column(1)`, where `content` is the existing widget load/render/error
`Element`. Two entry points set the value explicitly via `SetState::setter`:
Configs uses `set_show_config.setter(true)`, the widget nav entry uses
`set_show_config.setter(false)`. `.setter(v)` is reactor sugar for `move ||
set.call(v)` and clones the (Rc-backed) setter internally, so both handlers can be
built from the same `set_show_config` without a manual clone. A `bool` (not an
enum) suffices for two mutually-exclusive views for now — an enum would generalize
to more views later. The `tick`/`set_tick` widget re-render loop is unchanged and
only matters while the widget is shown.

### Left nav shows the widget's manifest name via a new accessor
`LoadedWidget` currently stores only its private `id` (folder name) and discards
`source.manifest.name`. Extend it to also store `name: String` (from
`source.manifest.name` at `load()`) and add `pub fn name(&self) -> &str`. The nav
renders a "Tools" heading plus a clickable `button(widget.name())` (activating the
widget view) when the load slot is `Some(Ok(w))`; for `Err`/absent it renders the
heading only (the right container already shows the notice/error). This is
additive — it consumes the manifest field the package layer already parses (its
comment calls this a "future feature") without changing the widget contract or
render behavior, so no widget-runtime/widget-package spec requirement changes.

### Config view is a self-contained placeholder helper
A small `config_view()` returning e.g. `vstack((text_block("Configs")
.font_size(20).bold(), text_block("Configuration coming soon.")))`. It reads and
writes nothing, satisfying the placeholder requirement, and is trivially swapped
for a real surface later.

### Keep render helpers inline in `shell.rs`
The new pieces (two-pane layout, left nav, config view) are private helper fns or
inline expressions in `shell.rs`. No new module — this is layout, and `shell.rs`
already owns the surface. Keeps the change to `shell.rs` plus the one-line
runtime accessor.

## Risks / Trade-offs

- **Grid child not filling as expected** → WinUI grid children default to
  Stretch, so `Star` cells fill; verify with `python3 scripts/winrun.py` that the
  main row and right container expand at the 800×600 minimum and when resized. If
  a child fails to stretch, set its alignment to `Stretch` explicitly.
- **200 DIP nav is wide at the 800 DIP min width** → leaves ~560 DIP (minus
  padding/spacing) for the widget, which is adequate; revisit only if the widget
  feels cramped.
- **Returning from config depends on a widget nav entry** → the config view is
  left by selecting the widget in the nav, so with no widget loaded there is no
  nav entry to return through. This is a benign edge (an empty `widgets/` folder):
  the widget view already shows the notice, and config is unlikely to be entered
  there. Revisit when real settings or multiple nav destinations land.
- **Header nested-grid regressions** → the header keeps its own two-column grid;
  only its placement changes (now `.grid_row(0)` inside the body grid). Low risk,
  but confirm the buttons stay pinned right after the move.
- **No visual regression tests** → correctness is confirmed by running the real
  Windows app via `winrun.py`, not by `cargo` on Linux (stub only).

## Open Questions

- Should the left nav eventually drive widget selection (and possibly become a
  WinUI `NavigationView`), with per-widget custom nav rendering replacing the
  static name? Out of scope now; the fixed-column + static-name approach keeps
  this change minimal and reversible.
