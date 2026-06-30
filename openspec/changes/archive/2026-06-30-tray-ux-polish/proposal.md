## Why

The bootstrap shell runs, but its window lifecycle is rough for a tray-resident app: clicking the window's close (X) button destroys/exits rather than tucking the window back to the tray, and a second launch would stack a duplicate tray icon. This change tightens those edges so the shell behaves like a proper Windows tray app.

## What Changes

- The window close affordance (title-bar **X** / `WM_CLOSE`) hides the window to the tray instead of terminating the process; only the tray **Exit** item ends the app. Implemented by subclassing the cached top-level HWND (`SetWindowLongPtrW` + `CallWindowProcW`) to intercept `WM_CLOSE` and call `ShowWindow(SW_HIDE)`.
- Single-instance guard: a named mutex (`CreateMutexW`; `ERROR_ALREADY_EXISTS`) so a second launch exits cleanly instead of stacking a duplicate tray icon.
- Verify the tray-**menu** click actually toggles the window via UI Automation against the notification area — the bootstrap change wired `toggle()` and the handler but only drove the show path through the cached HWND, never an actual menu click.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `tray-presence`: add close-to-tray (X hides, does not exit) and single-instance lifecycle requirements.

## Impact

- **Code**: `src/window.rs` (WM_CLOSE subclass + close-to-hide), `src/shell.rs` (single-instance mutex guard at startup).
- **Dependencies**: extend `windows-sys` features — add `Win32_System_Threading` (`CreateMutexW`); `SetWindowLongPtrW` / `CallWindowProcW` / `DefWindowProcW` come from the already-enabled `Win32_UI_WindowsAndMessaging`.
- **Build/run**: unchanged (`scripts/winrun.py`).
- **Out of scope**: flyout/borderless window (rung 1), Lua plugins, a second tool.
