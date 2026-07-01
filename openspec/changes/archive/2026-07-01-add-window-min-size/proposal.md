## Why

The main window can currently be resized by the user down to a near-zero size, at which point the app header (name + `Configs`/`GitHub` buttons) and the hosted tool clip and become unusable. Enforcing a minimum width and height keeps the window's contents legible and interactive at every size the user can drag to.

## What Changes

- The main window enforces a minimum inner size of **800 × 600 DIPs**: the user cannot drag the window's edges/corners smaller than this floor.
- The floor is applied through `windows-reactor`'s existing `App::inner_constraints(InnerConstraints { min_width, min_height, .. })` builder in `src/window.rs::run()` — reactor drives `OverlappedPresenter::SetPreferredMinimumWidth/Height`, so the OS clamps interactive resize. No `WM_GETMINMAXINFO` subclass is added.
- No change to startup size, flash-free launch, close-to-tray, or the tray lifecycle.

This change touches **Windows-only (`cfg(windows)`) code** only (`src/window.rs`). No change to the cross-build/staging path (`scripts/winrun.py`) or the dev-only Node layer.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `themed-window`: adds a requirement that the window enforce a minimum interactive size (800 × 600 DIPs).

## Impact

- **Code:** `src/window.rs` — `run()` gains an `.inner_constraints(..)` call on the reactor `App` builder; a small module constant holds the min dimensions.
- **Dependencies:** none added — `InnerConstraints` / `inner_constraints` already exist in the pinned `windows-reactor` (`crates/libs/reactor`); the presenter path (`IOverlappedPresenter3::SetPreferredMinimum*`) requires Windows 11 22H2+, which the app already targets.
- **Build/scripts:** none.
- **Behavior:** interactive resize below 800 × 600 is blocked; larger sizes and maximize are unaffected.

## Non-goals

- No maximum size, fixed size, or non-resizable window.
- No change to the window's initial/startup size (only the resize floor).
- No per-tool or dynamic (content-driven) minimum sizing — a single static floor for the shell window.
- No manual `WM_GETMINMAXINFO` handling in the HWND subclass.
