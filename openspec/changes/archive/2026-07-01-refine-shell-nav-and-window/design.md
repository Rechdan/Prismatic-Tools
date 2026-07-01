## Context

The shell today (`src/shell.rs`, `src/window.rs`, `src/widget/runtime.rs`) opens
a themed window at reactor's default size with an 800 × 600 DIP resize floor, and
lays out a header over a two-pane body: a fixed 200-DIP nav column beside a right
container. The nav column shows a single entry — the loaded widget's manifest
name as a `button` whose `on_click` selects the widget view (`set_show_config(false)`).
The right container shows either the widget (`LoadedWidget::render(&set_tick, tick)`)
or a static `config_view()` placeholder. The widget is loaded once into a
`use_ref` slot (`widget::load_first()`), and the widget contract is a Lua module
`{ state, render }` run in a sandboxed `mlua` VM; `render(state)` returns a UI
tree mapped by `node_to_element` (`vstack`/`hstack`/`text`/`button`). Button
callbacks mutate the widget's Lua `state`, set `last_error` on failure, then bump
`set_tick` to re-render.

Four refinements land on this surface: a bigger window, a config-view reload
action, a widget-drawn **preview** in the nav column that shares the main view's
state, and a full-width clickable nav card. All changes are `cfg(windows)`-only.

### Reactor capabilities this design leans on (verified in the crate + samples)

- `App::inner_size(w, h)` sets the opening size; `App::inner_constraints` sets the
  resize floor (already used).
- **Every element** (via `ElementExt`) exposes generic gesture handlers —
  `on_tapped`, `on_right_tapped`, `on_pointer_pressed/released/moved/entered/exited`
  — wired through the reconciler's `set_pointer_handlers`. Reactor's clickable
  *widgets* (`Button`, `HyperlinkButton`, …) all take **string content only**, but
  `on_tapped` on a container is the sanctioned way to make an arbitrary element
  tree clickable. Sample evidence: `samples/reactor/apps/examples/solitaire.rs`
  builds every card as `border(vstack((label, suit))).corner_radius(..).background(..)
  .padding(..).on_pointer_released(on_click)`; `dotsweeper.rs`/`minesweeper.rs`
  use `on_tapped`/`on_right_tapped` on cells.
- `border(child: impl Into<Element>) -> Border` wraps an arbitrary element and
  offers `corner_radius(f64)`, `border_brush(Color|ThemeRef)`, `border_thickness(Thickness)`,
  plus the shared `background`/`padding` modifiers (both accept `Color` **or**
  `ThemeRef`) and `on_tapped`. `Color::rgb(u8,u8,u8)` constructs a literal color;
  `ThemeRef::CardBackground`/`CardStroke`/`SubtleFill` are theme-aware brushes
  (Mica-safe; `card.rs` sample uses `.background(ThemeRef::CardBackground)`).
- **Implicit transitions animate opacity / scale / rotation / translation — not
  brush color.** `with_opacity_transition(Duration)` tweens `.opacity(..)` changes
  between renders (`opacity_transition.rs` sample). Generic
  `on_pointer_entered`/`on_pointer_exited` drive a hover state. An animated hover
  *fill* is therefore an opacity-crossfaded overlay, not a background-color tween.

## Goals / Non-Goals

**Goals:**
- Open the window at 1024 × 768 DIPs and clamp resize at that same floor.
- Let the user reload the widget from disk from within the config view without
  restarting the tray process (a fresh load — in-memory Lua state is discarded),
  with a label pluralized by the installed-widget count, leaving the active view
  unchanged.
- Render the nav entry as a full-width clickable card (tapped `border`) that
  activates the widget view, and let a widget draw a live **preview** of itself
  inside that card via an optional `nav(state)` sharing the exact same `state`
  table as `render(state)`.
- Add a `border` node to the widget UI vocabulary so a widget can style its nav
  card (and use cards elsewhere).

**Non-Goals:**
- Persisting widget state across reload/restart (reload deliberately resets it).
- File-watching / automatic hot-reload — reload is an explicit button press.
- Interactive controls inside the nav preview (display-only this change), and the
  nested-interactivity/tap-bubbling question it would raise.
- Theme-ref custom colors on `border`, keyboard/AT focusability of the card, an
  active-view selection highlight, and multi-widget listing.

## Decisions

### 1. Window size via reactor's `App::inner_size` + `App::inner_constraints`

Set the opening size with `App::inner_size(1024.0, 768.0)` and keep the resize
floor on the existing `App::inner_constraints` call, bumping `MIN_INNER_SIZE` from
`(800.0, 600.0)` to `(1024.0, 768.0)`. Add a sibling `INIT_INNER_SIZE = (1024.0, 768.0)`
const in `window.rs`; the two consts carry the same values but name distinct
intents (initial vs. floor). The window therefore opens exactly at its floor and
only grows.

- **Why:** reactor already exposes both builder calls; no Win32
  `WM_GETMINMAXINFO` subclassing is needed (consistent with the existing min-size
  approach). Setting an initial size does not force a show, so it does not
  reintroduce a launch flash (the CBT hook still suppresses the first show).
- **Edge accepted:** on a display smaller than 1024 × 768 the floor exceeds the
  screen; the OS/reactor handles clamping. Out of scope.

### 2. Nav entry is a full-width clickable `border` (tapped card)

Replace the nav name `button` with a **full-width tapped `border`**. The border
carries the host-owned behavior: `.on_tapped(set_show_config.setter(false))`
(activate the widget view) and `.horizontal_alignment(HorizontalAlignment::Stretch)`
(fill the 200-DIP column). This is one uniform path for every widget:

- No `nav` → the card wraps the widget **name** as text (`border(text_block(name))`).
- With `nav` → the card wraps the widget's `nav(state)` preview tree.

Rationale: reactor's clickable *widgets* are string-content-only, so a button
cannot host a custom preview; the sanctioned pattern (per the solitaire/dotsweeper
samples) is a styled `border` with a gesture handler. A single always-a-border
path keeps the look consistent and the code one branch.

- **Affordance:** the default card is styled with theme-aware brushes —
  `ThemeRef::CardBackground` + `ThemeRef::CardStroke` (1px) + `corner_radius(~6)`
  + `padding(~10)` — so it reads as a clickable tile in light/dark Mica
  (`card.rs` sample). This applies to the name-only default and to the wrapper
  when a widget's `nav` root is not a border; a widget that returns a border root
  owns its own look (host adds only behavior).
- **Animated hover:** the card carries a hover state (`use_state(bool)` toggled by
  `on_pointer_entered`/`on_pointer_exited`). Because brush color cannot tween, the
  highlight is an **opacity-crossfaded overlay**: a `ThemeRef::SubtleFill` layer
  in the same grid cell as the content, `.opacity(if hovered {1.0} else {0.0})
  .with_opacity_transition(Duration::from_millis(150))`. Each enter/exit
  re-renders (re-running `nav(state)`) — negligible for one card. This compensates
  for the missing native Button hover semantics. The hover fill (and the base
  `CardBackground`) must span the **whole card**, so the card's `padding` is
  applied to the *content*, not the card frame — otherwise the frame padding insets
  the grid and the hover fill only covers the inner content area.
- **Accessibility trade-off (accepted):** a tapped `border` is not a `Button` in
  the accessibility tree — no tab-focus, no Enter activation, no "button"
  announcement. This is reactor's own idiom (its game samples ship it); accepted
  for this change, mitigation deferred.
- **Alternatives rejected:** (a) `NavigationView` — its `NavViewItem.content` is
  string-only too and it subsumes the whole two-pane layout; a much larger
  re-arch that still can't host a custom preview. (b) forking reactor's `Button`
  to accept element content — out of scope.

### 3. `nav` is an optional, display-only, shared-state preview

Extend the widget module contract from `{ state, render }` to `{ state, render, nav? }`.
In `runtime.rs`, `load` resolves an optional `nav: Option<Function>`. `LoadedWidget`
gains `nav_render(&self) -> Option<Element>`: `None` when `nav` is absent;
otherwise it calls `nav(self.state.clone())` (the **same** `state` handle `render`
uses) and maps the result through a **display-only** node mapper.

- **Display-only mapper:** supports `vstack`/`hstack`/`text`/`border`; it does
  **not** wire button callbacks. A `button` node in a `nav` tree renders as an
  inline error (`⚠ buttons not supported in nav`), consistent with how the
  runtime already surfaces unrecognized nodes. This sidesteps the
  nested-interactivity / tap-bubbling question entirely.
- **No tick, no error cell:** because `nav` has no callbacks, `nav_render` needs
  neither `set_tick`/`tick` nor a `last_error` cell. The single existing
  `last_error` stays on the `render` (main-view) path only. Nav render-time
  failures (a throwing `nav`, a malformed node) surface synchronously inline, the
  same way `render` surfaces `Err(e) => error_text`.
- **Shared state — how the preview stays live:** `render` and `nav` are passed the
  one `self.state` `Table` (an `mlua` `Table` is a reference into the VM). When a
  **main-view** button mutates `state` and bumps `set_tick`, the whole surface
  re-renders; `nav_render` re-reads the shared table and the preview reflects the
  change. No second state cell, no nav-owned tick.

### 4. `border` node in the widget UI vocabulary + host/widget precedence

Add `border` as a first-class node (like `vstack`/`text`/`button`), usable in both
`render` and `nav`. Recognized props: `corner_radius`, `border_thickness`,
`padding` (numbers) and `background`, `border_color` (`{r,g,b}` / `{r,g,b,a}`
arrays → `Color::rgb`/`rgba`). Unknown props are ignored (forward-compatible).

Nav wrapping + precedence:

- If the widget's `nav` **root node is a `border`**, the host applies its behavior
  modifiers (`on_tapped`, `Stretch`) directly onto that mapped border — the widget
  fully styles the tapped card.
- Otherwise (a non-border root, or no `nav`), the host wraps the mapped preview
  (or the name text) in a **default theme-aware border** carrying those modifiers.
- **Host owns behavior, widget owns looks.** The host applies `on_tapped` +
  `Stretch` last (non-overridable). `on_tapped` is **not** exposed to Lua, so a
  widget cannot hijack the view-switch; the `border` node props are visual-only.
- **Color parsing (lenient):** read up to 4 numeric array entries, clamp each to
  0–255, missing alpha → 255; fewer than 3 entries or non-numeric → ignore the
  prop (no color). Consistent with the "unknown props ignored" stance.
- **Nav-card assembly (runtime resolves style, shell assembles):** hover +
  behavior need `cx` state/setters, so they are shell-side. `runtime.rs`'s
  `nav_render` (which knows the Lua node kind) resolves the two style-bearing
  pieces and returns them as `Option<NavCard { style, content }>`: when the `nav`
  root is a `border`, `style` carries its visual props and `content` is its mapped
  child (implicit-vstack if multiple); otherwise `style` is the default card and
  `content` is the mapped whole tree. The **shell** consumes the `NavCard` (or, on
  the no-`nav` fallback, builds one with the default style + name text) and
  assembles the final card: an outer `border` styled by `style.apply_frame(..)`
  (frame props only — background, stroke, radius; **not** padding), whose child is
  `grid((hover_fill.opacity(..).with_opacity_transition(150ms), content.padding(style.padding())))`,
  plus the shell-owned `use_state(hovered)`, `on_pointer_entered/exited`,
  `on_tapped`, and `Stretch`. Padding lives on the content (not the frame) so both
  background layers span the whole card; `BorderStyle` exposes `apply_frame`,
  `padding()`, and `corner_radius()` (the hover fill matches the card's radius),
  while `apply` (frame + padding) still serves the general `border` node in a
  widget's main render.

### 5. Reload widget(s) from the config view

`config_view` gains a reload button. Its `on_click` captures a clone of the
`use_ref` widget slot's `Rc<RefCell<..>>` and a clone of `set_tick`; on click it
does `*slot.borrow_mut() = Some(widget::load_first())` then bumps `set_tick`. A
fresh `load_first()` builds a new `mlua` VM, so the widget's Lua `state` is
recreated from its module definition — in-memory state discarded, on-disk edits
picked up.

- **View unchanged:** reload does **not** touch `show_config`; the user stays on
  the config view. Feedback is still visible because the nav column renders in
  both views — the reloaded widget's card/preview (or, on failure, the
  heading-only state) refreshes immediately.
- **Label pluralized by count:** add `package::count_widgets() -> usize`
  (immediate subdirectories of `widgets/`, the same filter `discover_first` uses;
  `0` when the folder is absent), surfaced as `widget::count()`. The config view
  computes the count at render and labels the button `count == 1 ? "Reload widget"
  : "Reload widgets"` (so `0 → plural`, `1 → singular`, `N → plural`). The button
  stays enabled at `0` so reload can discover a newly added widget.
- **Signature:** `config_view()` → `config_view(reload_label, on_reload)` (or
  built inline in `tool_surface`). A failing reload leaves an `Err` in the slot,
  surfaced by the existing match (right container error when the widget view is
  active, no nav card); a renamed widget updates the card automatically.

## Risks / Trade-offs

- **Tapped `border` is not keyboard/AT accessible** → accepted (reactor's own
  idiom). Mitigation (a focusable wrapper or a reactor change) deferred.
- **Nav preview shows no visual "clickable" cue unless styled** → the default
  card style (background + corner radius + padding) provides the affordance;
  hover states deferred.
- **Reload discards unsaved widget state** → intended (no persistence yet); a
  user mid-interaction loses their counter value. Documented as expected reload
  semantics.
- **Reload while on config gives only nav-column feedback** → acceptable: the nav
  column is visible in both views, so the refreshed card (or heading-only failure
  state) is seen without leaving config.
- **`count_widgets` counts subfolders, not manifest-bearing folders** → a
  manifest-less folder counts as a (broken) widget for labeling; matches
  discovery's candidate filter and avoids per-folder manifest stats.

## Migration Plan

Additive and backward-compatible: existing widgets (no `nav`) get the full-width
name card and render unchanged; the larger window, reload button, and `border`
node are transparent to widget authors. No data migration. Rollback is reverting
the `shell.rs`/`window.rs`/`runtime.rs`/`package.rs` edits. The shipped `counter`
widget gains a small `nav` preview to exercise the path.

## Open Questions

- Deferred (not blocking): a focusable/keyboard-activatable nav card;
  theme-ref custom colors for `border`; an active-view selection highlight on the
  card; whether the nav should ever become interactive (and how tap-bubbling
  would be resolved then).
