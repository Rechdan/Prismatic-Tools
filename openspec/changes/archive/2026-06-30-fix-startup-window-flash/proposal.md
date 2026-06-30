## Why

The app is supposed to launch silently into the tray with no visible window, but on every launch the WinUI window flashes on screen for a few frames before it is hidden. Reactor creates the window, activates (shows + paints) it on a high-priority dispatcher tick, and only a *later* tick runs our `use_effect` `hide()` — so the user sees a transient window. The current code even concedes this as "inherent" (`src/shell.rs`), but it is fixable and undermines the "background tray process" promise.

## What Changes

- Catch the top-level WinUI HWND **before its first paint** (rather than after the first render, as today), via a one-shot thread-local `WH_CBT` hook armed before reactor's message loop.
- Subclass that HWND at creation and, while a startup flag is set, rewrite `WM_WINDOWPOSCHANGING` to strip `SWP_SHOWWINDOW` / add `SWP_HIDEWINDOW` — so reactor's startup activation cannot make the window visible. This is the same `SW_HIDE` outcome the app already relies on, just applied at the earliest possible message instead of a dispatcher tick too late.
- The window-reveal path (tray left-click / re-show) clears the flag, so the first reveal shows normally.
- If the pre-paint capture does not fire (e.g. a future WinAppSDK class rename), an `HCBT_ACTIVATE`-by-title backstop still captures the HWND for close-to-tray, and behavior degrades to at worst today's brief flash — never worse.

This change touches **Windows-only (`cfg(windows)`) code** only — `src/window.rs` (startup hook + subclass/suppression) and `src/shell.rs` (startup effect comment). No new dependencies, and no changes to the cross-build/staging path (`scripts/winrun.py`) or the dev-only Node layer.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `tray-presence`: Strengthen the "Background tray process" requirement so that **no window is visible at any point** before the user first invokes it — explicitly forbidding the transient first-frame/startup flash, not just the steady-state window.

## Impact

- **Code:** `src/window.rs` (startup `WH_CBT` hook, HWND adopt, subclass with `WM_WINDOWPOSCHANGING` suppression + `WM_CLOSE` hide), `src/shell.rs` (startup effect comment). Both already `cfg(windows)`-gated.
- **Dependencies:** None added — all APIs are already under the `Win32_UI_WindowsAndMessaging` / `Win32_System_Threading` features. (DWM cloaking was tried and dropped: `DwmSetWindowAttribute` returns `E_HANDLE` at `HCBT_CREATEWND` — the HWND isn't valid for DWM that early — so it gave no flash protection.)
- **Build/runtime:** No effect on the cross-compile, staging, or single-instance/tray/close-to-tray behavior.
- **Tests:** None exist; verified by running the staged Windows app and observing a flash-free launch.

## Non-goals

- Not changing the tray menu, left-click toggle, close-to-tray, or single-instance behavior.
- Not changing the runtime show/hide model — it stays on `ShowWindow`; the suppression only governs the startup window before the first reveal.
- Not patching or forking `windows-reactor`, and not adding a "start hidden" option upstream.
- Not addressing any non-startup visual flash (resize, theme switch, re-show).
