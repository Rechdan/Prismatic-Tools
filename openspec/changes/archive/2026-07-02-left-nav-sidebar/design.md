## Context

Today `tool_surface` in `src/shell.rs` builds the window body as a two-row `Grid`:
an `Auto` header row (a two-column grid — app name star-left, a `Configs` button +
`GitHub` `HyperlinkButton` group auto-right) on top, and a `Star` main row holding
the two-pane grid (a fixed 200-DIP nav column + a star right container). The nav
column is a flat `vstack` of `[Home button, "Tools" heading, widget card?]`.

This change dissolves the header and folds all app-level chrome into the single left
navigation sidebar. Nothing about the three views, the `View` state machine, the
widget card, the hover highlight, the config placeholder, the reload action, or
window sizing changes in behavior — the work is re-parenting existing elements into
one column and adding a top/bottom split so Configs + GitHub anchor to the column
bottom. It is confined to `src/shell.rs` (`cfg(windows)`), so it must be typechecked
with `cargo check --target x86_64-pc-windows-gnu` (the Linux stub does not compile
this code).

## Goals / Non-Goals

**Goals:**
- Remove the top header row; make the two-pane grid the whole padded window body.
- Give the navigation column a three-zone layout: a pinned app title on top, a
  vertically scrollable middle (Home, `Tools` heading, widget card), and a bottom
  action group (Configs, GitHub) pinned to the column's bottom edge. The middle
  region scrolls on overflow so the pinned title and pinned bottom group never move
  or clip.
- Relocate the Configs action and GitHub link into the sidebar with identical
  behavior (Configs selects-not-toggles the config view; GitHub opens the repo URL).
- Highlight the active view's entry (Home / widget card / Configs) with a soft
  selected fill that tracks the existing `view` state, keeping the highlighted
  controls keyboard-accessible.
- Keep the 200-DIP nav width, the widget card's existing look + hover highlight, the
  Home entry, the window-edge inset, the config placeholder, and the reload action
  byte-for-byte equivalent in behavior (the selection fill is added alongside, not a
  redesign).

**Non-Goals:**
- New views, new nav entries, multi-widget lists, sidebar restyling *beyond* the
  active-view selection highlight (no icons, dividers, collapse), or nav-width
  changes.
- Any change to `src/window.rs`, the widget runtime (`src/widget/`, the Lua `nav`
  contract, the counter widget), the home/README rendering, the staging pipeline, or
  the dev-only Node layer.

## Decisions

### Nav column is a three-row `Grid` (`Auto`/`Star`/`Auto`) with the middle scrolling

The nav column becomes a `Grid` with three rows, occupying `grid_column(0)` of the
outer two-pane grid:

- **Row 0 (`Auto`) — pinned title:** the `Prismatic Tools` label. `Auto` sizes to the
  title's own height, so it stays pinned at the top.
- **Row 1 (`Star`) — scrollable tool list:** a `scroll_viewer` wrapping the
  `vstack((Home, "Tools" heading, widget card?))`. The `Star` row absorbs the
  column's free height; per the reactor note, a `scroll_viewer` inside a `Star` grid
  cell is bounded by that cell and scrolls (vertical `Auto`, horizontal `Disabled`
  by default — correct for a nav) with no explicit height. When the list fits, no
  scrollbar shows; when it overflows, only this region scrolls.
- **Row 2 (`Auto`) — pinned bottom actions:** a `vstack((Configs, GitHub))`. `Auto`
  hugs its content at the column's bottom edge.

Because row 1 is the only `Star` row, it soaks up all slack, keeping rows 0 and 2
pinned to the top and bottom edges respectively.

- **Why a `Grid`:** `vstack` only top-packs its children — it cannot pin a subgroup
  to the bottom or bound a scroll region. The `Star` middle row both pushes the
  bottom group down and gives the `scroll_viewer` the bounded height it needs. This
  reuses reactor primitives already in this file (the removed header used the same
  `Star`/`Auto` idea rotated to columns), adds no dependency, and needs no `WM_*`
  plumbing.
- **Why the title and Home are split across rows 0 and 1:** the title is pinned
  branding (always visible); Home is a nav list entry that belongs with the
  scrollable list. Pinning only the title keeps the pinned chrome minimal and lets
  the tool list — the part that can grow with a tall widget `nav` preview — own the
  scroll region.
- **Alternative — a two-row grid with the whole top group (title + list) in the
  `Star` row:** simpler, but the pinned-title decision (title stays visible while
  the list scrolls) requires the title in its own `Auto` row. Rejected in favor of
  the pinned title.
- **Alternative — scroll the entire column as one `scroll_viewer`:** would unpin the
  bottom group (Configs/GitHub would float under the list when content is short
  instead of sitting at the column bottom). Rejected — it breaks the bottom-pin.
- **Alternative — a `vstack` with a flexible spacer child:** reactor has no "spring"
  element that eats free space between siblings, so this cannot pin reliably.
  Rejected.

The `scroll_viewer`'s sole child is the tool-list `vstack` (a real `StackPanel`), not
a bare multi-child fragment — the reactor note warns that a `Group` as a
`ScrollViewer`'s sole child panics.

### Body becomes the two-pane grid directly (drop the outer two-row grid)

The outer `grid((header.grid_row(0), main.grid_row(1)))` with `rows([Auto, Star])`
is deleted. The two-pane grid (`columns([Pixel(200), Star]`)) becomes the returned
body element and keeps the existing `.margin(16.0)` inset (the window-edge padding
requirement, now owned by `main-content-layout`). One less grid nesting level.

### App title is a pinned, plain label in row 0

The `text_block("Prismatic Tools").font_size(20.0).bold()` currently in the header
becomes row 0 of the nav grid, kept a plain non-clickable label (no `on_click` — one
home affordance, the Home entry, avoids a duplicate). At `font_size(20).bold()` it is
~155 DIP wide, fitting the 200-DIP column on one line. It keeps its font size/weight,
drops the header-specific `grid_column`/`vertical_alignment(Center)` props, and gets a
little bottom spacing so it reads as a header rather than a list item.

### Configs + GitHub relocate unchanged, wired to the same `View` state

The `Configs` action keeps its `set_view(View::Config)` wiring; the `GitHub`
`HyperlinkButton` keeps its `navigate_uri`. Both move into row 2's `vstack` — Configs
above GitHub — and are stretched to the column width for a consistent sidebar look
(matching the Home entry and widget card). The `View` enum, the `use_state`, and the
right-container `match` are untouched.

### Active-view selection highlight: a `view`-driven fill overlay under every entry (no toggle state)

The active view's entry gets a soft persistent "selected" fill, driven entirely by
the existing `view` state (host-owned — no `src/widget/` or Lua-contract change). The
three view entries participate (Home, widget card, Configs); GitHub, an external
action, does not. All three use the **same declarative overlay mechanism** so they
read identically:

- **Widget card → a second overlay layer.** The card is already a
  `grid((hover_fill, content))` where `hover_fill` is a `SubtleFill` border whose
  opacity crossfades on `hovered`. Add a persistent `selection_fill` layer beneath
  the hover fill — `grid((selection_fill, hover_fill, content))` — whose opacity is
  `1.0` when `view == View::Widget`, with the same `with_opacity_transition`. A soft
  `ThemeRef` (e.g. `ControlFill`), distinct from the hover's `SubtleFill`. Selected
  and hover compose: a hovered selected card shows both.
- **Home + Configs → a real `button` layered over the same fill.** Each entry is
  `grid((selection_fill, button(label).subtle().on_click(set_view(thisView))))` with
  the button `horizontal_alignment(Stretch)`. `.subtle()` makes the button
  transparent at rest, so the `selection_fill` behind it shows through when
  `view == thisView` (opacity `1.0`, else `0.0`, animated). The entry stays a real
  `Button` — keyboard focus / Enter activation preserved.

**Why a declarative fill and not `ToggleButton` (the important finding).** The obvious
choice — `toggle_button(label, view == thisView)` with `on_checked → set_view` — is
**broken in this reactor**, verified against the crate source:

- `SetState::call` early-returns when the new value equals the old
  (`if *prev == value { return; }`), so re-selecting the already-active view
  schedules **no re-render**.
- The prop reconciler skips a prop whose virtual value is unchanged
  (`if *prev == value { return; }`), so even on a render an unchanged `is_checked`
  (`true → true`) is **not re-applied** to the live control.
- Reactor attaches an `Unchecked` handler to `ToggleButton` that fires `on_checked(false)`.

Together: clicking the currently-selected toggle flips WinUI's `IsChecked` to `false`,
fires `on_checked(false)` → `set_view(sameView)` → early-return → the control stays
visually **unchecked while its view is still active**, and nothing re-checks it. A
selected nav item that deselects itself on re-click. The declarative fill sidesteps
this entirely — there is no control-side checked state to desync; the fill's opacity
is recomputed from `view` on every render and re-clicking the active entry simply
leaves it lit.

Rejected alternatives:

- **`ToggleButton` (native checked fill):** the desync above. Rejected.
- **`RadioButton` (`checked = view == thisView`):** correct semantics (reactor wires
  only `Checked`, radios ignore re-click, no desync), but renders as a radio dot +
  label — wrong for a full-width sidebar entry. Rejected on appearance.
- **Convert Home/Configs to tapped `border`s (drop the button):** pixel-matches the
  card but **regresses accessibility** (a tapped border is not a `Button` — no
  focus/Enter/narration). Rejected; the `.subtle()` button keeps AT.
- **Solid `.accent()` fill:** unmistakable but reads as a primary CTA, too heavy for a
  sidebar (see the proposal's soft-fill decision). Rejected.
- **One shared fill toggled by `hovered || selected`:** conflates selected and hover
  into the same look. Rejected in favor of the separate composing layer.

Minor tuning left to the Windows run: pick/confirm the `selection_fill` `ThemeRef` so
it reads as "selected" and stays distinct from the `SubtleFill` hover; confirm the
`.subtle()` button is transparent enough at rest for the fill to show and that its
own hover chrome composes acceptably over the selection fill.

## Risks / Trade-offs

- **[Sidebar buttons stretched full-width look different from the old header
  buttons]** → Accept: this is the intended sidebar look; full-width stretch matches
  the existing Home button and widget card already in the column.
- **[The `Auto`/`Star`/`Auto` nav grid interacts with the 200-DIP fixed column
  width]** → Low risk: the inner nav grid's width is governed by the outer column's
  `Pixel(200)`; the inner grid only splits height. Verified visually at run time
  (resize taller/shorter to confirm the title stays pinned top, the bottom group
  stays pinned bottom, and only the middle scrolls).
- **[The widget card's hover/tap inside the `scroll_viewer`]** → The card is a
  tapped `border` with `on_pointer_entered`/`exited` now nested in the scroll region.
  Pointer events should still fire (scroll viewers pass them through), but this is
  the one interaction the re-parenting could disturb. Mitigation: confirm on the
  `winrun` check that hover highlight and tap-to-activate still work while the list
  is scrollable.
- **[`scroll_viewer` shows a scrollbar or fails to bound in the `Star` cell]** →
  Low risk: the reactor note documents the `Star`-cell-bounds-a-scroll_viewer
  pattern (the right container already hosts the home view this way). Confirm no
  scrollbar appears when the short default list fits.
- **[Typecheck gap]** → The change is `cfg(windows)`-only; `cargo build` on Linux
  will not catch errors in it. Mitigation: typecheck with
  `cargo check --target x86_64-pc-windows-gnu` and run via `yarn win` (or `yarn dev`)
  before considering it done.
- **[Focus/tab order shifts]** → Re-parenting moves Configs/GitHub from the header
  into the sidebar bottom; tab order now flows title → Home → widget card → Configs →
  GitHub. This is the natural reading order and acceptable; no explicit tab-index
  work is in scope.
