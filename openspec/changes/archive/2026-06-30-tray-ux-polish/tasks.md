## 1. Close-to-tray (intercept WM_CLOSE)

- [x] 1.1 Add the Win32 imports to `window.rs`: `SetWindowLongPtrW`, `CallWindowProcW`, `GWLP_WNDPROC`, `WM_CLOSE`, `WPARAM`/`LPARAM`/`LRESULT` types
- [x] 1.2 Add a `static ORIG_WNDPROC: AtomicIsize` and an `extern "system"` `wnd_proc` that hides on `WM_CLOSE` (`ShowWindow(SW_HIDE)`, return 0) and forwards everything else via `CallWindowProcW`
- [x] 1.3 Add `window::install_close_to_tray()` that swaps `GWLP_WNDPROC` on the cached HWND and stores the original proc
- [x] 1.4 Call `install_close_to_tray()` from the first-render `use_effect` in `shell.rs` (right after `capture_hwnd()`)

## 2. Single-instance guard

- [x] 2.1 Add `Win32_System_Threading` to the `windows-sys` features in `Cargo.toml`
- [x] 2.2 At the top of `shell::run()`, `CreateMutexW` with a `Local\\PrismaticTools.SingleInstance` name; if `GetLastError() == ERROR_ALREADY_EXISTS`, return early (exit) before building the tray
- [x] 2.3 Hold the mutex handle for the process lifetime (leak/keep in scope)

## 3. Build & verify

- [x] 3.1 `python3 scripts/winrun.py` builds (gnu) + stages + runs; confirm Linux host stub build stays green
- [x] 3.2 Verify close-to-tray: show the window, click X, confirm the window hides and the process keeps running (re-show works)
- [x] 3.3 Verify single instance: launch twice; confirm the second process exits and only one tray icon exists
- [x] 3.4 Verify the tray-menu click toggles the window via UI Automation (fallback: assert the menu item's `InvokePattern` fires `toggle()`)
- [x] 3.5 Confirm "Exit" still terminates cleanly (tray icon removed)
