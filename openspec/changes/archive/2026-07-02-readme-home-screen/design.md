## Context

The shell's main region is a two-pane grid: a fixed 200-DIP left nav column and a
star-sized right container. Today the right container shows one of **two** views,
selected by a single `show_config: bool` `use_state` in `tool_surface`
(`src/shell.rs`): the active widget (default) or a placeholder config view. The
nav column holds a `Tools` heading plus the loaded widget as a tapped-`border`
card; the header holds the app name plus `Configs` and `GitHub` buttons.

**Reactor rendering capabilities — verified against the crate checkout and
samples** (two grilling passes corrected earlier guesses; this is the confirmed
reality):

- `text_block(..)` supports `.font_size`, `.bold`/`.semibold`/`.font_weight`,
  `.wrap()`, `.selectable()`, and — via shared `ElementExt` modifiers —
  `.font_family(name)` and `.foreground(brush)` **at the block level** (backend
  `SetFontFamily`/`Foreground` on the `TextBlock` handle). A WinUI **type ramp** of
  factories exists: `title` (28 semibold), `subtitle` (20 semibold), `body_large`
  (18), `body_strong` (14 semibold), `body` (14), `caption` (12).
- `RichTextBlock`/`RichTextParagraph`/`RichTextRun`/`RichTextHyperlink` exist, but
  the WinUI backend (`backend/winui/mod.rs`) applies **only `RichTextRun.is_bold`**
  per run — `is_italic`, `is_strikethrough`, `font_family`, and `font_size` on a
  run are silently ignored, `RichTextRun` has no color field, and
  `RichTextHyperlink` is "rendered as plain text (no navigation support yet)". So
  `RichTextBlock`'s only real advantage over `text_block` is **mixed bold within
  one paragraph**, and inline links cannot navigate at all.
- `scroll_viewer(child)` wraps a single child (`PositionalSingle`), vertical bar
  `Auto`, horizontal `Disabled` by default. Samples bound it with `.max_height`
  inside a `vstack`; inside a **Grid star cell** it is bounded by the cell (our
  right container is a Grid star column), so it scrolls without an explicit height.
  An `Element::Group` (fragment) as the sole child of `ScrollViewer`/`Border`
  panics — the child must be a real panel (a `vstack`), which our mapper produces.
- `HyperlinkButton` (a block-level *button* widget, string content) *does*
  navigate — but it cannot be embedded inline in a paragraph's text flow.

The prose we want on the home screen already exists as the repo `README.md`. Its
markdown, measured: **11 h2 + 2 h3 + 2 h1** headings, **0 inline bold**, **0
italic**, **95 inline-code spans**, **0 tables**, **0 block quotes**, **0 images**.

## Goals / Non-Goals

**Goals:**
- A Home view that renders the whole project README as a readable, formatted,
  scrollable, selectable page in the right container.
- Correct **block-level** formatting: heading hierarchy, paragraphs, marked lists,
  monospaced fenced code blocks, thematic rules.
- Home reachable from an accessible Home button in the nav column, shown by
  default on open.
- Reuse the README as the single source of truth for "what this is."
- Fault-tolerant rendering: no panic; unhandled constructs degrade to text.
- Record the reactor rich-text/hyperlink limits in `docs/reactor-notes.md`.

**Non-Goals:**
- **Inline run styling.** Bold/italic/inline-code/strikethrough within a line are
  flattened to plain text — the reactor backend applies only `is_bold` per run
  (ignored for the rest), so per-run styling is not meaningfully expressible, and
  the README has no inline bold to preserve anyway. Fenced code blocks *are*
  monospaced (block-level `.font_family`); inline code is not.
- **Clickable / navigating links.** Reactor has no inline link navigation
  (`RichTextHyperlink` renders as plain text), so every link — relative or
  absolute — renders as its plain label text. (Rewriting relative → GitHub-blob
  URLs, or rendering standalone-link paragraphs as `HyperlinkButton`, were both
  rejected: fragile and/or inapplicable, as all README links are inline + relative.)
- **Curated / filtered Home content.** The whole README renders (developer sections
  included); curation is deferred until there is a non-developer audience.
- **Images and table layout.** Degrade to readable text. (The README has neither.)
- **Live in-app Reload of the README.** It is bundled (`include_str!`); a rebuild
  picks up edits and the dev watcher triggers that rebuild — but the config view's
  Reload (which reloads widgets) does not refresh it.
- **De-duplicating the app title.** The header shows `Prismatic Tools` and the
  README's `# Prismatic Tools` H1 renders below it; the redundancy is accepted.

## Decisions

### D1 — README source: bundle at build via `include_str!`, and watch it in dev

Compile the README into the exe: `include_str!("../README.md")` from
`cfg(windows)` `src/home.rs`. The path resolves at compile time on the Linux host,
so no cross-build/staging change is needed and there is no runtime file dependency
(no missing-file/UNC failure mode). The `include_str!` edge makes a normal
`cargo`/`winrun` build recompile `home.rs` when the README changes.

To make edits visible in the dev loop, add `-w "./README.md"` to the **`dev:app`**
nodemon watcher in `package.json` (`nodemon -w "./src" -w "./README.md" -e "*" -x
"yarn win"`). A README edit then fires the full build-stage-run. It belongs in
`dev:app`, **not** `dev:widgets` — the latter is a copy-only widget sync that would
not rebuild the compiled-in string.

Rejected alternative: staging `README.md` beside the exe (like `widgets/`) for
Reload-ability — more surface (`stage()` + a watcher + a missing-file fallback) for
little gain on the app's own doc.

### D2 — Parser: `pulldown-cmark`, block events only

Add `pulldown-cmark` (pure Rust, CommonMark, no C toolchain / build-script
compiler) under `[target.'cfg(windows)'.dependencies]`; pin a recent version and
match its event API (0.9 vs 0.10+ differ — `Event::Start(Tag)` / `End(TagEnd)`,
`Tag::Heading { level, .. }`, `Tag::CodeBlock(CodeBlockKind)`, `Tag::List(Option<u64>)`).
**Verify cross-compilation first** (`cargo check --target x86_64-pc-windows-gnu`);
hand-rolled block parsing is the fallback only if it won't cross-build.

Even though inline is flattened (D3), a real parser still earns its place: **block**
structure (fenced-vs-indented code, nested/ordered lists, setext vs ATX headings,
lazy paragraph continuation, thematic breaks) is the fiddly part. Inline handling
reduces to concatenating `Text` + `Code` events (and mapping `SoftBreak`→space,
`HardBreak`→newline) into the current block's string; `Emphasis`/`Strong`/`Link`
start/end markers are ignored.

### D3 — Markdown → reactor mapping (block-level, `text_block`-based)

`render_markdown(&str) -> Element` folds the event stream into a top-level `vstack`
of block elements (vertical spacing between blocks). That `vstack` is the single
`scroll_viewer` child (never a bare `Group` — see Context). **`RichTextBlock` is
not used** (Context: it buys nothing here). Each text-bearing block accumulates its
inline `Text`/`Code` into one `String`.

- **Heading(level)** → a type-ramp `text_block`: h1 → `title`, h2 → `subtitle`,
  h3 → `body_large`.bold() (deeper → `body_strong`), `.wrap().selectable()`.
- **Paragraph** → `body(text).wrap().selectable()`.
- **List** → a `vstack` of item rows; each item an `hstack((marker, body(text)))`
  with `•` (bulleted) or `N.` (ordered, using the list's start index) indented from
  surrounding text; nested lists indent further. Item text is the flattened string.
- **Fenced/indented code block** → a `border` (subtle `ThemeRef` background,
  padding, corner radius) wrapping `text_block(code).font_family("Consolas")`
  (block-level `.font_family` *is* applied; Consolas ships on all Windows, Cascadia
  Mono is the Win11 alt) `.wrap().selectable()`, preserving line breaks (per Q6,
  wrap rather than nest a horizontal scroller).
- **Block quote** → a set-off `body` block (indented via `.margin`, dimmed via
  block-level `.foreground`), flattened text.
- **Thematic break (`---`)** → a thin full-width `border` rule.
- **Links** → the link's visible label text is emitted into the block string like
  any other text (no styling, no navigation — Non-Goals).

**Fault tolerance:** any event/construct the mapper does not specifically handle
contributes its text content as a plain `body` block — nothing is dropped silently
and nothing panics (release aborts on panic).

`pub fn view() -> Element` wraps `render_markdown(README)` in `scroll_viewer(..)`
(vertical) for the right container. If the Grid star cell does not bound it in
practice (unexpected), fall back to an explicit height constraint.

### D4 — View state: three-way selection replacing the `show_config` bool

Replace `show_config: bool` in `tool_surface` with a small `enum View { Home,
Widget, Config }` held in `cx.use_state(View::Home)` (default `Home` = home by
default). Route the right container on it: `Home` → `home::view()`, `Widget` → the
widget `content`, `Config` → `config_view(..)`. Rewire selectors: header `Configs`
→ `View::Config`; widget nav card `on_tapped` → `View::Widget`; new Home button →
`View::Home`. The widget card's `hovered` state and hover crossfade are unchanged.

### D5 — Home nav entry: a plain accessible button (Q9-B)

Add `button("Home")` at the **top** of the left nav `vstack`, above the `Tools`
heading, `on_click` selecting `View::Home`. A real `button` is keyboard-focusable
and narrated as a button (the tapped-`border` card is not — see
`reactor-notes.md`), which is the right tradeoff for a primary nav action; the mild
visual mismatch with the widget card below is accepted. Built **unconditionally**
(outside the `Some(Ok(w))` widget branch) so it is present even when no widget is
loaded.

### D6 — Document the reactor findings (Q11)

Add a `docs/reactor-notes.md` section recording, so the next UI change does not
re-derive them: (1) `RichTextRun` applies only `is_bold` — `is_italic`/
`is_strikethrough`/`font_family`/`font_size` per-run are ignored, no per-run color;
(2) `RichTextHyperlink` does not navigate (plain text) — inline links are not
possible, only the block-level `HyperlinkButton` navigates; (3) what works:
block-level `.font_family`/`.foreground`/`.wrap()`/`.selectable()`, the type-ramp
factories, and `scroll_viewer` bounding inside a Grid star cell.

## Risks / Trade-offs

- **`pulldown-cmark` cross-compilation** — low risk (pure Rust), unverified for
  `gnu` here. Mitigation: cross-`check` as the first task; hand-rolled block-parser
  fallback (D2).
- **`scroll_viewer` bounding** — relies on the Grid star cell giving a finite
  height (standard WinUI); if it instead grows unbounded, Home won't scroll.
  Mitigation: verify scrolling in-app (task 7); fall back to an explicit height
  constraint (D3).
- **Inline code (95 spans) renders as plain prose** — no inline monospace exists,
  so `yarn win`/paths/flags read as ordinary words. Accepted (Q10-A) for the
  simplicity of dropping `RichTextBlock`; fenced code blocks (the multi-line stuff)
  *are* monospaced. Revisit with an `is_bold` "bold-as-code" hack if it grates.
- **All links become plain text** — the README's `LICENSE`/`reactor-notes`
  pointers don't navigate on Home. Forced by reactor (no inline nav); accepted
  while the audience is developers (who have the repo).
- **Whole-README content on Home** — end users would see build/WSL/cargo/gotcha
  prose. Accepted for the current developer audience; curation deferred (Q1-C).
