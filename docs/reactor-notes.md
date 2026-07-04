# windows-reactor notes

Hard-won facts about the `windows-reactor` crate and the cross-build loop, gathered while building the widget UI. Reach for these before re-deriving reactor's API or debugging a "why won't this compile / animate" problem.

## Where the crate lives (grep it)

On this dev host the git checkout is at:

```
~/.cargo/git/checkouts/windows-rs-*/*/crates/libs/reactor/     # the crate
~/.cargo/git/checkouts/windows-rs-*/*/crates/samples/reactor/  # runnable samples
```

The samples are the best usage reference (`apps/examples/solitaire.rs`, `dotsweeper.rs`, `samples/examples/{card,opacity_transition,navigation_view,border}.rs`, etc.). Grep the crate `src/` for exact signatures; grep the samples for idioms.

## Typecheck the real code — the Linux stub does NOT

`cargo build` / `cargo check` on the Linux host compile only the `main` stub; all UI code (`src/shell.rs`, `src/window.rs`, `src/widget/`) is `cfg(windows)`-gated and **is not compiled**, so it cannot catch errors there.

- **Typecheck the Windows code:** `cargo check --target x86_64-pc-windows-gnu` (compiles `shell`/`window`/`widget` without linking/staging — fast, catches type errors the stub misses).
- **Full build + link + stage:** `python3 scripts/winrun.py --no-run`.
- **Build + stage + launch:** `python3 scripts/winrun.py` (single-instance mutex: if an old instance is in the tray, the new launch exits and you keep seeing the old build — Exit the tray app first, then relaunch).

Example gotcha this catches: `ThemeRef` is `Clone` but **not `Copy`**, so any type holding one (`Brush`, `BorderStyle`) can't derive `Copy`. The stub build stays green regardless; only the cross-`check` flags it.

## Clickable widgets take STRING content only — use `on_tapped` for rich content

Reactor's clickable *widgets* — `Button`, `HyperlinkButton`, `SplitButton`, `DropDownButton`, `RepeatButton`, and `NavViewItem` — all expose `content: String` (or `Option<String>`). **None accept an `Element` child.** So a button/nav-item whose inner content is a custom UI tree (e.g. a live preview) is not expressible as those widgets.

For a **clickable container with arbitrary element content**, use the generic gesture handlers on `ElementExt` — available on *every* element (`Border`, `StackPanel`/`vstack`/`hstack`, `Grid`, `TextBlock`, …), wired through the reconciler's `set_pointer_handlers`:

```rust
fn on_tapped(self, f: impl IntoUnitCallback) -> Self          // left click/tap
fn on_right_tapped(self, f) -> Self
fn on_pointer_pressed/released/moved/entered/exited(self, f) -> Self
```

Idiom (from `solitaire.rs`): `border(vstack((..))).corner_radius(..).background(..).padding(..).on_tapped(cb)` — a styled, clickable card holding a custom element tree.

Caveat: a tapped `Border` is **not** a `Button` in the accessibility tree (no keyboard focus / Enter activation / "button" narration). Reactor's own game samples ship this pattern anyway; accept or wrap in a focusable control if AT matters.

`NavViewItem.content` is string-only too, so a widget-drawn preview can't live inside a nav item — which is exactly why the shell surfaces widgets by **name** and dropped the widget `nav` preview. But `NavigationView` *is* the shell's navigation now (it subsumes the whole two-pane layout — owns both the item list and the content pane); see the next section.

## `NavigationView` — the shell's navigation (sharp edges)

`src/shell.rs::tool_surface` is a single `NavigationView::new(menu_items, content)`: `.pane_display_mode(NavigationViewPaneDisplayMode::Left)` + `.pane_toggle_button_visible(true)` for the expanded, burger-toggled icon+label pane; `.pane_title("Prismatic Tools")`; `.settings_visible(false)` (the built-in gear is disabled — see below); `.selected_tag(tag)` + `.on_selection_changed(set_tag)` for tag-driven routing. Every destination is a real `NavViewItem::new(label).tag(id).icon(Symbol::…)` (Home, the widget, **Configs**) (icon by value; `Symbol::{Home,Document,Important,Setting}` are real consts). The whole builder chain and `NavigationView.into() → Element` are compile-verified.

Edges that bite:

- **The built-in Settings gear does NOT route through reactor.** Its selection is not delivered as a usable tag by `on_selection_changed` (`select_nav_item_by_tag` never walks it either — `backend/winui/convert.rs`), and reactor's own sample never enables it (`settings_visible(false)`). Runtime-confirmed: clicking the gear fell through to the home view, not config. **So don't use the gear as a destination** — make config a normal tagged menu item (`.tag("config")`) and disable the gear. (This is why the shell's Configs is a menu item, not the gear.)
- **The reconciler skips an *unchanged* `selected_tag`** (`reconciler/diff_helpers.rs` `diff_props`: `old == new ⇒ no set_prop`). It diffs the previous vdom against the new one, *not* the control's live selection — so a `NavigationView` is **not** a "snap selection back every render" controlled component. The control's selection is corrected only when the tag value you pass actually **changes** to a real menu-item tag. (This is why an item that opens an external browser and tries to revert its own selection — e.g. a GitHub link — can't work: re-passing the prior tag is a no-op and the pane stays stuck. GitHub was dropped for this reason.)
- **A missing item tag defaults to the item's *content* string, not `""`** (`build_nav_view_item`). So a tag-less item routes by its label. Always set an explicit `.tag(...)`.
- **Deselection artifact.** Reactor re-applies a changed `menu_items` via `menu.Clear()` + re-append. Clearing a *currently-selected* MenuItem makes WinUI fire `SelectionChanged` with a null item → the handler's `unwrap_or_default()` yields `""`. The shell routes `"" ⇒ config` and keeps its sole `menu_items` mutator (reload) reachable **only from the config view** — so the only item selected during a rebuild is Configs itself, and content stays on config. Keep that invariant: do not add a menu-mutating action reachable from Home or the widget view.

## `border(child)` + brushes

`border(child: impl Into<Element>) -> Border` wraps a single element. Props: `corner_radius(f64)`, `border_brush(impl Into<BrushBinding>)`, `border_thickness(Thickness)`; plus the shared `ElementExt` setters `background(impl Into<BrushBinding>)` and `padding(impl Into<Thickness>)` (so `.padding(10.0)` or `.padding(Thickness::uniform(10.0))`).

`BrushBinding` accepts a literal `Color` **or** a `ThemeRef`:

- `Color { pub a, r, g, b }` is `Copy`; only `Color::rgb(u8,u8,u8)` exists (no `rgba`) — build with alpha via the struct literal `Color { a, r, g, b }`.
- `ThemeRef` is theme-aware (tracks light/dark Mica) and `Clone` (not `Copy`). Useful brushes: `CardBackground`, `CardStroke`, `SubtleFill`, `LayerFill`, `ControlFill*`, `SurfaceStroke`, plus accent/text/system tokens. Prefer these over literal colors so content reads correctly on Mica in both themes. Sample: `card.rs`.

Border padding insets the child. If you layer backgrounds under padded content (e.g. a hover fill that must span the whole card), put the **padding on the content**, not on the frame border — otherwise the frame padding insets the inner grid and the fill only covers the content area.

## Animations: opacity/scale/translation only — NOT brush color

Implicit transitions animate **opacity, scale, rotation, translation** — there is no brush/background-color tween. `with_opacity_transition(Duration)` makes `.opacity(x)` changes tween between renders (sample: `opacity_transition.rs`).

To animate a color change (e.g. a hover fill), **crossfade opacity of an overlay**: stack a fill element in the same grid cell as the content and animate its opacity `0.0 ↔ 1.0`, driven by a `use_state(hovered)` toggled from `on_pointer_entered` / `on_pointer_exited`.

## Hooks / state plumbing

- `use_state<T>(initial) -> (T, SetState<T>)`. `SetState` is `Clone`; `.call(v)` sets, `.setter(v)` returns an `impl Fn() + Clone + 'static` (handy for `on_click`/`on_tapped`).
- `use_ref<T: 'static>(initial) -> HookRef<T>` where `HookRef` is `{ inner: Rc<RefCell<T>> }` and is **`Clone`** (clones the `Rc`). Clone it into a `'static` callback to mutate the ref on click (e.g. a "reload" that replaces the held value), then bump a `use_state` tick to re-render.
- `App` builder: `.inner_size(w, h)` sets the opening size; `.inner_constraints(InnerConstraints { min_width, min_height, .. })` sets the resize floor. Both are DIPs.
- Grid children with no explicit row/column default to cell (0,0) and **overlap** in z-order (later child on top) — the basis for the hover-overlay-under-content pattern.

## Rich text: only block-level styling works; `RichTextBlock` runs and hyperlinks are half-wired

Reactor exposes a `RichTextBlock` (`RichTextParagraph` → `RichTextInline::{ Run, LineBreak, Hyperlink }`, with `RichTextRun { text, is_bold, is_italic, is_strikethrough, font_family, font_size }` and `RichTextHyperlink { text, uri }`). It looks like a full inline rich-text API. **It is not, in this checkout** — the WinUI backend (`backend/winui/mod.rs`) only wires part of it:

- **A `RichTextRun` applies only `is_bold`** (→ `FontWeight 700`). `is_italic`, `is_strikethrough`, per-run `font_family`, and per-run `font_size` are **silently ignored**, and a run has **no color field** at all. So `RichTextBlock`'s only real gain over a plain `text_block` is **mixed bold within one paragraph**.
- **`RichTextHyperlink` does not navigate** — it is "rendered as plain text (no navigation support yet)". Inline links are therefore impossible; only the block-level **`HyperlinkButton`** widget (string content) actually opens a URI, and it can't sit inside a paragraph's text flow.

What **does** work for styling text, all at the **block (whole-element) level** via shared `ElementExt` modifiers on any element (`text_block`, `border`, …):

- `.font_family(name)` (e.g. `"Consolas"` for a code block — the backend `SetFontFamily`s the `TextBlock` handle), `.foreground(brush)` (a `Color` or `ThemeRef`), `.font_size(f64)`, `.bold()`/`.semibold()`/`.font_weight(u16)`, `.wrap()` (`TextWrapping::Wrap`), `.selectable()`.
- WinUI **type-ramp** factories return a pre-sized `TextBlock`: `title` (28 semibold), `subtitle` (20), `body_large` (18), `body_strong` (14 semibold), `body` (14), `caption` (12).

Practical upshot (the README home screen, `src/home.rs`): map markdown **block-by-block** to `text_block`s and flatten inline runs to text — per-run bold/italic/inline-code/link styling is not worth `RichTextBlock` unless you specifically need mixed **bold** and nothing else.

## `scroll_viewer` — vertical by default; bounds inside a Grid star cell

`scroll_viewer(child)` wraps a **single** child (`PositionalSingle`): vertical scrollbar `Auto`, horizontal `Disabled` by default (`horizontal_scroll_bar_visibility(..)` to change). It needs a **bounded height** to scroll rather than grow: inside a `vstack`/`StackPanel` (infinite height) give it `.max_height(..)`; inside a **Grid star row/column cell** the cell already bounds it, so it scrolls with no explicit height (that is how the shell's right container hosts the home view). An `Element::Group` (fragment) as the **sole** child of a `ScrollViewer`/`Border` **panics** — hand it a real panel (a `vstack`/`grid`), not a bare multi-child fragment.
