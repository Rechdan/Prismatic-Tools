## 1. Phase 0 spike (de-risk before committing UI mode)

- [x] 1.1 Add a throwaway `windows-reactor` example that opens `App::new().backdrop(Backdrop::Mica)` and confirms it builds and runs on the Windows host — **DONE: real Reactor app builds via WSL cargo + mingw (`x86_64-pc-windows-gnu`, no MSVC) and runs as a live WinUI 3 window via manual self-contained staging + WSL interop. Full pipeline recorded in `design.md` → "Phase 0 spike findings".**
- [x] 1.2 Probe `.presenter(..)`: determine whether borderless and always-on-top are reachable; record exact API used — **DONE: `PresenterKind` = Default/FullScreen/CompactOverlay only (no borderless/always-on-top); reachable via `host.window() → AppWindow → OverlappedPresenter` (windows-rs). See design.md.**
- [x] 1.3 Hunt for raw HWND or a XAML-island / render-into-existing-window surface from Reactor; record yes/no and how — **DONE: YES. `ReactorHost::window() -> &Microsoft.UI.Xaml.Window`; HWND via `AppWindow` + `Win32Interop.GetWindowFromWindowId`; also `App::run_custom` for full control.**
- [x] 1.4 Stand up `tray-icon` alongside Reactor; confirm the tray event loop and Reactor's WinRT dispatcher coexist and that the tray can signal the window — **DONE: combined `tray-icon` 0.19 + `windows-reactor` binary (WSL/mingw) runs with tray built on main thread + reactor window responsive, no deadlock. See design.md. (Single-thread model; separate-thread fallback documented.)**
- [x] 1.5 Decide the window-fidelity rung (1 flyout / 2 fixed-borderless / 3 plain Mica) and record the decision in `design.md` Open Questions — **DONE: MVP = rung 3 (plain Default-presenter Mica window); rung 1 flyout reachable later via AppWindow.**

## 2. Project & build setup

- [x] 2.1 Add dependencies to `Cargo.toml`: `windows-reactor`, `tray-icon`, `windows-core`/`windows` (pin exact versions) — **DONE: `windows-reactor` (git) + `tray-icon` 0.19 under `[target.'cfg(windows)'.dependencies]` so the Linux host build stays green.**
- [x] 2.2 Add `windows-reactor-setup` as a build-dependency and create `build.rs` calling `as_self_contained()` — **DONE via deviation: `as_self_contained()` can't run cross-built (spike), so staging is replicated in `scripts/winrun.py` (build → stage Bootstrap.dll + runtime + manifest → run from NTFS). No `build.rs`.**
- [x] 2.3 Set `[profile.release] panic = "abort"` and verify a release build links — set in `Cargo.toml`; hello-world release build links on Linux (full Windows link pending Windows host)
- [x] 2.4 Fix the crate name typo if desired (`primatic-tools` → `prismatic-tools`) or record the decision to keep it — **Decision: keep `primatic-tools`** (per CLAUDE.md; npm pkg stays `prismatic-tools`)

## 3. Tray presence

- [x] 3.1 Initialize the tray icon on startup with the app icon; ensure no window appears on launch — **DONE: tray initialized; window hidden on first render via `ShowWindow(SW_HIDE)` (HWND captured by title at mount). Verified: launches with no visible window (`MainWindowTitle` empty, screenshots show none).**
- [x] 3.2 Build the tray menu with "Open" (toggle window) and "Exit" items — **DONE: "Show / hide window" + "Exit" in `shell.rs`.**
- [x] 3.3 Wire tray actions to window show/hide and to clean process shutdown (tray icon removed) — **DONE: "Show / hide" → `toggle_window()` (cached HWND + `ShowWindow`), verified show-from-hidden reveals the window (screenshot). Exit → `process::exit` (tray dropped → icon removed). HWND cached at mount because a hidden WinUI window reports an empty title (FindWindow-by-title then misses).**
- [x] 3.4 Verify the process persists with the window hidden and the tray icon remains present — **DONE: process stays alive + `Responding=True` with the window hidden.**

## 4. Themed window

- [x] 4.1 Implement the shell entrypoint: `bootstrap()?` then `App::new()` with `Backdrop::Mica` and the rung chosen in task 1.5 — **DONE: `shell::run()` → `App::new().title().backdrop(Backdrop::Mica).render(...)` (rung 3; `bootstrap` handled by App SDK init, no explicit call needed).**
- [x] 4.2 Read the OS app theme at startup and bind Reactor `ThemeRef` brushes for light/dark — **DONE (free): `RequestedTheme::Default` auto-follows the OS theme; screenshot confirms dark-theme content under OS dark mode.**
- [x] 4.3 Confirm Mica backdrop + rounded Win11 styling render on Windows 11 22H2+ — **DONE: confirmed live on Windows 11 build 26200 (screenshot).**
- [x] 4.4 Isolate all Reactor window calls behind a thin `window` module to localize future API churn — **DONE: `src/window.rs` owns `App::new()...render()` + Win32 show/hide/toggle; `shell.rs` calls `window::run/toggle/capture_hwnd/hide`.**

## 5. Tool surface + demo tool

- [x] 5.1 Define the host content area where a single tool's UI is rendered — **DONE: `tool_surface(cx)` is the rendered root inside the window.**
- [x] 5.2 Implement one hardcoded demo tool with at least one interactive control using `use_state` — **DONE: counter via `cx.use_state` + a "Click me" `button` updating `clicks`.**
- [x] 5.3 Verify interacting with the control updates the rendered UI (render/state loop works) — **DONE: UI-Automation invoke of "Click me" increments the rendered counter (`clicks` updates live), confirming button → `use_state` → re-render.**

## 6. Verification

- [x] 6.1 Run the app on Windows 11: tray appears, window toggles, Mica + correct theme, demo tool interacts — **DONE on Win11 26200: starts hidden → show reveals Mica window + OS-dark theme + demo tool (screenshots); show/hide path verified; demo button click increments the counter via UI Automation. (Tray *menu* click not driven headlessly, but the same `toggle()` path + handler are proven.)**
- [x] 6.2 Confirm a release build runs self-contained without a separate Windows App SDK install — **DONE: `winrun.py --release` builds (panic=abort) + stages; release exe runs, `Responding=True`, no errors, no separate install.**
- [x] 6.3 Update `CLAUDE.md` with the real run/build notes (Windows host requirement, build.rs, panic=abort) — **DONE: rewrote Commands + Cross-compilation + Architecture for the WSL build-stage-run pipeline; added `yarn win` / `winrun.py`.**
