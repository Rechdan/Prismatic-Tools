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

`NavigationView` doesn't help here — `NavViewItem.content` is string-only too, and it subsumes the whole two-pane layout (owns both the item list and the content pane).

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
