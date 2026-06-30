## Context

After `bootstrap-tray-shell`, the shell starts hidden, toggles from the tray, and exits via the tray "Exit" item. Two lifecycle gaps remain: the window's X button is not intercepted (default WinUI behavior on close is undesirable for a tray app), and nothing prevents a second instance from stacking another tray icon. Reactor's window/AppWindow bindings are `pub(crate)`, so — as established in the bootstrap spike — window control is done through the cached top-level HWND with `windows-sys`.

## Goals / Non-Goals

**Goals:**
- Close (X) hides to tray; only "Exit" terminates.
- One running instance; a second launch exits cleanly.
- Confirm the tray-menu click toggles the window (not just the cached-HWND show path).

**Non-Goals:**
- Borderless/flyout window (rung 1), Lua plugins, a second tool.
- Intercepting close via WinRT `AppWindow.Closing` (blocked by `pub(crate)` bindings).

## Decisions

### Decision: Intercept WM_CLOSE by subclassing the HWND, not via WinRT
- Reactor does not expose the `AppWindow.Closing` event (bindings are `pub(crate)`), and reaching it via our own `windows` crate risks windows-core crate-identity mismatches. Instead, subclass the cached top-level HWND: store the original window proc with `SetWindowLongPtrW(GWLP_WNDPROC, new_proc)`, and in `new_proc` handle `WM_CLOSE` by `ShowWindow(SW_HIDE)` + returning `0` (swallow), forwarding everything else to the original proc via `CallWindowProcW`.
- **Rationale**: stays on the same `windows-sys` surface already in use; no extra runtime/ABI risk. The original proc pointer is stored in a `static AtomicIsize` (single window).
- **Alternative considered**: `SetWindowSubclass` (comctl32) — cleaner but adds a comctl32 dependency and reference-data plumbing; the raw `GWLP_WNDPROC` swap is sufficient for one window.

### Decision: Subclass after the HWND is captured
- The HWND is captured in the first-render `use_effect` (existing code). Install the subclass there, right after `capture_hwnd()`, before/after the initial hide. The window proc swap must happen on the UI thread (it does — effects run there).

### Decision: Single instance via a named mutex
- At startup, `CreateMutexW(NULL, TRUE, "Local\\PrismaticTools.SingleInstance")`. If `GetLastError() == ERROR_ALREADY_EXISTS`, exit immediately (before building the tray). Hold the mutex handle for the process lifetime.
- **Rationale**: standard, race-free, no IPC needed for the MVP. Bringing the existing instance to front on second launch is a future nicety (needs cross-process signalling), explicitly deferred.

### Decision: Verify the tray-menu toggle with UI Automation
- Drive an actual tray/notification-area menu invoke (not just the cached-HWND show path) to confirm `toggle()` runs from a real menu click. If the notification-area automation proves flaky in the WSL-interop harness, fall back to asserting the menu item's `InvokePattern` fires the handler.

## Risks / Trade-offs

- **GWLP_WNDPROC swap on a WinUI top-level window** → WinUI may re-parent or own the proc; if swallowing `WM_CLOSE` misbehaves, fall back to handling `WM_SYSCOMMAND`/`SC_CLOSE` or hooking earlier. Mitigate by forwarding all other messages untouched.
- **Mutex name collision / leftover** → use a `Local\\` session-scoped name; the handle releases on process death, so a crash won't wedge future launches.
- **Tray-menu automation flakiness** → fall back to the `InvokePattern` assertion (above).

## Open Questions

- Should a second launch *signal the running instance to show* (not just exit)? Deferred — needs a named pipe / `WM_COPYDATA`, out of scope here.
