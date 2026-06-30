## Context

`windows-reactor` owns window creation and activation; the app cannot tell reactor to "start hidden." The launch sequence (verified in the cached crate source, rev `a1e9fce`):

1. `App::render()` → `Application::Start` → `OnLaunched` builds `ReactorHost` (`host.rs`). `create_window()` does `Window::new()` (the Win32 HWND already exists here) and sets the title *afterward* via `window.SetTitle(...)`.
2. `OnLaunched` calls `host.activate()`, which enqueues a **High-priority** `DispatcherQueueHandler` that runs `window.Activate()` — this is what shows + paints the window.
3. On a *later* dispatcher tick, our component's `use_effect` runs `capture_hwnd()` + `hide()` (`src/shell.rs:49`).

So the window is visibly shown at step 2 and only hidden at step 3 — the flash. The app otherwise works: it does end up hidden, and tray reveal/close-to-tray/single-instance are all correct. The current code comments this off as "inherent to reactor startup" (`src/shell.rs:46-48`); this design removes it without forking reactor.

Constraints:
- Reactor's `AppWindow` bindings are `pub(crate)`; visibility is already driven through the raw HWND with `windows-sys` (`src/window.rs`).
- Windows 11 target, cross-built `x86_64-pc-windows-gnu` from WSL. No new crates; only `windows-sys` features may be added.
- The title is **not** present in the window's `CREATESTRUCT` (set after creation), so it cannot be used to identify the window at creation time.
- Single UI thread: reactor pumps the WinUI window and the tray's hidden window on one thread (`src/shell.rs:66-71`).

## Goals / Non-Goals

**Goals:**
- Zero visible window — no flash — from process start until the user first invokes it from the tray.
- Keep the existing show/hide/toggle, close-to-tray, single-instance, and tray-menu behavior unchanged.
- No reactor fork, no new crate. Fail safe: if the early intercept misses, fall back to today's behavior (a brief flash), never worse.

**Non-Goals:**
- Changing the runtime show/hide model. The suppression only governs the pre-reveal startup window; runtime show/hide stays on `ShowWindow`.
- Eliminating any non-startup flash (resize, theme switch, re-show).
- Upstreaming a "start hidden" option to `windows-reactor`.

## Decisions

### Decision 1: Intercept the HWND *before first paint* with a one-shot `WH_CBT` hook

Arm a thread-local CBT hook (`SetWindowsHookExW(WH_CBT, proc, NULL, GetCurrentThreadId())`) in `src/window.rs` immediately **before** `App::render()` is called. On `HCBT_CREATEWND` for the main window, cache the HWND, cloak it (Decision 2), and immediately `UnhookWindowsHookEx` (one-shot). The hook runs synchronously, in-thread, at window creation — strictly before step 2's `Activate()`/paint — which is the only point early enough to prevent the flash.

**Window identification** (title is unavailable at create time): accept the first window whose `CREATESTRUCTW.hwndParent` is null (top-level) **and** whose class name (`GetClassNameW`) starts with `WinUIDesktop` — the WinAppSDK main-window class (`WinUIDesktopWin32WindowClass`). The hook is armed right before reactor creates exactly this window, and the tray's hidden message window already exists from `shell::run()` before `window::run()`, so the first matching top-level window is ours. One-shot unhook prevents matching later windows.

*Alternatives considered:*
- *Post-activate hide (today):* runs a tick too late → the flash. Rejected.
- *`use_effect` running before `Activate()`:* effects are deferred to a later dispatcher tick, after the High-priority activate already ran — not controllable from our side without forking reactor. Rejected.
- *Strip `WS_VISIBLE` from `CREATESTRUCT` at `HCBT_CREATEWND`:* reactor shows via `window.Activate()`, not a `WS_VISIBLE` create flag, so stripping the create style does nothing. Rejected.
- *`HCBT_ACTIVATE` instead of `HCBT_CREATEWND`:* fires during/after the show, too late to reliably beat the paint, and title-based matching there is moot. Rejected as primary; `HCBT_CREATEWND` is strictly earlier.
- *`SetWinEventHook(EVENT_OBJECT_SHOW)`:* fires after the window is already shown → same flash risk. Rejected.

### Decision 2: Suppress the startup show by rewriting `WM_WINDOWPOSCHANGING` in a subclass

In the hook, subclass the HWND and set a `SUPPRESS_SHOW` flag. While the flag is set, the subclass proc rewrites every `WM_WINDOWPOSCHANGING` to strip `SWP_SHOWWINDOW` and add `SWP_HIDEWINDOW`. Reactor's `Activate()` then calls `ShowWindow(SW_SHOW)` as usual, but the window stays hidden — it never composites, so there is no flash. This is the *same* hidden outcome the app already achieves via `SW_HIDE`; the only change is moving it from a dispatcher tick *after* activation to the earliest message *during* activation. Because the window was never shown, the existing `use_effect` `hide()` is now just reinforcement.

> **DWM cloaking was tried first and rejected.** Cloaking the HWND at `HCBT_CREATEWND` (`DwmSetWindowAttribute(.., DWMWA_CLOAK, ..)`) returns `0x80070006` (`E_HANDLE`) — the window is created but not yet valid for DWM that early — so it provided zero flash protection in testing. The `WM_WINDOWPOSCHANGING` rewrite does not depend on DWM and worked. Cloaking and the `Win32_Graphics_Dwm` feature were removed.

*Alternatives considered:*
- *DWM cloak at create:* `E_HANDLE`, no effect (see above). Rejected.
- *Off-screen `SetWindowPos` then move back:* fights reactor's `center_window_on_display` and can still flash at the final position. Rejected.
- *Layered window `alpha = 0`:* heavier, interacts badly with Mica backdrop. Rejected.
- *Minimize-on-create:* shows a taskbar/animation artifact. Rejected.

### Decision 3: Clear the suppression flag on the reveal path

The window-show path (`window::toggle()` show branch) clears `SUPPRESS_SHOW` before `ShowWindow(SW_SHOWNORMAL)` + `SetForegroundWindow`, so the `WM_WINDOWPOSCHANGING` rewrite stops and the window can become visible. After the first reveal, runtime visibility is governed entirely by `ShowWindow(SW_HIDE/SW_SHOWNORMAL)` exactly as before — the suppression only ever touches the pre-reveal startup window. Hide paths (`SW_HIDE`) are unchanged.

### Decision 4: Subclass once in the hook; keep `use_effect` as the fail-safe

The hook subclasses (for both suppression and `WM_CLOSE` close-to-tray) under a one-shot `SUBCLASSED` guard, so `install_close_to_tray()` in the render effect becomes a no-op on the happy path. If the hook never matched (e.g. a future WinAppSDK class rename not caught by the `HCBT_ACTIVATE`-by-title backstop), `capture_hwnd()`'s `FindWindowW` and the fallback `subclass()` still wire up close-to-tray, and the app degrades to today's behavior — a brief flash, not a broken window. This is the explicit fail-safe.

## Risks / Trade-offs

- **Window misidentification in the hook** (another top-level window created first) → match guard is narrow (top-level + exact `WinUIDesktopWin32WindowClass`) and one-shot; on no match the suppression never engages and the `use_effect` fallback yields today's behavior. Confirmed in testing the matched HWND is the `WinUIDesktopWin32WindowClass` top-level window.
- **WinAppSDK class name changes across SDK versions** → soft dependency: the `HCBT_ACTIVATE`-by-title backstop still captures the HWND (close-to-tray preserved), degrading at worst to a brief flash. Worth a follow-up if the SDK is bumped.
- **WinUI replacing the window proc after our subclass** → not observed; the post-create subclass has always worked, and `HCBT_CREATEWND` sees WinUI's registered class proc which it does not later swap. The `SUBCLASSED` guard keeps it single-install.
- **Suppressed window rendering blank on first reveal** → not observed; XAML lays out independent of window visibility, and the reveal path is the same `ShowWindow(SW_SHOWNORMAL)` the app already used. Confirm Mica renders on first reveal.
- **CBT hook left installed** → one-shot `UnhookWindowsHookEx` on adopt; also unhook defensively after `App::render()` returns in case it never fired.

## Migration Plan

No data/state migration. Rollout is a code change to `src/window.rs` / `src/shell.rs` plus one `windows-sys` feature. Validate by running the staged Windows app (`python3 scripts/winrun.py`) and confirming a flash-free launch, then tray reveal/close/re-show. Rollback = revert the change; the prior (flashing) behavior returns with no residual state.

## Open Questions

- ~~Exact main-window class string on the pinned WinAppSDK `2.1.3` runtime~~ — **resolved:** the startup log showed `WinUIDesktopWin32WindowClass` (matched at `HCBT_CREATEWND`, top-level). The match uses the exact class string.
- `FindWindowW` in `capture_hwnd()` is kept strictly as the documented fallback (no-op on the happy path).
