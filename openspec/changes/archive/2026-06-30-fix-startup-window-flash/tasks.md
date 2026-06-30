## 1. Dependency wiring

- [x] 1.1 No new `windows-sys` feature needed — the hook, subclass, and `WM_WINDOWPOSCHANGING` APIs are all under the already-enabled `Win32_UI_WindowsAndMessaging` / `Win32_System_Threading` features. (DWM cloaking via `Win32_Graphics_Dwm` was tried and removed: `DwmSetWindowAttribute` returns `E_HANDLE` at `HCBT_CREATEWND`.)
- [x] 1.2 Confirm the Linux host stub still builds green: `cargo build` and `cargo test` (the window code is `cfg(windows)`-gated, so this stays unaffected).

## 2. Pre-paint show-suppression (src/window.rs)

- [x] 2.1 Add a subclass `wnd_proc` that, while `SUPPRESS_SHOW` is set, rewrites `WM_WINDOWPOSCHANGING` to strip `SWP_SHOWWINDOW` / add `SWP_HIDEWINDOW`, and still hides on `WM_CLOSE`; forward everything else via `CallWindowProcW`. Add a one-shot `subclass()` (guarded by `SUBCLASSED`).
- [x] 2.2 Add a one-shot `WH_CBT` thread-local hook proc: on `HCBT_CREATEWND`, match the first top-level window (`CREATESTRUCTW.hwndParent` null) whose class (`GetClassNameW`) is `WinUIDesktopWin32WindowClass`; on match, `adopt_window()` (cache HWND, set `SUPPRESS_SHOW`, subclass) and `UnhookWindowsHookEx`. Add an `HCBT_ACTIVATE`-by-title backstop. Forward all other calls via `CallNextHookEx`.
- [x] 2.3 Add `arm_startup_hook()` (installs the CBT hook via `SetWindowsHookExW(WH_CBT, .., NULL, GetCurrentThreadId())`) and call it in `window::run()` immediately before `App::new()...render()`. Defensively unhook after `render()` returns if it never fired.
- [x] 2.4 In `window::toggle()` show branch, clear `SUPPRESS_SHOW` before `ShowWindow(SW_SHOWNORMAL)` + `SetForegroundWindow`. Leave `SW_HIDE` paths unchanged.
- [x] 2.5 Keep `capture_hwnd()`'s `FindWindowW` as the documented fallback; only set `WINDOW_HWND` from it if the hook did not already cache an HWND. Update the doc comments (drop the "first-frame flash is inherent" note).

## 3. Shell startup effect (src/shell.rs)

- [x] 3.1 Update the `use_effect` block comment: the hook already claimed + suppressed the window, so `capture_hwnd()` / `install_close_to_tray()` are no-ops and `hide()` just reinforces the hidden state. Rewrite the "(A brief first-frame flash is inherent to reactor startup.)" comment accordingly, including the fallback.

## 4. Verify on Windows (only via `python3 scripts/winrun.py`)

> Build/link verified off-Windows: `cargo build` for `x86_64-pc-windows-gnu`
> (debug + release) compiles and links cleanly.

- [x] 4.1 Build + stage + run (`yarn win`); confirmed a **flash-free** launch — no window appears before tray invocation.
- [x] 4.2 Confirmed via the startup log that the hook matched the `WinUIDesktopWin32WindowClass` top-level window at `HCBT_CREATEWND` (and that DWM cloak there returns `E_HANDLE`, which is why suppression — not cloak — is the fix).
- [x] 4.3 Regression-check: tray left-click reveals the window (Mica renders correctly on first reveal), title-bar X hides to tray, re-show works, single-instance and tray "Exit" still behave as before.
- [x] 4.4 Build the release profile (`yarn win:release`) and confirm it launches flash-free (`panic = "abort"` intact).
