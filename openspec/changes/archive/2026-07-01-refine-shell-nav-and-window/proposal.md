## Why

The shell's window opens smaller than the tool surface now wants (the two-pane
layout plus a header is cramped at 800 × 600), the navigation column can only
show a widget's name as a flat button (no per-widget nav UI), and there is no way
to pick up on-disk widget edits without restarting the whole tray process. This
change grows the window, lets a widget draw a custom **preview** of itself in the
navigation column sharing the same live state as its main view, and adds an
in-app reload.

## What Changes

- **Window size → 1024 × 768.** Both the initial (opening) size and the enforced
  minimum resize floor become 1024 × 768 DIPs (up from the current 800 × 600
  minimum, and up from reactor's default opening size). The initial size uses
  reactor's `App::inner_size`; the floor stays on `App::inner_constraints`.
- **Reload-widget action in the config view.** The config view gains a reload
  button that re-discovers and reloads the widget from the `widgets/` folder on
  disk, rebuilding its runtime from scratch (in-memory Lua state is discarded — a
  fresh load) so edits to a widget's files are picked up without restarting the
  app. The button label is pluralized by the count of installed widget folders
  (`Reload widget` for one, `Reload widgets` otherwise). Reloading leaves the
  active view unchanged (the user stays on the config view); the refreshed widget
  shows in the always-visible navigation column.
- **Navigation entry becomes a full-width clickable card with an optional custom
  preview.** The nav entry is rendered as a full-width tappable `border`
  (reactor's generic `on_tapped` gesture on any element) that activates the widget
  view when clicked. A widget's Lua module MAY export an optional `nav(state)`
  function that draws a **preview** of the widget inside that card, using a
  display-only subset of the UI vocabulary; it receives the **same** `state` table
  as `render`, so the preview stays live as the widget's main view mutates state.
  When a widget provides no `nav`, the card shows the widget's name. The preview is
  display-only (no interactive controls); a button placed in a `nav` tree renders
  as an inline error. The card shows an animated hover highlight (an opacity-faded
  fill) so it reads as clickable despite not being a native button.
- **New `border` UI node.** The widget UI vocabulary gains a `border` node
  (usable in both `render` and `nav`) with visual props — `corner_radius`,
  `border_thickness`, `padding` (numbers) and `background`/`border_color`
  (`{r,g,b}`/`{r,g,b,a}`) — so a widget can style its nav card (and use cards in
  its main UI). Unknown props are ignored (forward-compatible). Host-owned
  behavior (the card's tap-to-activate and full-width stretch) is not
  Lua-overridable.

This touches Windows-only (`cfg(windows)`) code only — `src/window.rs` (sizing),
`src/shell.rs` (nav card + reload button + reload wiring), `src/widget/runtime.rs`
(`nav` contract, `border` node, display-only nav mapper), and
`src/widget/package.rs` (widget count). No change to the cross-build/staging path
(`scripts/winrun.py`) or the dev-only Node layer.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `themed-window`: the minimum window size becomes 1024 × 768 (was 800 × 600),
  and a new requirement fixes the window's initial opening size at 1024 × 768.
- `main-content-layout`: the navigation entry becomes a full-width clickable card
  that shows a widget-drawn preview (or the widget name) and activates the widget
  view on click; the config view gains a reload action whose label is pluralized
  by the installed-widget count.
- `tool-surface`: the host-loaded widget becomes reloadable from disk on demand,
  rebuilding its runtime and discarding in-memory state, without changing the
  active view.
- `widget-runtime`: the module contract extends from `{ state, render }` to
  `{ state, render, nav? }`, where `nav` is an optional **display-only** preview
  render sharing the module's single `state` table with `render`; and the UI
  vocabulary gains a `border` node.

## Impact

- **Code:** `src/window.rs` (add `inner_size`, bump `MIN_INNER_SIZE` to
  1024 × 768); `src/shell.rs` (nav rendered as a full-width tapped `border`
  wrapping the preview or name, reload button in `config_view` with a
  count-based label, reload wiring against the widget `use_ref`);
  `src/widget/runtime.rs` (resolve + hold an optional `nav` function, a
  display-only nav mapper, the `border` node in the vocabulary);
  `src/widget/package.rs` (`count_widgets`); `src/widget/mod.rs` (expose reload +
  count entry points). `widgets/counter/main.lua` gains a `nav` preview to
  exercise the new path.
- **APIs/deps:** no new dependencies; uses reactor's existing `App::inner_size`,
  the `border` builder, `ThemeRef` brushes, the generic `on_tapped` /
  `on_pointer_entered`/`on_pointer_exited` gesture handlers, and
  `with_opacity_transition` for the animated hover.
- **Systems:** no change to staging, tray, single-instance, or flash-free
  startup behavior.

## Non-goals

- Persisting widget state across reloads or restarts (reload deliberately resets
  state; persistence remains future work).
- A real configuration surface — the config view stays a placeholder apart from
  the new reload action (it still reads/writes no settings).
- Interactive controls in the nav preview (the nav is display-only this change),
  and resolving how nested interactivity would coexist with the card's tap.
- Loading or listing more than one widget, hot-reload/file-watching, or a
  multi-widget navigation list (reload is manual, single-widget).
- Theme-ref (theme-aware) custom colors for the `border` node, keyboard/AT
  focusability of the nav card, and an active-view selection highlight — all
  deferred.
