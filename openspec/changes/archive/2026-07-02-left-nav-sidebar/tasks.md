## 1. Restructure the navigation column (`src/shell.rs`)

- [x] 1.1 Build the pinned title (nav row 0): `text_block("Prismatic Tools").font_size(20.0).bold()` as a plain non-clickable label, dropping the header-only `grid_column`/`vertical_alignment(Center)` props; give it a little bottom spacing so it reads as a header.
- [x] 1.2 Build the scrollable tool list (nav row 1): a `vstack` in existing order — the Home entry (see 3.1), `text_block("Tools").bold()`, then the widget card (push only when present), spacing `8.0` — wrapped in a vertical `scroll_viewer`. Keep the `vstack` (a real panel) as the scroll_viewer's sole child, never a bare fragment.
- [x] 1.3 Build the pinned bottom group (nav row 2): a `vstack` of the Configs entry (see 3.1) above the `GitHub` `HyperlinkButton` (`navigate_uri("https://github.com/Rechdan/Prismatic-Tools")`), both full-width stretch, spacing `8.0`.
- [x] 1.4 Compose the nav column as a `Grid` with rows `[GridLength::Auto, GridLength::Star(1.0), GridLength::Auto]`: title in `grid_row(0)`, scroll region in `grid_row(1)`, bottom group in `grid_row(2)`; set `grid_column(0)` on the nav grid for the outer two-pane grid.

## 2. Drop the header and re-parent the body (`src/shell.rs`)

- [x] 2.1 Delete the `header` grid (app name + Configs/GitHub two-column grid) and its construction.
- [x] 2.2 Remove the outer two-row body grid (`grid((header.grid_row(0), main.grid_row(1)))` with `rows([Auto, Star])`); return the two-pane grid (`columns([Pixel(200), Star]`) directly as the body, keeping `.margin(16.0)` and `.column_spacing(12.0)`.
- [x] 2.3 Confirm the right container `match view { Home | Widget | Config }` and its `grid_column(1)` are unchanged; the `View` enum, its `use_state`, and `config_view(..)` keep the same signatures.

## 3. Active-view selection highlight (`src/shell.rs` only; no `src/widget/` change)

- [x] 3.1 Give Home and Configs a `view`-driven selection fill while keeping them real `button`s (do NOT use `ToggleButton` — its controlled `is_checked` desyncs on re-click, see design.md). Render each `button(label).subtle().on_click(set_view.setter(View::X)).horizontal_alignment(Stretch)` layered over a `selection_fill` border in a `grid((selection_fill, button))`; `.subtle()` keeps the button transparent at rest so the fill shows through. The `selection_fill` opacity = `1.0` when `view == View::X` else `0.0`, with `with_opacity_transition`. Because the fill is declarative from `view`, re-clicking the active entry (`set_view` same value) is a harmless no-op and the fill stays lit.
- [x] 3.2 Add the same persistent selection-fill overlay to the widget card: extend the card's `grid((hover_fill, content))` to `grid((selection_fill, hover_fill, content))`, where `selection_fill` is a soft `ThemeRef` border (e.g. `ControlFill`, distinct from the hover's `SubtleFill`), `corner_radius` matching the card, `opacity` = `1.0` when `view == View::Widget` else `0.0`, with `with_opacity_transition` (selected + hover compose).
- [x] 3.3 Confirm GitHub stays a plain `HyperlinkButton` with no selected state, and the config-view Reload control stays a plain `button` (an action, not a view — never highlighted).

## 4. Delete the removed capability's spec folder

- [x] 4.1 Delete `openspec/specs/app-header/` (all four requirements are removed by this change; no requirement survives, so the capability folder is removed rather than left empty). This is done at apply time alongside the code change.

## 5. Verify

- [x] 5.1 Keep the Linux host stub green: `cargo build` (and `cargo test`) still compiles the stub.
- [x] 5.2 Typecheck the real UI: `cargo check --target x86_64-pc-windows-gnu` passes (the Linux stub does not compile `src/shell.rs`).
- [x] 5.3 Windows run-check via `python3 scripts/winrun.py` (or `yarn win`): no header row; the sidebar shows `Prismatic Tools` pinned on top, then Home + `Tools` + widget card in the middle, and Configs + GitHub pinned at the bottom; the right container fills the rest.
- [x] 5.4 Windows scroll check (Windows-only): with the short default list no scrollbar shows; resizing the window shorter keeps the title pinned top and Configs + GitHub pinned bottom while the middle region scrolls if needed; the widget card's hover highlight and tap-to-activate still work inside the scroll region.
- [x] 5.5 Windows selection check (Windows-only): exactly one of Home / widget card / Configs is highlighted at a time, matching the active view; switching views moves the highlight; **re-clicking the already-active entry keeps it highlighted** (the desync-regression guard); hovering the selected card shows hover + selection composed; GitHub never highlights; the Home/Configs `.subtle()` fill and the card's selection overlay read as the same soft-selected family (tune the `selection_fill` brush if not).
- [x] 5.6 Windows behavior check (Windows-only): Configs selects the config view (does not toggle back to the widget on a second click), Home selects home, the widget card selects the widget, GitHub opens the repo page; the `Prismatic Tools` title is a plain label (clicking it does nothing).
