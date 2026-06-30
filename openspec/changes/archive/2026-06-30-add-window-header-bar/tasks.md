## 1. Header layout in `tool_surface`

- [x] 1.1 In `src/shell.rs`, wrap the `tool_surface` content in an outer `vstack` that
  carries uniform window-edge padding via `.padding(..)` (e.g. ~16px), keeping the existing
  `use_effect` window-capture call unchanged.
- [x] 1.2 Add a header `hstack` as the first child of the outer `vstack`, with `.spacing(..)`,
  containing the app name `text_block("Prismatic Tools")` (bold, ~20pt) as its left-most
  element.
- [x] 1.3 Remove the now-redundant `Prismatic Tools` heading from the demo tool content and
  keep the remaining demo elements (subtitle, click counter, button) as the tool content
  rendered below the header.
- [x] 1.4 Make the header space-between (name far left, buttons far right): use a two-column
  `Grid` (`Star(1.0)` name column + `Auto` button column) instead of a left-packing
  `hstack`, with the buttons grouped in the `Auto` column.

## 2. Header actions

- [x] 2.1 Add a `button("Configs")` to the header `hstack` with a placeholder `on_click`
  handler (no navigation, no state change).
- [x] 2.2 Add a `HyperlinkButton::new("GitHub").navigate_uri("https://github.com/Rechdan/Prismatic-Tools")`
  to the header `hstack`, after the app name, in the same row.
- [x] 2.3 Confirm `HyperlinkButton` is in scope via the existing `use windows_reactor::*;`
  glob (add an explicit import only if the glob does not re-export it). Confirmed: reactor
  re-exports it via `pub use widgets::*;` — no explicit import needed.

## 3. Build & verify

- [x] 3.1 Keep the Linux host stub green: run `cargo build` and `cargo test` (compiles the
  stub `main`; no Windows code path exercised). Both green.
- [x] 3.2 Windows-only verification — ran the app and confirmed: content is inset from the
  window border; the header shows `Prismatic Tools` on the far left with the Configs and
  GitHub buttons on the far right of the same row; the GitHub button opens
  `https://github.com/Rechdan/Prismatic-Tools` in the default browser; the Configs button is
  a no-op; the demo tool still renders and its counter still updates below the header.
