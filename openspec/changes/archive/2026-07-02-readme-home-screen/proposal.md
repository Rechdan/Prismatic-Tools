## Why

The app opens straight into a widget (today, the `counter` demo). A first-time
user gets no orientation — no statement of what Prismatic Tools is, how the tray
lifecycle works, or how to add a widget. That story already exists, written well,
in the repo `README.md`. Rendering the README as an in-app **home screen** gives
the window a real landing view and reuses a single source of truth for "what this
is" instead of maintaining a second copy of the same prose inside the UI.

## What Changes

- Add a **Home** view to the two-pane main region: a scrollable, formatted
  rendering of the whole project `README.md` shown in the right container.
- Add a **Home** entry to the top of the left navigation column — a plain,
  keyboard-accessible `button` (not a card) — that activates the Home view; the
  widget card and the header `Configs` action continue to select their own views.
- Make **Home the default active view** when the window opens, replacing the
  widget as the landing view. The widget and config views are unchanged and still
  reachable from the nav card and the header.
- Bundle the `README.md` into the executable at build time and render its
  **block-level** markdown onto reactor `text_block`/`vstack`/`border` elements:
  headings via the WinUI type ramp; paragraphs and list items as wrapping,
  selectable body text; fenced code blocks as monospaced text (block-level
  `.font_family`) in a distinct container; thematic rules as separators. Inline
  runs (bold, italic, inline code, links) are **flattened to readable text** —
  the reactor rendering layer applies no per-run inline styling and has no inline
  link navigation (verified against the crate; see design). Rendering is
  fault-tolerant — an unhandled construct degrades to readable text and never
  panics.
- Watch `README.md` in the dev loop's `dev:app` watcher so a README edit triggers
  a full rebuild-stage-run (the README is compiled in, so it needs a rebuild, not
  a widget-style sync).
- Record the reactor rich-text/hyperlink findings this change surfaced in
  `docs/reactor-notes.md` (the `RichTextRun` inline props are largely no-ops and
  `RichTextHyperlink` does not navigate, while block-level text styling works), so
  the next UI change does not re-derive them.

This change touches **Windows-only (`cfg(windows)`) code** (a new `src/home.rs`
module and edits to `src/shell.rs`), adds one `cfg(windows)` dependency (the
pure-Rust `pulldown-cmark` parser), makes one **dev-only** tweak (the
`package.json` `dev:app` watch list), and updates one doc (`docs/reactor-notes.md`).
It does **not** touch the cross-build/staging path (`scripts/winrun.py`) — the
README ships compiled into the exe, not staged beside it.

## Capabilities

### New Capabilities
- `home-screen`: the README-driven home view — a read-only, vertically
  scrollable, formatted rendering of the whole project README (bundled at build
  time) shown in the main region's right container, with fault-tolerant
  block-level markdown-to-UI mapping (headings, paragraphs, lists, monospaced
  fenced code, rules), selectable text, and inline runs flattened to readable
  text (links included).

### Modified Capabilities
- `main-content-layout`: the left navigation column gains a Home entry, the right
  container's set of active views expands from two (widget, config) to three
  (home, widget, config), and the default active view on open becomes the home
  view instead of the widget view.

## Impact

- **Code**: new `src/home.rs` (`cfg(windows)`) owning the bundled README constant,
  the pulldown-cmark-driven block markdown→reactor mapping (inline runs
  concatenated to text), and the scrollable home view; `src/shell.rs` gains a Home
  nav button and a three-way active-view selection (default Home); `src/main.rs`
  declares `mod home`.
- **Dependencies**: one new pure-Rust dep, `pulldown-cmark`, under
  `[target.'cfg(windows)'.dependencies]` (no C toolchain; must cross-compile for
  `x86_64-pc-windows-gnu`, verified before mapping code is written).
- **Reactor surface**: uses `scroll_viewer` (vertical) around the document; plain
  `text_block` (with the shared `.font_family` modifier for monospaced fenced
  code, `.wrap()`, `.selectable()`), the type-ramp factories
  (`title`/`subtitle`/`body_large`/`body`) for headings, and `vstack`/`hstack`/
  `border`. `RichTextBlock` is intentionally **not** used — its per-run inline
  props (italic/strikethrough/font_family/font_size) are ignored by the backend
  and `RichTextHyperlink` does not navigate, so it offers nothing over `text_block`
  for this README (which has no inline bold).
- **Docs**: `docs/reactor-notes.md` gains a note on the rich-text/hyperlink limits
  and the working block-level text styling.
- **Dev loop**: `package.json` `dev:app` adds `-w "./README.md"` so a README edit
  rebuilds. No change to `scripts/winrun.py` or the run-dir staging — the README
  is compiled in via `include_str!`, so an edit is picked up by the next build
  (which the watcher now triggers), not by the in-app Reload.
- **Specs**: adds `openspec/specs/home-screen/`; modifies
  `openspec/specs/main-content-layout/`.
