# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Prismatic Tools is a **Windows 11 desktop app**: a background process living in the system tray that opens a WinUI 3 window (Mica backdrop, OS-matched light/dark) hosting individual tools. It is built on the pure-Rust `windows-reactor` crate (declarative WinUI 3, no XAML). The foundational shell lives in two Windows-only modules: `src/shell.rs` (single-instance guard + tray presence + the tool-rendering surface — a persistent app header with the app name on the left and a `Configs` button + a `GitHub` repo-link button on the right, plus window-edge padding, above a hardcoded demo tool standing in for future plugin tools) and `src/window.rs` (the themed reactor window plus Win32 show/hide/close plumbing and the flash-free startup hook). As a tray-resident app it owns its window lifecycle: it starts hidden — flash-free, via a startup CBT hook that claims the window at creation and suppresses its first paint — the tray toggles it, the title-bar **X** hides to tray instead of exiting (a `WM_CLOSE` HWND subclass), a named-mutex single-instance guard drops duplicate launches, and only the tray **Exit** item terminates. See the archived changes for the originating work: `openspec/changes/archive/2026-06-30-bootstrap-tray-shell/` (shell), `.../2026-06-30-tray-click-behavior/` (left-click toggle + Exit-only menu), `.../2026-06-30-tray-ux-polish/` (close-to-tray + single-instance), and `.../2026-06-30-fix-startup-window-flash/` (flash-free startup). The in-window app header (app name + `Configs` + `GitHub`-link buttons + window padding) was added later: `openspec/changes/archive/2026-06-30-add-window-header-bar/` (new `app-header` capability, `openspec/specs/app-header/spec.md`). The window's 800×600-DIP minimum size came later still: `openspec/changes/archive/2026-07-01-add-window-min-size/` (adds the `Minimum window size` requirement to `openspec/specs/themed-window/spec.md`).

Node tooling exists only to drive the dev loop and the OpenSpec spec-driven workflow — there is no JavaScript application code.

## Commands

- `python3 scripts/winrun.py [--release] [--no-run]` (or `yarn win`; `yarn win:release` = `--release`) — **build + run the actual Windows app**: cross-builds `x86_64-pc-windows-gnu`, stages a self-contained Windows App SDK runtime next to the exe, and launches it via WSL interop. `--no-run` builds + stages but doesn't launch. This is the real dev-run path.
- `cargo run` / `cargo build` — builds the **Linux host stub** only. The Windows deps are `cfg(windows)`-gated, so the host build stays light and green (a fast compile check for non-UI logic); it does not produce the app.
- `cargo test` — `cargo test <name>` runs a single test by substring match. Note: there are **no tests** (no `tests/`, no `#[test]` in `src/`); `cargo test` only confirms the host stub compiles.
- `yarn dev` — `nodemon` watch loop over `src/` (runs the host stub via `cargo run`).
- `yarn openspec <cmd>` — OpenSpec CLI (proposals, specs, changes). Not a `package.json` script: `yarn` resolves the `@fission-ai/openspec` bin from `node_modules/.bin`.

Note: `--no-run` (build + stage, don't launch) is only on the `python3 scripts/winrun.py` form — the `yarn win`/`win:release` scripts take no extra args.

Toolchain is pinned: Node `v24.18.0` (`.nvmrc`), Rust `1.96.0` edition 2024 (`rust-toolchain.toml`, which also auto-installs the `x86_64-pc-windows-gnu` cross-target). Run `nvm use` before yarn commands.

### Cross-compilation

The dev host is WSL2 Linux but the product is Windows-only. The full build-stage-run loop works **from WSL** — no Visual Studio / MSVC — and is automated by `scripts/winrun.py`. Key facts (proven; details in `openspec/changes/archive/2026-06-30-bootstrap-tray-shell/design.md` → "Phase 0 spike findings"):

- **Build:** `cargo build --target x86_64-pc-windows-gnu` with the mingw toolchain wired in `.cargo/config.toml`. `windows-reactor` only link-depends on gnu-capable `windows-rs` crates; WinUI 3 is WinRT-activated at runtime, so MSVC is not needed. The MSVC target fails here (no `link.exe`); use the `gnu` target.
- **Stage (self-contained):** `windows-reactor-setup`'s `as_self_contained()` cannot run cross-built (it shells to Windows curl/tar and rejects the `gnu` ABI), so `winrun.py` replicates it: next to the exe it places `Microsoft.WindowsAppRuntime.Bootstrap.dll` (a static import, **not** in `runtime.txt`), the App SDK `2.1.3` runtime DLLs (downloaded from NuGet and cached under `target/winstage-cache`), and an external `<exe>.manifest` sidecar (registration-free WinUI 3 activation).
- **Run:** from a real NTFS path (`%USERPROFILE%\PrismaticTools\run`, override with `PRISMATIC_RUN_DIR`), **not** a `\\wsl.localhost` UNC path — UNC fails DLL load with exit `53`. WSL interop launches the exe. Requires Windows 11 22H2+ for Mica.
- **Release:** add `--release`; `[profile.release] panic = "abort"` is required (unwinding into WinUI's C++ frames is UB).

## Architecture

Two layers, intentionally thin:

- **Rust crate** — the actual product. `src/main.rs` is a thin `cfg` entry (Windows → `shell::run()`, else a stub). `src/shell.rs` acquires the single-instance mutex (`CreateMutexW` named `Local\PrismaticTools.SingleInstance`; second launch returns early), installs the tray (Exit-only menu) plus a left-click handler that toggles the window, and renders the tool surface — a persistent app header (app name left; `Configs` + `GitHub`-link buttons pinned right via a two-column `Grid`, name in a `Star` column, buttons in an `Auto` column; window-edge padding) above the demo tool; `src/window.rs` owns the `windows-reactor` window entrypoint, caches the top-level HWND for Win32 show/hide/toggle, sets an 800×600-DIP minimum window size via reactor's `App::inner_constraints` (a builder call that drives the window's `OverlappedPresenter` minimum — **not** a Win32 `WM_GETMINMAXINFO` subclass; `MIN_INNER_SIZE` const), and subclasses that HWND (`SetWindowLongPtrW`/`GWLP_WNDPROC` + `CallWindowProcW`). The subclass does two jobs: it intercepts `WM_CLOSE` to hide to tray instead of exiting, and — while a startup `WH_CBT` hook (armed before reactor's message loop, adopting the window at `HCBT_CREATEWND` by class `WinUIDesktopWin32WindowClass`, with an `HCBT_ACTIVATE` title backstop) holds the suppress-show flag — rewrites `WM_WINDOWPOSCHANGING` to swallow reactor's startup activation, so the window never paints until the first tray reveal (flash-free launch, spec'd by `openspec/specs/tray-presence/spec.md` → "No startup flash"). Reactor's `AppWindow` bindings are `pub(crate)`, so both visibility and the close intercept are driven through the HWND with `windows-sys`. Both modules are `cfg(windows)`-gated. Windows deps (`windows-reactor` git, `tray-icon`, `windows-sys` with `Win32_Foundation`/`Win32_Security`/`Win32_System_Threading`/`Win32_UI_WindowsAndMessaging` features) live under `[target.'cfg(windows)'.dependencies]`. Build everything here.
- **Node / scripts layer** — dev-only. `scripts/winrun.py` drives the Windows build-stage-run; `nodemon` provides host-stub hot-reload; `@fission-ai/openspec` provides the spec workflow. Vendored agent skills under `.agents/skills/` and `.claude/skills/` are tooling, not product (only the externally-vendored `grilling` + `writing-great-skills` are pinned in `skills-lock.json`; the `openspec-*`, `audit-docs`, and `push` skills are local). Not shipped, not imported by Rust.

Note: crate name in `Cargo.toml` is `primatic-tools` (missing the first `s`) while the npm package is `prismatic-tools` — preserve both as-is unless asked to rename.

## OpenSpec workflow

This repo uses OpenSpec (`schema: spec-driven`) for planning changes before code. Layout:

- `openspec/specs/` — current capability specs (source of truth for what exists).
- `openspec/changes/` — in-flight change proposals; `openspec/changes/archive/` holds completed ones.
- `openspec/config.yaml` — project context and per-artifact rules fed to AI when generating proposals; keep `context:` current as the stack grows.

Skills are available (`openspec-propose`, `openspec-apply-change`, `openspec-archive-change`, `openspec-explore`, `openspec-sync-specs`, also as `/opsx:propose`, `/opsx:apply`, `/opsx:archive`, `/opsx:explore`, `/opsx:sync` slash commands) — prefer these for proposing and implementing changes rather than editing `openspec/` files by hand.

> The globally-installed `openspec` CLI is broken on this dev host (it resolves to a stale Windows npm path and crashes with `MODULE_NOT_FOUND`). Use the repo-local binary via `yarn openspec <cmd>` instead.
