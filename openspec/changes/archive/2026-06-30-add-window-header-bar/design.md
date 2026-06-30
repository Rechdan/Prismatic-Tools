## Context

`tool_surface` in `src/shell.rs` currently renders a flat `vstack` directly into the
window content: an app-name heading (`Prismatic Tools`, 22pt bold), a demo subtitle, a
click counter, and a button — all flush against the Mica window edges. There is no
app-level chrome separating the app's identity from the hosted tool, and no entry point
for settings or for the project's online home.

This change adds a persistent **header row** (app name on the left; Configs + GitHub
buttons in the same row) and **window-edge padding**, framing whatever single tool the
host renders below. It is confined to the Windows-only (`cfg(windows)`) `tool_surface`
function; the window lifecycle in `src/window.rs` (CBT hook, HWND subclass, tray toggle,
flash-free startup) is untouched. No new crate dependency is introduced — all primitives
already exist in `windows-reactor`.

Relevant `windows-reactor` API (confirmed in the pinned git checkout):
- `vstack(..)` / `hstack(..)` → `StackPanel` with `.spacing(f64)`.
- `ElementExt` modifiers on every builder: `.padding(impl Into<Thickness>)`,
  `.horizontal_alignment(HorizontalAlignment)`, `.width(f64)`, etc.; `Thickness: From<f64>`
  for uniform insets plus a `Thickness::new(..)` constructor for per-side values.
- `text_block(..)` with `.font_size(f64).bold()`.
- `button(content).on_click(..)`.
- `HyperlinkButton::new(content).navigate_uri(uri)` — opens `uri` in the default browser
  via the WinUI HyperlinkButton; no shell-out / `windows-sys` call needed.

## Goals / Non-Goals

**Goals:**
- A persistent header row above the hosted tool: app name left, Configs + GitHub buttons in
  the same row.
- Uniform padding between the window border and all content.
- The GitHub button opens `https://github.com/Rechdan/Prismatic-Tools` in the default
  browser.
- Keep the change inside `tool_surface`; no `src/window.rs` or build-path changes.

**Non-Goals:**
- A real configuration surface behind **Configs** (placeholder click only).
- Reading/writing any settings or persisted state.
- Tray / window-lifecycle / startup-flash changes.
- Plugin or tool-switching mechanics.
- Custom caption-bar / title-bar theming beyond the in-content header.

## Decisions

### Layout: padded outer `vstack` with a header `hstack`
Wrap the content in a single outer `vstack` carrying the window-edge padding, with the
header row as its first child and the existing tool content as the following children:

```
vstack( [ header_row, ...tool_content ] )
    .spacing(..)
    .padding(<window-edge inset>)

header_row = grid( [ text_block("Prismatic Tools").font_size(20).bold()
                         .grid_column(0).vertical_alignment(Center),
                     hstack( [ button("Configs").on_click(<placeholder>),
                               HyperlinkButton::new("GitHub")
                                   .navigate_uri("https://github.com/Rechdan/Prismatic-Tools") ] )
                         .spacing(8).grid_column(1) ] )
    .columns([Star(1.0), Auto]).column_spacing(8)
```

- *Why an outer `vstack` for padding:* `.padding(..)` on the outer container insets every
  child uniformly from the window border — satisfies the "padding from the window's border"
  requirement in one place, applying to both the header and the tool surface.
- *Alternative considered — `border(..)` wrapper:* Reactor has a `Border` widget that could
  carry the padding; rejected as unnecessary indirection when `.padding()` on the existing
  `vstack` does the same job.

### "Same row", space-between, via a two-column `Grid`
The header is a `Grid` with two columns — a star-sized first column (`GridLength::Star(1.0)`)
holding the app name, and an auto-sized second column (`GridLength::Auto`) holding the
button group `hstack`. The star column absorbs the row's free space, so the name sits at the
far-left edge and the buttons hug the far-right edge (space-between). The name is centered
vertically against the taller buttons.

- *Why `Grid` over `hstack`:* a plain `hstack` left-packs all children, so the buttons would
  sit immediately after the name rather than at the right edge. The star/auto `Grid` is the
  clean way to get the name-left / buttons-right split.
- *Alternative considered — stretched spacer in an `hstack`:* an empty element between name
  and buttons that grows to fill; rejected as less direct than a two-column grid and harder
  to keep robust across resizes.

### GitHub link via `HyperlinkButton::navigate_uri`
Use Reactor's `HyperlinkButton` with `navigate_uri` so WinUI opens the URL in the default
browser — no `ShellExecute`/`windows-sys` shell-out, no new dependency, and it renders as a
button in the row.

- *Alternative considered — `button("GitHub").on_click(|| ShellExecuteW(..))`:* would pull
  in another `windows-sys` feature and hand-rolled URL launching; rejected since
  `HyperlinkButton` already does exactly this.

### Configs button is a placeholder
`button("Configs").on_click(..)` with a no-op (or trivial) handler. No config surface
exists yet; wiring one is explicitly a later change (see Non-Goals). Keeping the button now
reserves its place in the header so the later change is purely additive.

### Keep the demo tool below the header
The existing counter/button demo stays as the hosted-tool content beneath the header, with
its redundant `Prismatic Tools` heading removed (the header now owns the app name). This
preserves the `tool-surface` "interactivity proves the render/state loop" behavior while
the header frames it.

## Risks / Trade-offs

- **Exact `HyperlinkButton` / `navigate_uri` rendering and naming in the pinned reactor
  commit** → Confirmed present in the checked-out source
  (`crates/libs/reactor/src/widgets/hyperlink_button.rs`); if a future reactor bump changes
  the API, the fallback is a `button` + `ShellExecuteW`. Verify at build time via
  `python3 scripts/winrun.py`.
- **Padding/spacing are visual judgments** → Settled against the Mica backdrop during the
  Windows run: 16px uniform window-edge padding, 12px outer column spacing, 8px header
  spacing; not load-bearing for correctness.
- **No automated tests on Windows** → Header behavior (name, two buttons, browser-open,
  padding) is verifiable only by running the app under `scripts/winrun.py`; `cargo
  build`/`cargo test` only proves the Linux stub still compiles.

## Open Questions

- ~~Should the button group be right-aligned (Grid/spacer) now?~~ **Resolved:** the buttons
  are pinned to the far right via a two-column star/auto `Grid` (space-between header).
- ~~Final padding/spacing values~~ **Resolved during the Windows run:** 16px window-edge
  padding, 12px outer column spacing, 8px header button/column spacing.
