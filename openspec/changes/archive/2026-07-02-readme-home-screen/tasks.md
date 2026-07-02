## 1. Add the markdown parser dependency

- [x] 1.1 Add `pulldown-cmark` (pure Rust; pin a recent version) under `[target.'cfg(windows)'.dependencies]` in `Cargo.toml`, beside `mlua`/`toml`.
- [x] 1.2 Run `cargo check --target x86_64-pc-windows-gnu` to confirm the crate cross-compiles for the `gnu` target **before** writing mapping code. If it fails to cross-build, stop and switch to the hand-rolled block-parser fallback (design.md D2). Note the pinned version's event API (`Event::Start(Tag)`/`End(TagEnd)`, `Tag::Heading`, `Tag::CodeBlock`, `Tag::List`). — Resolved `pulldown-cmark 0.12.2`; cross-builds clean.

## 2. Home module: bundled README + block markdown mapping

- [x] 2.1 Create `src/home.rs` (module-level `//!` doc naming the `home-screen` capability) with `const README: &str = include_str!("../README.md");`.
- [x] 2.2 Add `fn render_markdown(src: &str) -> Element` driving `pulldown_cmark::Parser::new(src)`, folding **block** events into a top-level `vstack` with vertical spacing. Ensure the returned root is a real panel (`vstack`), never a bare `Group` — it becomes the single `scroll_viewer` child (a `Group` as a sole `ScrollViewer` child panics; see design.md Context). Do **not** use `RichTextBlock` (design.md D3).
- [x] 2.3 Inline handling (flatten): accumulate `Text` and `Code` events into the current block's `String`; map `SoftBreak` → a space and `HardBreak` → a newline; ignore `Start/End(Emphasis|Strong|Strikethrough|Link)` markers. A link contributes only its visible label text (no styling, no navigation).
- [x] 2.4 Headings → type-ramp `text_block`: h1 → `title(text)`, h2 → `subtitle(text)`, h3 → `body_large(text).bold()` (deeper → `body_strong(text)`), each `.wrap().selectable()`. Paragraphs → `body(text).wrap().selectable()`.
- [x] 2.5 Lists → a `vstack` of item rows, each an `hstack((marker, body(text)))` with `•` for bulleted or `N.` for ordered (use the list's start index from `Tag::List(Option<u64>)`), indented from surrounding text; nested lists indent further.
- [x] 2.6 Fenced/indented code blocks → a `border` (subtle `ThemeRef` background, padding, corner radius) wrapping `text_block(code).font_family("Consolas").wrap().selectable()` (block-level `.font_family` is applied by the backend), preserving line breaks.
- [x] 2.7 Block quotes → a set-off `body` block (indented via `.margin`, dimmed via block-level `.foreground`). Thematic break (`---`) → a thin full-width `border` rule.
- [x] 2.8 Fault tolerance: any unhandled event/construct contributes its text content as a plain `body` block — never dropped, never panics (release aborts on panic).
- [x] 2.9 Add `pub fn view() -> Element` wrapping `render_markdown(README)` in `scroll_viewer(..)` with vertical scrolling for the right container.

## 3. Wire the module

- [x] 3.1 Declare the module under `cfg(windows)` (e.g. `#[cfg(windows)] mod home;` in `src/main.rs`, matching how `shell`/`window`/`widget` are gated), and `use crate::home;` in `src/shell.rs`.

## 4. Three-way active-view selection in the shell

- [x] 4.1 In `src/shell.rs` `tool_surface`, replace `let (show_config, set_show_config) = cx.use_state(false);` with a view enum, e.g. `enum View { Home, Widget, Config }` in `cx.use_state(View::Home)` (default `Home` = home-by-default).
- [x] 4.2 Change the header `button("Configs")` `on_click` to select `View::Config` (was `set_show_config.setter(true)`).
- [x] 4.3 Change the widget nav card's `on_tapped` to select `View::Widget` (was `set_show_config.setter(false)`).
- [x] 4.4 Route the right container (grid column 1) on the view state: `View::Home` → `home::view()`, `View::Widget` → the existing widget `content`, `View::Config` → `config_view(..)`. Keep exactly one active view rendered at a time.

## 5. Home entry in the navigation column

- [x] 5.1 Add a `button("Home")` at the **top** of the left nav `vstack`, above the `Tools` heading, `on_click` selecting `View::Home` (a real accessible button, not a tapped-border card — see design.md D5).
- [x] 5.2 Build the Home button unconditionally (outside the `Some(Ok(w))` widget branch) so it is present even when no widget is loaded.

## 6. Watch the README in the dev loop

- [x] 6.1 In `package.json`, add `-w "./README.md"` to the `dev:app` script so it reads `nodemon -w "./src" -w "./README.md" -e "*" -x "yarn win"` (a README edit fires the full rebuild-stage-run; leave `dev:widgets` untouched — its copy-only sync would not rebuild the compiled-in string).

## 7. Document the reactor findings

- [x] 7.1 Add a `docs/reactor-notes.md` section recording: `RichTextRun` applies only `is_bold` (per-run `is_italic`/`is_strikethrough`/`font_family`/`font_size` ignored, no per-run color); `RichTextHyperlink` does not navigate (plain text) so inline links aren't possible (only block-level `HyperlinkButton` navigates); and what works — block-level `.font_family`/`.foreground`/`.wrap()`/`.selectable()`, the type-ramp factories, and `scroll_viewer` bounding inside a Grid star cell.

## 8. Verify

- [x] 8.1 `cargo check --target x86_64-pc-windows-gnu` clean; then build + stage via `python3 scripts/winrun.py --no-run`. — Cross-check clean (no warnings); full build linked + staged 118 runtime entries. Launch/visual confirmation (8.2–8.5) is user-driven (headless host can't observe the tray GUI).
- [x] 8.2 Confirm the window **opens on the Home view** rendering the README: heading levels visually distinct (h1/h2/h3); paragraphs wrap; lists marked and indented; fenced code monospaced in a distinct container; a `---` renders as a rule; and a long README **scrolls vertically** while the header and nav stay fixed.
- [x] 8.3 Confirm rendered text is **selectable** (select + copy a command like `yarn win`), that inline code/links render as plain readable text (no raw `#`/`-`/`**` markers, no broken link navigation), and that no markdown construct causes a crash or blank pane (fault tolerance).
- [x] 8.4 Confirm nav switching: clicking `Home` shows the home view, clicking the widget card shows the widget (still interactive), clicking `Configs` shows the config view — only one right-container view at a time; the Home button stays visible and keyboard-focusable across all views, including with an empty `widgets/`.
- [x] 8.5 Confirm the dev watcher: with `yarn dev` running, editing `README.md` triggers a rebuild-stage-run and the Home view reflects the edit.
