## Context

`src/shell.rs::tool_surface` currently builds navigation by hand: a two-column
`grid` (`Pixel(200)` nav + `Star` right), a three-row nav sub-grid
(`Auto/Star/Auto`) with a pinned `Prismatic Tools` label, a `scroll_viewer`'d middle
holding a `Home` `.subtle()` button, a `Tools` heading, and a tapped-`border` widget
**card**, and a pinned bottom group with `Configs` and a `HyperlinkButton` GitHub
link. Active view is a `View { Home, Widget, Config }` `use_state`; selection and
hover are hand-animated `ControlFill`/`SubtleFill` overlays crossfaded via
`with_opacity_transition`. The widget runtime additionally supports an optional
`nav(state)` function producing a display-only preview drawn inside the card
(`widget::NavCard`, `nav_render`, plus nav-only `BorderStyle` plumbing).

Reactor ships a native `NavigationView` (wrapper in
`crates/libs/reactor/src/widgets/navigation_view.rs`; sample
`samples/examples/navigation_view.rs`) that provides the burger toggle, collapsible
pane, icon+label items, built-in selection highlight, keyboard accessibility, and a
Settings item. `docs/reactor-notes.md` already records that it "subsumes the whole
two-pane layout (owns both the item list and the content pane)" and that
`NavViewItem.content` is string-only — exactly why the widget card's custom preview
cannot live inside a nav item, which is the motivation to drop `nav`.

Verified reactor API (this checkout, `windows-rs-a5de4a2dc783ec71/a1e9fce`):

- `NavigationView::new(menu_items: IntoIterator<NavViewItem>, content: impl Into<Element>)`
  then builder methods: `.selected_tag(impl Into<String>)`,
  `.on_selection_changed(impl IntoCallback<String>)`,
  `.pane_display_mode(NavigationViewPaneDisplayMode)`,
  `.pane_toggle_button_visible(bool)` (default `true` — the burger),
  `.settings_visible(bool)` (default `true`), `.pane_title(impl Into<String>)`,
  `.header(impl Into<String>)`, `.back_button_visible(bool)`.
- `NavViewItem::new(content).tag(id).icon(Symbol)`; also `NavViewItem::header(text)`
  (non-selectable group header) and `.child(item)` (nesting).
- `Symbol` is `Symbol(pub i32)` with associated consts: `Home` (57615),
  `Setting` (57621), `Document`, `List`, `Link`/`Globe`-family, etc. The nav item's
  icon is rendered as a WinUI `SymbolIcon` (`convert.rs`).
- Selection reporting (`backend/winui/mod.rs`): `on_selection_changed` fires with
  the selected item's **Tag** string. A tag-less item reports `""`
  (`unwrap_or_default()`). **Runtime finding:** selecting the built-in **Settings**
  gear does **not** produce a usable config route — clicking it fell through to the
  home view (its selection is not delivered as a routable tag). Reactor's own sample
  disables the gear (`settings_visible(false)`), so this path is untested upstream.
  Hence Configs is a **normal tagged menu item**, not the gear (D3).
- `menu_items` is the **only** item collection the wrapper exposes — there is no
  custom pane-footer / `FooterMenuItems` binding. The only pane-footer affordance is
  the built-in Settings gear.
- **Reconciler only pushes *changed* props** (`reconciler/diff_helpers.rs`
  `diff_props`): if a render's `selected_tag` equals the previous render's, `set_prop`
  is skipped — reactor does **not** re-assert an unchanged value. So NavigationView is
  *not* a "snap selection back every render" controlled component: the control's
  selection is corrected only when the tag value we pass actually **changes** to a
  real menu-item tag.
- **`select_nav_item_by_tag` walks only `MenuItems` (and children), never the
  built-in Settings item** (`backend/winui/convert.rs`). So `selected_tag("")` (or any
  tag not present as a menu item) highlights **nothing** programmatically; the
  Settings gear is highlighted only by WinUI's native click.
- **`Symbol` link/warning glyphs** (`bindings.rs`): `Symbol::Important` (57713, the
  exclamation) exists for the error item; `Symbol::Document` (57648) for the widget
  item. (`Warning`/`Error` at low ordinals belong to a different InfoBar enum, not
  `Symbol`.) `NavViewItem` has **no** `is_enabled`/disable field — an error item is a
  normal *selectable* item.

## Goals / Non-Goals

**Goals:**

- Replace the bespoke nav column with reactor's `NavigationView`, keeping the
  destinations Home, the widget, and Configs (a tagged menu item), plus the
  home-by-default landing behavior.
- Show a burger toggle and icon+label items.
- Delete the custom selection-fill, hover-fill, tapped-card, and three-row-grid code.
- Remove the widget `nav(state)` preview contract; represent a widget by its name.

**Non-Goals:**

- GitHub as a nav item (dropped — see D4; the README home view keeps the link).
- Multi-widget nav (still `load_first`, one widget item).
- Widget-authored icons or a custom pane footer.
- Any change to tray lifecycle, single-instance guard, window sizing, themed window,
  or the widget sandbox/load/reload pipeline beyond dropping `nav`.

## Decisions

### D1 — Route views by `tag` string, not a `View` enum

`NavigationView` is a controlled component: it takes `selected_tag: String` and
reports the newly-selected item's tag via `on_selection_changed(Callback<String>)`.
We keep a `use_state::<String>` holding the current tag (default `"home"`), pass it
as `.selected_tag(tag.clone())`, and in `on_selection_changed` route on the tag.

Tag namespaces are kept structurally distinct so nothing can collide (see D6):
`"home"` → home README view; `"widget:<id>"` (folder id, e.g. `"widget:counter"`) →
the widget view (its render, or its load-error text — see D7); `""` (empty) →
**Settings/Configs** view. The right-container `content` is chosen by matching the
current tag, mirroring the old `match view { … }`.

The default state is `"home"`; `.selected_tag("home")` highlights the Home item on
first paint (`select_nav_item_by_tag` finds it), matching the default Home view.

*Why over keeping the `View` enum:* the enum can't name "which widget" and duplicates
what NavigationView already tracks; a tag string is exactly what the control emits.
*Alternative considered:* keep `View` and map tag→enum in the callback — rejected as
a redundant second source of truth that can desync from the control's own selection.

*Desync note:* because the reconciler skips unchanged `selected_tag` (Context), the
scheme is only safe if every real menu-item click ends with state holding a tag that
matches the clicked item — which it does for all three destinations: Home→`"home"`,
widget→`"widget:<id>"`, Configs→`"config"`. Removing GitHub (D4) eliminated the only
"click but don't change state" case, which was the sole real desync path. The one
residual non-item tag is `""` (a reload deselection artifact), routed to config (D3).

### D2 — Burger + Left pane display mode

Use `.pane_display_mode(NavigationViewPaneDisplayMode::Left)` with
`.pane_toggle_button_visible(true)` (the default) so the pane is expanded with a
visible hamburger toggle at 1024-DIP width. `.pane_title("Prismatic Tools")` replaces
the old pinned title label.

*Alternative considered:* `Auto` — adapts to width, collapsing below its
`ExpandedModeThresholdWidth` (default **1008 DIP**). Our window is **1024×768** (min ==
initial), which sits *just above* that 1008 threshold — so `Auto` would in fact render
the expanded pane with labels today. `Left` is chosen not because `Auto` would hide
labels (it wouldn't at 1024), but because it removes all dependence on a breakpoint
sitting ~16 DIP from our exact width — any future min-size change or inset math could
cross it. `Left` guarantees the expanded icon+label pane + burger regardless.

### D3 — Configs is a normal tagged menu item (the gear is disabled)

Configs is a regular menu item — `NavViewItem::new("Configs").tag("config")
.icon(Symbol::Setting)`, placed last — routed like any `widget:*`/`home` tag. The
built-in Settings gear is **disabled** (`settings_visible(false)`).

**Why not the gear (runtime-corrected).** An earlier draft mapped Configs to the
built-in gear, relying on the gear's selection reporting an empty tag that we routed to
config. Runtime testing proved this **does not work**: clicking the gear fell through
to the **home view**, not config — the gear's selection does not deliver a routable tag
through reactor's `on_selection_changed`/`select_nav_item_by_tag` path (reactor's own
sample never enables the gear — `settings_visible(false)` — so this path is untested
upstream). A regular menu item routes and highlights by the exact mechanism the widget
item uses (runtime-confirmed working), so Configs becomes one. The cost is placement:
reactor exposes no pane-footer, so Configs sits as the last item in the top list rather
than a bottom-pinned gear — an acceptable trade for a config entry that actually works.

**The `""` deselection artifact.** With the gear gone, the only source of an empty tag
is **deselection**: reactor re-applies a changed `menu_items` via `menu.Clear()`, and
clearing a *currently-selected* MenuItem makes WinUI fire `SelectionChanged` with a
null item → `""`. The only `menu_items` mutator is the reload closure, reachable solely
from the config view, so the only item that can be selected during a rebuild is Configs
itself. We therefore route **`"" ⇒ config`** as well, so a widget-changing reload (the
only case that rebuilds the menu) keeps the content on config instead of bouncing to
home. (A plain reload of an unchanged widget leaves `menu_items` byte-identical → no
`Clear` → no deselection at all.) It is **not** true that a tag-less menu item would
route to config — reactor's `build_nav_view_item` defaults a missing tag to the item's
**content** string — so always set an explicit `.tag(...)`.

*Alternative considered:* keep the gear (conventional bottom placement) — rejected
because it is empirically broken here. A future reactor that wires the gear's selection
into `on_selection_changed` could revisit this.

### D4 — GitHub is NOT a nav item (dropped)

An earlier draft made GitHub a `NavViewItem` that opens the browser on selection and
then "reverts" the selection back to the prior view. Audit against reactor proved the
revert **cannot work**:

- The reconciler skips an unchanged `selected_tag` (Context), so after WinUI moves its
  internal selection to GitHub, re-passing the prior tag (which reactor already
  recorded) calls no `set_prop` → `SetSelectedItem` never fires → the pane stays
  visually stuck on GitHub. If the callback mutates no state, there is no re-render at
  all.
- Reverting to a config-active prior view is doubly broken: the prior tag would be
  `""`, which `select_nav_item_by_tag` can't select anyway.
- It would also force a new `windows-sys` `Win32_UI_Shell` feature + a `ShellExecuteW`
  helper (no URL opener exists in the repo today — the only one is the
  `HyperlinkButton` being removed), contradicting "no dependencies added".

So GitHub is **removed from navigation**. The project is still linked from the README
home view. This deletes both the reconciler-revert hazard and the new dependency.

*Alternative considered:* force the revert with a nonce/`Unset`→tag toggle to make the
prop actually change — rejected as fighting the framework for an external link that
was never a real destination; the README already carries the link.

### D5 — Remove the widget `nav` contract

Delete `nav(state)` handling end-to-end: the runtime's `nav` extraction and
validation (`button`-in-nav inline error, display-only mapping via the `Mapper::Nav`
variant), the public `NavCard` / `nav_render` surface and the `NavCard` re-export in
`widget/mod.rs`, and the **nav-only `BorderStyle` methods** — `default_card`, `over`,
and the `corner_radius()`/`padding()` accessors — plus de-`pub`ing `apply_frame`
(still called internally by `apply`). **`BorderStyle`'s five fields
(`corner_radius`/`border_thickness`/`padding`/`background`/`border_color`) all feed
the main `border` render node and MUST stay** — none are nav-only (correcting the
first-draft task). The shell always uses `w.name()` for the widget's `NavViewItem`
content. Update `widgets/counter/main.lua` to drop its `nav` function. The
`{ state, render }` contract otherwise stands.

*Why:* a `NavViewItem` renders string content only (reactor-notes), so a
widget-drawn preview cannot live in a native nav item; keeping the mechanism would be
dead code.

### D6 — Widget tag is `widget:<id>` (namespaced)

The widget's tag is `format!("widget:{}", id)` where `id` is its folder name (already
the widget identity in `widget-package`). The `widget:` prefix keeps the three tag
namespaces structurally disjoint — `"home"`, `""` (config), `"widget:*"` — so no
folder name (even one literally named `home`) can collide with a reserved tag and
misroute. Routing strips the prefix.

`LoadedWidget` has an `id: String` field but **no `id()` accessor today — it must be
added** (`pub fn id(&self) -> &str`); `name()` already exists. Display name (`name()`)
and routing key (`id()`) stay distinct (two widgets could share a name). With
single-widget loading this is low-stakes but keeps the tag stable and unique.

### D7 — A broken widget is a selectable error item

`load_first` already distinguishes an **empty notice** (no widget folder at all) from
a real **load error** (folder present, but manifest/Lua failed). Because a widget with
no nav item leaves its error unreachable (there is nothing to select to reach the
widget view), a load error is surfaced as its **own selectable nav item**:
`NavViewItem::new(<folder id>).tag("widget:<id>").icon(Symbol::Important)`. Selecting
it routes (like any `widget:*` tag) to the widget view, whose content is the error
text. The folder id is known even on failure (package discovery yields it before Lua
parsing), so it is a reliable label; the manifest display name may be unavailable (a
bad manifest is a failure mode), so we label with the id, not `name()`.

Extracting that id: `WidgetError` exposes **no `id()` accessor**, and its variants'
fields are private — its `Manifest`/`Entry`/`Lua` variants carry an `id` while
`NoWidgets` does not. So the shell pattern-matches the variant (or a new
`WidgetError::id() -> Option<&str>` helper is added): `Some(id)` → the error item;
`None` (`NoWidgets`) → the no-item case. Using `Display` for the label/tag would be
wrong (it yields the whole error message, not the id).

An **absent** widget (empty notice) still shows **no item** — it is not an error, just
nothing to host.

*Why:* `NavViewItem` has no disable flag, so a normal selectable item is the only
shape; routing the error through the existing `widget:*` tag needs no new view state.
*Alternative considered:* show the error in the config view — rejected during grilling
in favor of a dedicated, self-explanatory error item.

## Risks / Trade-offs

- **Deselection artifact routes to config** → Configs is a real `"config"` menu item,
  but reactor rebuilds a changed `menu_items` via `menu.Clear()`, which deselects the
  currently-selected item and fires `SelectionChanged` with a null item → `""`. Since
  the only `menu_items` mutator (reload) is reachable solely from the config view, the
  only item selected during a rebuild is Configs itself, so `""` is routed to config
  (content stays put). Mitigation/invariant: keep reload reachable only from the config
  view; never add a menu-mutating action reachable from Home or the widget view without
  handling deselection. (Not a tag-less-item hazard — reactor defaults a missing tag to
  the item's content, so always set an explicit `.tag(...)`.) Documented in
  reactor-notes + a code comment (tasks 2.7, 4.1).
- **Rewrite drops window plumbing** → the `use_effect` running
  `window::capture_hwnd`/`install_close_to_tray`/`hide` lives *inside* `tool_surface`
  (the function being rewritten); losing it breaks hide-to-tray and the flash-free
  launch. Mitigation: an explicit "preserve" task; keep the `use_effect` verbatim at
  the top of the new `tool_surface`, along with the `tick`/`slot`/`set_tick`
  re-render machinery.
- **Accessibility/behavior parity** → NavigationView items are real, keyboard-
  focusable controls (a strict improvement over the old tapped-`border` card, which
  reactor-notes flags as not a Button in the AT tree). Low risk.
- **Loss of the live widget nav preview** → intentional (BREAKING). The pane shows the
  name instead. Any external widget relying on `nav` silently loses the preview (no
  error; `nav` is simply ignored/removed).
- **Content inset** → the deleted `.margin(16)` is the only padding today, and no view
  pads itself (`home::view` is a bare `scroll_viewer`). Mitigation: wrap the resolved
  `right_content` once in a `~16`-DIP padded container before handing it to
  `NavigationView::new` — one application point so it can't drift (minor cosmetic: the
  home scrollbar sits inside the padding). The wrapper MUST **stretch** to fill the
  content area (`Vertical`/`HorizontalAlignment::Stretch`); a size-to-content wrapper
  would hand the `scroll_viewer` an unbounded height and the README would stop
  scrolling (reactor-notes: a `scroll_viewer` needs a bounded cell).

## Migration Plan

1. Add `LoadedWidget::id()`; strip `nav`/`NavCard`/`nav_render`/`Mapper::Nav` and the
   nav-only `BorderStyle` methods from `src/widget/` (keep all fields; drop `NavCard`
   from the `mod.rs` re-export). Update `widgets/counter/main.lua`.
2. Rewrite `tool_surface`, **preserving** the window `use_effect` and
   `tick`/`slot`/`set_tick`: build `menu_items` (Home; the widget item when loaded, or
   an `Important` error item when the folder is present but failed; nothing when
   absent; then a `Configs` item `.tag("config").icon(Symbol::Setting)` last), wrap
   `right_content` once in ~16-DIP padding, then
   `NavigationView::new(items, right_content).selected_tag(tag).on_selection_changed(…)
   .pane_display_mode(Left).pane_toggle_button_visible(true).settings_visible(false)
   .pane_title("Prismatic Tools")` (no `.header`). Replace the `View` enum with a
   `tag` `use_state` (default `"home"`); route `"home"` / `"config" | ""` / `"widget:*"`;
   delete the fills/hover/card grid and the `HyperlinkButton`.
3. `cargo check --target x86_64-pc-windows-gnu` (the Linux stub compiles **none** of
   `src/widget` or `src/shell` — it is all `cfg(windows)`), then
   `python3 scripts/winrun.py` to verify the burger, items, Home default, widget view,
   Configs (the tagged menu item + reload), the broken-widget error item, and hide-to-tray on
   the X button.
4. Update `docs/reactor-notes.md` and `CLAUDE.md`.

Rollback: revert the branch; no data/schema/migration state is involved.

## Open Questions

- (Resolved) Icons: `Symbol::Home` for Home, `Symbol::Document` for the widget,
  `Symbol::Important` for the error item — all verified to exist; confirm they compile
  under `cargo check --target x86_64-pc-windows-gnu`.
- (Resolved) `.header(...)` is **omitted** — the home/widget/config views render their
  own titles.
