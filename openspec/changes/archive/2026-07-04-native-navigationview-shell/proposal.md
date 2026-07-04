## Why

The shell hand-rolls its navigation: a fixed 200-DIP column with a pinned title, a
scrollable tool list, view-driven selection fills, hover crossfades, and a pinned
Configs/GitHub group — all built from `grid`/`border`/`button` primitives. Reactor
ships a native `NavigationView` that provides the same affordances for free (a
collapsible pane with a hamburger/burger toggle, icon+label items, built-in
selection highlighting and keyboard accessibility, and a Settings item). Adopting it
deletes a large amount of bespoke layout/selection/hover code, gives us the platform-
standard WinUI 3 navigation look, and removes the widget-authored nav-preview
mechanism in favor of just the widget's name.

## What Changes

- **BREAKING (widget contract):** Remove the widget module's optional `nav(state)`
  function and the display-only nav-preview mechanism. Widgets are surfaced in the
  pane by their manifest **name** only. The bundled `counter` widget's `nav` preview
  is dropped.
- Replace the custom two-pane `grid` sidebar (`src/shell.rs`) with reactor's
  `NavigationView`: a burger-toggled left pane (`pane_display_mode` Left,
  `pane_toggle_button_visible`) whose menu items are icon+label `NavViewItem`s.
- Nav items become: **Home** (`Symbol::Home`) and the loaded widget (its manifest
  name, tag `widget:<id>`, a default `Symbol`). A widget that **fails to load** is
  surfaced as a selectable **error item** (`Symbol::Important`, labeled with the
  widget folder id) that shows the load error in the content area when selected; an
  **absent** widget shows no item.
- **Configs** is a normal, tag-routed menu item (`NavViewItem::new("Configs")
  .tag("config").icon(Symbol::Setting)`), placed last; the config view (with its
  reload action) shows when it is selected. The NavigationView's built-in Settings
  gear is **disabled** (`settings_visible(false)`): its selection does not deliver a
  routable tag through reactor (reactor's `select_nav_item_by_tag` never walks it, and
  its sample never enables it), so a real menu item is used instead. A deselection
  artifact (empty tag `""`, produced only when a reload rebuilds the menu while Configs
  is selected) also routes to config, keeping content stable.
- **GitHub is not a nav item.** Reactor's reconciler does not re-assert an unchanged
  `selected_tag`, so an item that opens an external browser and must un-highlight
  itself cannot revert cleanly; the project link stays on the README home view.
- Drive the active view from `NavigationView`'s `selected_tag` /
  `on_selection_changed` (tag-based) instead of the bespoke `View` enum + per-entry
  selection fills. The pane title shows `Prismatic Tools`.
- Delete the now-unused custom nav machinery: selection/hover fill overlays, the
  `on_tapped`/`on_pointer_entered`/`on_pointer_exited` card, the `Auto/Star/Auto`
  three-row nav grid, and the `HyperlinkButton` GitHub entry.
- Keep unchanged: the home README view as the default landing view, the tray
  lifecycle, the single-instance guard, the window sizing, and the widget
  load/sandbox/reload pipeline (minus `nav`).

This change touches **Windows-only (`cfg(windows)`) code** (`src/shell.rs`,
`src/widget/`) and the bundled `widgets/counter/` Lua source. It does **not** touch
the cross-build/staging path (`scripts/winrun.py`) or the dev-only Node layer.

## Non-goals

- No multi-widget navigation: the shell still loads only the **first** widget folder
  (`widget::load_first`); this change does not add a per-widget item for every folder.
- No custom pane-footer or per-widget icons chosen by the widget manifest — icons are
  shell-assigned `Symbol`s. Widget-authored icons are out of scope.
- No change to the tray lifecycle, single-instance guard, window sizing, themed
  window, or the widget sandbox/load/reload pipeline (beyond dropping `nav`).
- No change to the home-screen README rendering.
- No new dependencies and no changes to `scripts/winrun.py` or the Node dev layer.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `main-content-layout`: The navigation surface is rebuilt on reactor's native
  `NavigationView` instead of a custom two-pane grid. Requirements for the fixed
  200-DIP column, the pinned app-title label, the three-zone scroll layout, the
  widget-as-card entry, the custom Configs/GitHub bottom group, and the bespoke
  selection/hover highlight are replaced by NavigationView-based requirements (burger
  toggle, icon+label items, pane title, a tag-routed Configs menu item, a broken-widget
  error item, tag-driven selection). GitHub is removed from navigation (it stays
  linked from the README home view). The default-home-view and view-content-inset
  behavior are preserved.
- `widget-runtime`: Remove the optional `nav(state)` function and the display-only
  nav-preview contract; a widget is represented in navigation by its manifest name
  only.
- `tool-surface`: The reload action's description no longer references a "navigation
  card"; on load failure the shell shows the error in the right container with no
  widget nav item (rather than "no navigation card").

## Impact

- **Code:** `src/shell.rs` (nav rebuilt on `NavigationView`; `View` enum replaced by
  tag routing; custom fills/hover/card and the `HyperlinkButton` GitHub entry
  deleted; the `use_effect` window plumbing and the `tick`/`slot`/`set_tick`
  re-render machinery preserved), `src/widget/runtime.rs` + `src/widget/*.rs` (drop
  `nav`/`NavCard`/`nav_render` and the nav-only `BorderStyle` **methods**;
  `BorderStyle`'s fields all feed the main `border` render and stay), `src/widget/`
  gains a `LoadedWidget::id()` accessor, `widgets/counter/main.lua` (drop `nav`).
- **APIs (reactor):** `NavigationView`, `NavViewItem`, `Symbol`,
  `NavigationViewPaneDisplayMode`, `on_selection_changed`. Verified reactor facts
  drive the design: (1) the built-in **Settings gear does not route through reactor's
  tag mechanism** — `select_nav_item_by_tag` never walks it, and clicking it does not
  activate a routable tag (runtime-confirmed: it fell through to the home view), so
  **Configs is a normal tagged menu item and the gear is disabled**; (2) reactor's
  reconciler skips re-applying an **unchanged** `selected_tag`, so selection corrects
  only on a value *change* to a real menu-item tag (this is why GitHub-as-item was
  dropped); (3) reactor rebuilds a changed `menu_items` via `menu.Clear()`, which
  deselects a live item and emits an empty tag `""` — routed to config, since a
  rebuild only occurs on reload while Configs is selected.
- **Dependencies:** none added or removed (dropping GitHub avoids the `ShellExecuteW`
  / `Win32_UI_Shell` feature that a browser-opening item would have required).
- **Docs:** `docs/reactor-notes.md` already notes NavigationView subsumes the
  two-pane layout and that `NavViewItem.content` is string-only — update its
  guidance to reflect that the shell now uses it. `CLAUDE.md` architecture section
  for `src/shell.rs` and `src/widget/` needs updating.
