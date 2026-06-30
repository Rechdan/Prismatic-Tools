## Why

Prismatic Tools is currently an empty hello-world Rust binary. Before any individual tool can be built, the product needs its shell: a process that lives in the Windows system tray and opens a window that visually belongs on Windows 11 (Mica backdrop, light/dark matching the OS). This change stands up that shell end-to-end on the `windows-reactor` crate (pure-Rust WinUI 3), proving the foundation that every future tool plugs into.

## What Changes

- Add a system-tray presence: a tray icon (via the `tray-icon` crate) with a menu that toggles the main window and exits the app. The process keeps running with no visible window until invoked.
- Add a `windows-reactor` window with a `Backdrop::Mica` system backdrop and rounded Win11 styling, whose light/dark theme is matched to the OS theme (via `ThemeRef`) at startup.
- Render one hardcoded demo tool inside the window to prove the host→tool rendering surface (no plugin system yet).
- Configure self-contained packaging via `windows-reactor-setup::as_self_contained()` so the app ships the Windows App SDK runtime and needs no separate install; set `panic = "abort"` for release per Reactor's FFI-boundary requirement.
- First task is a **Phase 0 spike**: probe Reactor's `.presenter(...)` for borderless/always-on-top support and find whether the window's raw HWND or a XAML-island surface is reachable. The result selects the window-fidelity rung (full anchored, hide-on-blur flyout vs. a plain toggled Mica window) without blocking the rest of the change.

## Capabilities

### New Capabilities
- `tray-presence`: background process with a system-tray icon and menu that toggles the main window's visibility and exits the app; defines the no-window-on-launch lifecycle.
- `themed-window`: a Windows-Reactor main window with Mica backdrop, Win11 rounded styling, and OS light/dark theme matched at startup; window-fidelity rung is selected by the Phase 0 spike.
- `tool-surface`: the host's contract for rendering a single tool's UI inside the main window, demonstrated by one hardcoded demo tool.

### Modified Capabilities
<!-- None — this is the first capability set in the repo. -->

## Impact

- **Code**: replaces `src/main.rs` hello-world with the shell entrypoint (`bootstrap()`, tray init, `App::new()` window); adds modules for tray, window/theme, and the demo tool.
- **Dependencies (Cargo.toml)**: adds `windows-reactor`, `tray-icon`, `windows-core`/`windows`; adds `windows-reactor-setup` as a build-dependency and a `build.rs`.
- **Build/packaging**: introduces `build.rs` staging the App SDK runtime (self-contained); release profile sets `panic = "abort"`. Requires building/running on Windows 11 (22H2+ for Mica) — the current dev loop (`nodemon` + `cargo run`) still applies on a Windows host.
- **Out of scope** (future changes): Lua dynamic plugins, live OS theme re-flip detection (WM_SETTINGCHANGE), and any second tool.
