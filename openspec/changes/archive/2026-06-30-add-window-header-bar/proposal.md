## Why

The window currently opens straight onto the demo tool with no app-level chrome: the
app name is just the demo tool's own heading, content sits flush against the window
edges, and there is no entry point for app settings or for finding the project online.
A persistent header row gives the app an identity that is independent of whichever tool
is hosted, and reserves the obvious places for a future config surface and a link out to
the repo.

## What Changes

- Add a persistent **header row** at the top of the window content, above the hosted tool
  surface:
  - The app/repo name (`Prismatic Tools`) on the **left** of the row.
  - A **Configs** button in the same row (placeholder action for now — opens nothing yet;
    wired to a real config surface in a later change).
  - A **GitHub** button in the same row that opens the project's GitHub page
    (`https://github.com/Rechdan/Prismatic-Tools`) in the default browser.
- Add **padding between the window border and its content** so the header and the tool
  surface no longer sit flush against the window edges.
- The hosted tool (the existing demo tool) continues to render **below** the header; its
  own redundant `Prismatic Tools` heading is dropped now that the header owns the app name.

This is purely a UI/layout change to the Windows-only tool surface in `src/shell.rs`
(`cfg(windows)` code). It does not touch the cross-build/staging path (`scripts/winrun.py`)
or the dev-only Node layer. The GitHub button uses Reactor's `HyperlinkButton` /
`navigate_uri`, so no new crate dependency is required.

## Capabilities

### New Capabilities
- `app-header`: A persistent header row, framed by window-edge padding, that carries the
  app name on the left and an inline action group (Configs + GitHub-link) on the right of
  the same row — independent of whichever tool is hosted below it.

### Modified Capabilities
<!-- No spec-level requirement of tool-surface changes: the host still renders exactly one
     tool in the content area. The demo's heading text is implementation detail, not a
     spec requirement, so tool-surface is left unchanged. -->

## Impact

- **Code:** `src/shell.rs` `tool_surface` — restructured to render the header row plus the
  tool content inside a padded container. Windows-only (`cfg(windows)`); the Linux host
  stub is unaffected.
- **APIs / deps:** Uses `windows-reactor` primitives already available (`hstack`/`vstack`,
  `text_block`, `button`, `HyperlinkButton::navigate_uri`, `.padding(..)`). No new
  dependency.
- **Specs:** new `openspec/specs/app-header/spec.md` capability.
- **Verification:** Windows-only behavior — confirm via `python3 scripts/winrun.py`. No
  automated tests exist; `cargo build`/`cargo test` only checks the Linux stub still
  compiles.
- **Non-goals (see proposal Non-goals below).**

## Non-goals

- Implementing an actual configuration UI/flyout behind the **Configs** button — this
  change only adds the button and a placeholder click handler.
- Persisting or reading any settings/state.
- Reworking the tray, window lifecycle, single-instance, or flash-free startup behavior.
- A plugin/tool-switching mechanism — the header sits above whatever single tool the host
  already renders.
- Custom title-bar / caption-button theming beyond the in-content header row.
