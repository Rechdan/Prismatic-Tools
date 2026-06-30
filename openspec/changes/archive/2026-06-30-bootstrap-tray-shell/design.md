## Context

Prismatic Tools is a greenfield Rust binary (`src/main.rs` is hello-world). The product vision is a Windows 11 tray app that hosts many small tools (eventually as dynamic Lua plugins). This change builds only the shell.

The stack was chosen during exploration: `windows-reactor` (Microsoft, kennykerr — PR #4479 in `microsoft/windows-rs`, landed May 2026) is a pure-Rust, React-style, declarative UI library backed by WinUI 3. It is the only path that delivers all of: pure-ish Rust, native Fluent controls, and a Win11 look. Confirmed-solid facts that anchor this design:

- `App::new().title(..).inner_size(..).backdrop(Backdrop::Mica).presenter(..).render(app_fn)`; `bootstrap()?` must run once at startup; ~60 WinUI controls; `webview()` and `animated_canvas()` exist.
- Packaging is self-contained via `windows-reactor-setup::as_self_contained()` in `build.rs` — ships its own App SDK runtime, no MSIX or separate install required.
- `ThemeRef` provides light/dark brush bindings.

Known constraints from the same research: the crate is v0.0.0 / experimental; it does **not** expose tray icons, raw HWND, borderless windows, window positioning, or live OS-theme-flip detection in the Rust version; startup has a brief black-screen-then-Mica flash; animations are limited; panics must not unwind into WinUI C++ frames.

## Goals / Non-Goals

**Goals:**
- A tray-resident process that toggles a Win11-themed window.
- Mica backdrop + OS light/dark match at startup via Reactor.
- One hardcoded demo tool proving the host→tool render surface.
- Self-contained packaging; correct panic configuration.
- De-risk the flyout vision with a Phase 0 spike before committing to a presentation mode.

**Non-Goals:**
- Lua / dynamic plugin loading (future change).
- Live OS theme re-flip detection (WM_SETTINGCHANGE).
- More than one tool, navigation between tools, or persistence/settings.
- Animations beyond Reactor defaults.

## Decisions

### Decision: Build on `windows-reactor` rather than C# WinUI 3, egui/Tauri, or raw windows-rs
- **Why**: It is the only option satisfying pure-Rust + native Fluent + Win11 look simultaneously. C# would be a mature but polyglot, two-runtime split. egui/Tauri are stable but not native Fluent. Raw windows-rs WinUI 3 has no XAML compiler and is far more code.
- **Trade-off**: betting on a v0.0.0 experimental crate (API churn, feature gaps). Accepted because the alternatives each fail a hard requirement.

### Decision: Tray via the `tray-icon` crate, not Reactor
- **Why**: Reactor's Rust crate has no tray API. `tray-icon` (tao/tauri ecosystem) is proven and standalone.
- **Integration**: the tray runs its own event loop / menu; toggling the window signals the Reactor app. The exact wiring between the `tray-icon` event loop and Reactor's dispatcher is a spike concern (see Open Questions).

### Decision: Window ownership decided by Phase 0 spike, with a guaranteed fallback
- Reactor owns its window and does not document HWND/borderless access. The spike probes `.presenter(..)` for borderless/always-on-top and looks for any HWND or XAML-island surface.
- **Fidelity ladder** (ship the highest reachable rung; bottom rung is guaranteed):
  1. borderless + anchored-near-tray + hide-on-blur + (future) live theme — needs HWND/presenter
  2. borderless + fixed position + manual close — presenter only
  3. plain small Mica window toggled from tray — pure `App::new`, ships with zero unknowns
- The change commits only to rung 3 as the floor; the spike may promote it.

### Decision: Self-contained packaging + `panic = "abort"`
- `build.rs` calls `windows_reactor_setup::as_self_contained()` so end users need no App SDK install.
- Release profile sets `panic = "abort"` because unwinding through WinUI's C++ frames is UB; Reactor catches panics at FFI boundaries and via `ErrorBoundary`, but the global default must still be abort.

### Decision: Theme match at startup only
- Read the current OS app theme once and bind Reactor `ThemeRef` brushes. Live re-theming on OS flip needs WM_SETTINGCHANGE, which needs the HWND we may not have — explicitly deferred.

## Risks / Trade-offs

- **Reactor v0.0.0 churn / missing features** → Pin an exact version; isolate Reactor calls behind a thin `window` module so API breaks are localized; be ready to patch/PR upstream.
- **No HWND → flyout polish (anchor, hide-on-blur, live theme) may be unreachable** → Fallback rung 3 ships a usable product regardless; flyout is an upgrade, not a gate.
- **Tray event loop vs Reactor dispatcher threading conflict** → Resolve in the spike; keep tray↔window signalling to a minimal, well-defined channel.
- **Startup black-screen-then-Mica flash** → Cosmetic; accept for MVP, revisit if jarring.
- **Windows-only, requires Win11 22H2+ for Mica** → Expected; document the dev/runtime requirement. Build must happen on a Windows host (current dev loop already assumes `cargo run`).

## Migration Plan

Greenfield: `src/main.rs` hello-world is replaced by the shell entrypoint. No data or users to migrate. Rollback = revert the change; the repo returns to hello-world.

## Phase 0 spike findings — build/run pipeline (VALIDATED on the WSL2 dev host)

The dev host is WSL2 Linux with no Visual Studio / MSVC. We proved a full build-and-run loop anyway:

- **WSL cargo + mingw builds a real Reactor app.** A genuine `App::new().render(app)` binary compiles and links to a Windows `.exe` with the Linux `cargo` and the `x86_64-pc-windows-gnu` target (mingw linker from `.cargo/config.toml`). No MSVC, no `link.exe`, no Visual Studio. `windows-reactor` only link-depends on gnu-capable `windows-rs` crates; WinUI 3 is WinRT-activated at runtime, not linked.
- **The `windows-reactor` crate is not on crates.io** — depend on it via git (`microsoft/windows-rs`); `windows-reactor` `v0.0.0`, edition 2024.
- **`windows-reactor-setup` cannot run in a Linux cross-build.** `as_self_contained()` / `as_framework_dependent()` pass `assert_windows()` (gnu target OS = windows) but then shell out to `%SystemRoot%\System32\curl.exe` + `tar.exe` (unset on Linux → silent no-op), and the ABI match only accepts `msvc` or `gnu`+`llvm` (plain `gnu` hits `panic!("unsupported target environment")`). So the crate's build-time staging is unusable when cross-building.
- **Manual self-contained staging from Linux WORKS.** Replicate it by hand, next to the exe: (1) `Microsoft.WindowsAppRuntime.Bootstrap.dll` from the crate's `bootstrap/x64/` — **required: it is a static import of the exe, and is NOT listed in `runtime.txt`** (a missing Bootstrap.dll fails the loader with `0xC0000135` STATUS_DLL_NOT_FOUND, surfaced as exit `53`); (2) download `Microsoft.WindowsAppSDK.Runtime` `2.1.3` nupkg with Linux `curl`, extract the `win10-x64` MSIX, copy the files listed in `assets/runtime.txt`; (3) `assets/app.manifest` as an external `**<exe>.manifest**` sidecar (registration-free SxS activation of the WinUI 3 COM/WinRT classes). Result: a live WinUI 3 window (`MainWindowTitle = "Spike"`, `Responding = True`).
- **Two gotchas, both resolved:**
  1. **Run from NTFS, not the WSL share.** Launching the exe from `\\wsl.localhost\...` fails early with exit `53` (`ERROR_BAD_NETPATH`) — App SDK DLL loading rejects the UNC path. Copy to `C:\...` (or set the cargo target-dir under `/mnt/c`) and run there.
  2. **Version skew.** The machine has `WindowsAppRuntime 2.2.0.0`; the crate bootstraps `2.1.3`. Framework-dependent gives `0x80040154` "Class not registered". Self-contained staging of `2.1.3` beside the exe sidesteps it.

**Implication for tasks 2.x (packaging):** target `x86_64-pc-windows-gnu` with WSL cargo; do **not** rely on `windows-reactor-setup` in `build.rs` for cross-builds — instead add a staging step (script / cargo-xtask) that performs the manual self-contained stage above, and run the exe from NTFS.

## Phase 0 spike findings — Reactor window API (read from crate source @ `a1e9fce`)

- **Presenter (task 1.2).** `App::presenter(PresenterKind)` exposes only `Default` (overlapped + title bar), `FullScreen` (frameless, fills monitor), `CompactOverlay` (floating PiP). **No borderless-windowed or always-on-top variant in the enum.** However those are still reachable (see HWND below) by driving `Microsoft.UI.Windowing.OverlappedPresenter` (`SetBorderAndTitleBar(false,false)`, `IsAlwaysOnTop(true)`, `Move(pos)`) via `windows-rs`.
- **HWND / window handle (task 1.3) — YES.** `ReactorHost::window() -> &Window` exposes the real `Microsoft.UI.Xaml.Window`. From it: `window.AppWindow()` (via `IWindow2`) → `AppWindow`, and the raw `HWND` via `Microsoft.UI.Win32Interop.GetWindowFromWindowId(appWindow.Id)`. Reach the host from anywhere on the UI thread with `with_active_host(|h| h.window())`, or take full control with `App::run_custom(|app| …)` ("caller manages windows and content"). So borderless flyout, custom positioning, hide-on-blur (`Window.Activated` / `WM_ACTIVATE`), DWM corner prefs, and `WM_SETTINGCHANGE` are all achievable.
- **Theme — nearly free.** `set_requested_theme(Default | Light | Dark)`; `Default` **inherits the OS setting** (and WinUI updates the title bar via the crate's `update_titlebar_theme`). So "match OS light/dark at startup" is the default behaviour, and live OS flips are largely handled by WinUI's `ElementTheme::Default` natively — the WM_SETTINGCHANGE work we deferred may be unnecessary.
- **Backdrop.** `Backdrop::{Mica, MicaAlt, Acrylic}`; also `Backdrop::apply_to(window)` for `run_custom` setups. Mica confirmed.
- **Runtime mutators on the host:** `set_presenter`, `set_backdrop`, `set_requested_theme`, `set_titlebar_height`, `new_with_window_options`.

### Window-fidelity rung decision (task 1.5)
**MVP ships rung 3** — a plain `Default`-presenter Mica window toggled from the tray (`Default` auto-follows OS theme; zero `windows-rs` `AppWindow` gymnastics; already proven to run). **Rung 1 (borderless anchored flyout + hide-on-blur) is reachable later** via `host.window() → AppWindow → OverlappedPresenter`, deferred to keep MVP off the v0.0.0 `AppWindow` escape-hatch path.

### Tray ↔ Reactor coexistence (task 1.4) — RESOLVED
A combined `tray-icon` 0.19 + `windows-reactor` binary (built by WSL cargo + mingw, staged + run as above) confirmed: the tray icon builds on the **main thread before** `App::render`, and reactor's WinUI message loop pumps the shared thread — both subsystems initialize and the window stays responsive (`Responding = True`), no deadlock. So the simple single-thread model works (tray created first, then `render`); the separate-thread + `DispatcherQueue.TryEnqueue` design remains available if menu-event latency or pump conflicts show up under load. Menu-event delivery uses `MenuEvent::set_event_handler` fed by the running pump.

## Open Questions

- Build vs run filesystem: settle on building under a `/mnt/c` target-dir vs. build-on-Linux-fs + copy-to-NTFS for the dev loop.
- Confirm a tray **menu click** (e.g. "Quit"/"Toggle") round-trips through the shared pump in practice (the handler is wired; only an interactive click was unverified in the spike).
