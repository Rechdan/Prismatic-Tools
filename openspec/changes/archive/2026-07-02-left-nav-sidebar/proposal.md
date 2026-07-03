## Why

The window body is currently split into a top header row (app name left; Configs +
GitHub right) and, below it, a two-pane region (navigation column + selected view).
Splitting the app-level actions across two axes — some in the header, navigation in
the left column — scatters the chrome. Consolidating everything the app owns into a
single left navigation sidebar gives one predictable place for app identity,
view-switching, and app-level actions, and hands the entire remaining window width
to the selected view.

## What Changes

- **The top header row is removed.** The window body is no longer "header on top
  plus a two-pane region below." The two-pane region (left navigation sidebar +
  right selected view) becomes the whole padded window body.
- **The navigation sidebar becomes the single home for app chrome**, laid out as a
  three-row column — a pinned title on top, a scrollable middle, and a pinned
  action group on the bottom:
  - **Pinned title (top):** the app title `Prismatic Tools` (relocated from the
    removed header) as a plain, non-clickable label, always visible.
  - **Scrollable middle (fills the free height):** the **Home** entry, then the
    `Tools` heading and the loaded widget card (all unchanged in behavior),
    wrapped in a vertical scroll region so an overlong tool list scrolls instead of
    overrunning the pinned bottom group.
  - **Pinned action group (bottom):** the **Configs** entry above the **GitHub**
    link, both relocated from the removed header, both full-width, always visible at
    the column's bottom edge.
- **The tool list overflows gracefully.** The middle region scrolls vertically when
  its content exceeds the available height, so the pinned title and the pinned
  bottom actions stay put and Configs/GitHub never clip. (Previously the body did
  not scroll at all.)
- **The active view's nav entry is highlighted** with a soft selected fill, so the
  sidebar shows which view is current. Exactly one of the three view entries — Home,
  the widget card, or Configs — is highlighted at a time, tracking the existing
  `view` state; the GitHub link (an external action, not a view) never highlights.
  Home and Configs stay real (keyboard-focusable) `button`s rendered `.subtle()`
  (transparent at rest) and layered over a persistent selection-fill overlay whose
  opacity is driven declaratively by `view`; the widget card gains the same kind of
  persistent selection-fill overlay beneath its existing hover fill (the two
  compose). The fill is purely declarative from `view` — no toggle/checked control
  state — so re-selecting the active entry is a harmless no-op and the highlight
  never desyncs. This is host-owned styling in `src/shell.rs` — the widget `nav`
  renderer and Lua contract are untouched.
- **Configs is now activated from the navigation sidebar**, not from a header
  button. It still selects (does not toggle) the config view in the right
  container; the widget view is still returned to via the widget card.
- **GitHub link relocates unchanged** — still opens
  `https://github.com/Rechdan/Prismatic-Tools` in the default browser, now from the
  sidebar's pinned bottom group.
- **The `app-header` capability is deleted, spec folder and all.** Its four
  requirements are removed here (the header ceases to exist); because no requirement
  remains, the `openspec/specs/app-header/` folder is deleted outright as part of
  this change rather than left as a hollow, requirement-less spec.
- The right container's three views (home / widget / config), the 200-DIP fixed
  nav width, the widget card + hover highlight, the Home entry behavior, the config
  placeholder + reload action, and the window-edge inset are all **unchanged** in
  behavior — they are only re-parented into the new single-column body.

This touches Windows-only (`cfg(windows)`) code only — `src/shell.rs` (drop the
header grid, restructure the nav column into a pinned-title / scrollable-list /
pinned-actions three-row grid, re-parent the body as the two-pane grid). No change
to `src/window.rs`, the cross-build/staging path (`scripts/winrun.py`), or the
dev-only Node layer.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `app-header`: **removed as a capability, spec folder deleted.** The header row
  ceases to exist; its four requirements (window-edge inset, app name, Configs
  action, GitHub link) are removed here — the inset and the three chrome elements
  relocate into `main-content-layout` (the navigation sidebar). No requirement
  survives, so `openspec/specs/app-header/` is deleted as part of this change.
- `main-content-layout`: the window body drops the header and becomes the two-pane
  region directly, filling the whole padded window; the left navigation column
  becomes a three-row layout — a pinned app title on top, a vertically scrollable
  middle holding the Home entry, `Tools` heading and widget card, and a pinned
  bottom action group holding the Configs and GitHub entries (relocated from the
  header); the config view is now activated from the sidebar's Configs entry rather
  than a header button; the Home entry now sits below the pinned title inside the
  scrollable region; and the active view's entry (Home, widget card, or Configs) is
  highlighted with a soft selected fill.

## Impact

- **Code:** `src/shell.rs` only — remove the `header` grid and the outer
  header-row/main-row body grid; make the two-pane grid the window body (keep its
  `.margin(16.0)` inset); restructure the nav column from a flat `vstack` into a
  three-row `Grid` (`rows([Auto, Star, Auto])`) — an `Auto` top row holding the
  pinned app title, a `Star` middle row holding a `scroll_viewer` around the
  Home + `Tools` heading + widget-card `vstack`, and an `Auto` bottom row holding
  the pinned Configs + GitHub `vstack`; move the `set_view.setter(View::Config)`
  wiring onto the sidebar Configs entry; move the `GitHub` `HyperlinkButton` into
  the sidebar. Render the Home and Configs `button`s `.subtle()` and layer each over
  a `view`-driven selection-fill overlay, and add the same persistent selection-fill
  overlay to the widget card (beneath its hover fill). No signature changes to
  `config_view` or the widget runtime; `src/widget/` is untouched.
- **Specs:** delete the `openspec/specs/app-header/` folder (all four requirements
  removed; the capability no longer exists).
- **APIs/deps:** no new dependencies; reuses reactor's `Grid` (`Auto`/`Star`/`Auto`
  rows — the `Star` middle absorbs free height to pin the title and the bottom
  group), `scroll_viewer` (vertical, bounded by the `Star` cell), `vstack`,
  `button` with `.subtle()` (transparent so the selection fill shows through on
  Home/Configs), `HyperlinkButton`, a soft `ThemeRef` fill (e.g. `ControlFill`) plus
  the existing `on_pointer_*`/`on_tapped` + `with_opacity_transition` overlay pattern
  for the selection/hover fills, and the existing `View` state.
- **Systems:** no change to the tray, single-instance guard, flash-free startup,
  window sizing, home/README rendering, or the widget runtime.

## Non-goals

- Adding new views, new nav entries, or a multi-widget navigation list (still a
  single loaded widget, unchanged).
- A real configuration surface — the config view stays a placeholder apart from the
  existing reload action.
- Restyling the sidebar *beyond* the active-view selection highlight: no icons, no
  section dividers, no collapsing, no change to the 200-DIP nav width, and no change
  to the widget card's existing look or hover highlight (the selection fill is added
  alongside the hover fill, not a redesign of it).
- Keyboard/AT focus order changes beyond what re-parenting the existing accessible
  controls naturally yields.
- Any change to `src/window.rs`, the staging pipeline, or the dev-only Node layer.
