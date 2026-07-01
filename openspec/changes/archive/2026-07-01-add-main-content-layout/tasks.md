## 1. Expose the widget's manifest name

- [x] 1.1 In `src/widget/runtime.rs`, add a `name: String` field to `LoadedWidget` and populate it from `source.manifest.name` in `load()`.
- [x] 1.2 Add `pub fn name(&self) -> &str` to `LoadedWidget` returning the stored name.

## 2. Body layout: header + main region

- [x] 2.1 In `src/shell.rs` `tool_surface`, replace the outer `vstack((header, content)).spacing(12).padding(16)` with a two-row `grid`, placing the header in row 0 and the main region in row 1.
- [x] 2.2 Set the body grid to `.rows([GridLength::Auto, GridLength::Star(1.0)])` (header sizes to content, main fills the rest), inset it with `.margin(16.0)` (WinUI `Grid` has no Padding, so the old outer padding becomes a body margin), and use `.row_spacing(12.0)` in place of the old vstack spacing.
- [x] 2.3 Add `.grid_row(0)` to the existing header grid and `.grid_row(1)` to the main region so each lands in the correct row.

## 3. View-mode state and Configs toggle

- [x] 3.1 Add `let (show_config, set_show_config) = cx.use_state(false);` near the existing `tick` state at the top of `tool_surface`.
- [x] 3.2 Change the header `button("Configs")` `on_click` from the no-op to `set_show_config.setter(true)` (activate the config view — an explicit select, not a toggle).

## 4. Persistent two-pane main region

- [x] 4.1 Build the main region as a `grid` with the left nav in `.grid_column(0)` and the right container in `.grid_column(1)`.
- [x] 4.2 Set the main-region grid to `.columns([GridLength::Pixel(200.0), GridLength::Star(1.0)])` with `.column_spacing(12.0)` (fixed 200 DIP nav, flexible right container). This grid is always rendered.
- [x] 4.3 Fill the right container (column 1) with `if show_config { config_view() } else { content }`, where `content` is the existing widget load/render/error `Element`.

## 5. Left navigation column

- [x] 5.1 Render the left nav (column 0) as a `vstack` with a "Tools" heading.
- [x] 5.2 When the load slot is `Some(Ok(w))`, add a clickable `button(w.name())` beneath the heading whose `on_click` is `set_show_config.setter(false)` (activates the widget view); for `Err`/absent, render the heading only (no widget entry).

## 6. Config placeholder view

- [x] 6.1 Add a `config_view` helper (or inline expression) returning placeholder content (e.g. a bold "Configs" heading + a "coming soon" line) that reads/writes no settings.
- [x] 6.2 Confirm switching to the config view replaces only the right container's content while the left nav stays visible (only one right-container content at a time).

## 7. Verify

- [x] 7.1 Build the host stub for a quick compile check (`cargo build`), then build + run the real app via `python3 scripts/winrun.py`.
- [x] 7.2 Confirm: header pinned on top with buttons right-aligned; main fills the rest and tracks resizing; the left column holds a fixed 200 DIP width and shows the widget name (`Counter`), while the right container (with the widget) fills the remaining width.
- [x] 7.3 Confirm: clicking Configs activates the config placeholder in the right container only (nav stays visible); clicking the widget's nav entry returns to the widget view; the widget's interactive control still updates.
