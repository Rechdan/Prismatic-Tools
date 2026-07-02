# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Prismatic Tools is a **Windows 11 desktop app**: a tray-resident background process that opens a WinUI 3 window (Mica backdrop, OS-matched light/dark) hosting individual tools. It is built on the pure-Rust `windows-reactor` crate (declarative WinUI 3, no XAML). Tools are Lua **widgets** loaded from a `widgets/` folder beside the exe.

As a tray app it owns its window lifecycle:

- Starts **hidden and flash-free** — a startup CBT hook suppresses the first paint, so the window never flickers on launch.
- The tray icon's **left-click toggles** the window.
- The title-bar **X hides to tray** instead of exiting.
- A **named-mutex single-instance guard** drops duplicate launches.
- Only the tray **Exit** item terminates the process.

Node tooling exists only to drive the dev loop and the OpenSpec workflow — there is no JavaScript application code.

## Gotchas / must-knows

Break-your-build facts. Each stands alone; read this section before touching the build/run loop or the UI.

- **Build target is `x86_64-pc-windows-gnu`, not MSVC.** MSVC fails here (no `link.exe`). `windows-reactor` only link-depends on gnu-capable `windows-rs` crates, and WinUI 3 is WinRT-activated at runtime, so MSVC is not needed.
- **Typecheck the real UI with `cargo check --target x86_64-pc-windows-gnu`.** Plain `cargo build`/`cargo run` compile only the Linux host stub — all UI code (`shell`/`window`/`widget`) is `cfg(windows)`-gated and **not compiled** there, so the stub can't catch type errors in it.
- **Run from an NTFS path, never a `\\wsl.localhost` UNC path** — UNC fails DLL load with exit `53`. Default run dir is `%USERPROFILE%\PrismaticTools\run` (override with `PRISMATIC_RUN_DIR`). Needs Windows 11 22H2+ for Mica.
- **Release requires `panic = "abort"`** (`[profile.release]`) — unwinding into WinUI's C++ frames is UB.
- **Use `yarn openspec`, not the global `openspec`** — the globally-installed CLI resolves a stale Windows npm path and crashes with `MODULE_NOT_FOUND`. The repo-local bin works.
- **Crate name is `primatic-tools`** (missing the first `s`) in `Cargo.toml`, while the npm package is `prismatic-tools`. Preserve both as-is unless asked to rename.
- **Reactor API gotchas live in `docs/reactor-notes.md`** — reach for them before re-deriving reactor's API. Key points: clickable widgets take **string content only** (use `ElementExt::on_tapped` on a `border`/stack for a clickable container with custom content); implicit transitions animate **opacity/scale/translation, not brush color** (crossfade a hover-fill overlay's opacity); `ThemeRef` brushes (`CardBackground`/`CardStroke`/`SubtleFill`…) are theme-aware and `Clone`-not-`Copy`.

## Commands

- `python3 scripts/winrun.py [--release] [--no-run]` (or `yarn win`; `yarn win:release` = `--release`) — **the real dev-run:** cross-builds the app, stages a self-contained Windows App SDK runtime next to the exe, and launches it via WSL interop. `--no-run` builds + stages but doesn't launch (that flag is only on the `winrun.py` form; the `yarn win`/`win:release` scripts take no extra args). The separate `python3 scripts/winrun.py --sync-widgets` mode is **copy-only**: it mirrors the repo `widgets/` into `<run-dir>/widgets` and returns — no build, no staging, no launch — so a running app + in-app **Reload** picks up widget edits live (used by `yarn dev` below).
- `cargo run` / `cargo build` — builds the **Linux host stub** only: a fast, always-green compile check for non-UI logic (Windows deps are `cfg(windows)`-gated). Does not produce the app.
- `cargo test` — there are **no tests**; this only confirms the stub compiles. `cargo test <name>` filters by substring.
- `yarn dev` — runs two watchers together via `concurrently`: `dev:app` (`nodemon -w ./src -w ./README.md -e "*"` → re-runs **`yarn win`**, the full build-stage-run, on any `src/` or `README.md` change — the README is compiled into the exe, so an edit needs a rebuild, not a widget-style sync) and `dev:widgets` (`nodemon -w ./widgets -e "*" --on-change-only` → runs `winrun.py --sync-widgets` on a widget edit — copy-only, **no rebuild or relaunch**). Widget edits therefore mirror into the run dir; press the config view's **Reload** to load them without restarting. `--on-change-only` keeps the widgets watcher idle at startup so it never races `yarn win`'s run-dir wipe.
- `yarn openspec <cmd>` — OpenSpec CLI via the repo-local bin (resolved from `node_modules/.bin`; not a `package.json` script).

Toolchain is pinned: Node `v24.18.0` (`.nvmrc`), Rust `1.96.0` (`rust-toolchain.toml`, which also auto-installs the `x86_64-pc-windows-gnu` cross-target), edition 2024 (`Cargo.toml`). Run `nvm use` before yarn commands.

## Cross-compilation

The dev host is WSL2 Linux but the product is Windows-only. The full build-stage-run loop works **from WSL** — no Visual Studio / MSVC — and is automated by `scripts/winrun.py`. (Target, run-path, and release watch-outs are in [Gotchas](#gotchas--must-knows); proven details in the bootstrap change's `design.md` → "Phase 0 spike findings".) The pipeline:

- **Build** — `cargo build --target x86_64-pc-windows-gnu`, with the mingw toolchain wired in `.cargo/config.toml`.
- **Stage (self-contained)** — `windows-reactor-setup`'s `as_self_contained()` can't run cross-built (it shells to Windows curl/tar and rejects the `gnu` ABI), so `winrun.py` replicates it. Next to the exe it places `Microsoft.WindowsAppRuntime.Bootstrap.dll` (a static import, **not** in `runtime.txt`), the App SDK `2.1.3` runtime DLLs (downloaded from NuGet and cached under `target/winstage-cache`), and an external `<exe>.manifest` sidecar (registration-free WinUI 3 activation). `stage()` wipes and re-stages the run dir each run, copies the runtime files enumerated from reactor-setup's `runtime.txt`, and copies the repo `widgets/` folder next to the exe every run (the app resolves widgets relative to the exe).
- **Run** — WSL interop launches the exe from the run dir.

## Architecture

Two layers, intentionally thin. All Rust UI modules (`home`/`shell`/`window`/`widget`) are `cfg(windows)`-gated; the Windows deps (`windows-reactor` git, `tray-icon`, `windows-sys` with the `Win32_*` features it needs, `mlua` with `lua54`+`vendored`, `toml`, and `pulldown-cmark` — the README markdown parser, `default-features = false`) live under `[target.'cfg(windows)'.dependencies]` (exact features in `Cargo.toml`).

### Rust crate — the product

`src/main.rs` is a thin `cfg` entry (Windows → `shell::run()`, else a stub).

- **`src/shell.rs`** — the single-instance guard, tray, and rendered tool surface.
  - **Single instance** — `CreateMutexW` named `Local\PrismaticTools.SingleInstance`; a second launch returns early.
  - **Tray** — an Exit-only menu plus a left-click window toggle.
  - **Layout** — a two-row `Grid` (`Auto` header row, `Star` main row) inset from the window edge via a `.margin` (WinUI `Grid` has no `Padding`): the header row holds the app name (left) and `Configs` + `GitHub`-link buttons (right); the main row is a two-pane `Grid` (`Pixel(200)` nav column + `Star` right container).
  - **Nav** — an accessible `button("Home")` at the top (built unconditionally, stretched full-width, selects Home), then a `Tools` heading plus the loaded widget as a full-width clickable **card**: a tapped `border` rendering the widget's optional `nav` preview or, by default, its manifest name, with a `SubtleFill` hover fill animated by opacity crossfade.
  - **View state** — the right container shows exactly one of three views, held in a `View { Home, Widget, Config }` `use_state` that **defaults to `Home`** (the README home screen, `src/home.rs`): the header `Configs` button selects `Config`, the nav Home button selects `Home`, the nav card `on_tapped` selects `Widget` (explicit selects, not toggles). A `hovered` `use_state` drives the card fill.
- **`src/home.rs`** — the `Home` view (the shell's default landing view): renders the project `README.md`, bundled into the exe at build time via `include_str!("../README.md")` — so **`README.md` is a compile input**; an edit needs a rebuild (not a widget sync). `pulldown-cmark` parses it and `render_markdown` maps **block-level** markdown to reactor elements (headings via the type ramp, wrapping paragraphs, `•`/`N.` lists, monospaced fenced code in a `border`, `---` rules), **flattening inline runs to text** — reactor renders no per-run inline styling and no inline-link navigation (see `docs/reactor-notes.md`). `pub fn view()` wraps the document in a vertical `scroll_viewer`. Rendering is fault-tolerant: bad markdown degrades to text, never panics.
- **`src/window.rs`** — owns the `windows-reactor` window entrypoint, caches the top-level HWND for Win32 show/hide/toggle, sets a 1024×768-DIP initial size (`App::inner_size`) and a matching 1024×768-DIP minimum (`App::inner_constraints` — this drives the window's `OverlappedPresenter` minimum, **not** a Win32 `WM_GETMINMAXINFO` subclass), and subclasses that HWND. The subclass does two jobs: it intercepts `WM_CLOSE` to hide-to-tray instead of exiting, and — while a startup `WH_CBT` hook holds a suppress flag — swallows reactor's startup window activation so the window never paints until the first tray reveal (the flash-free launch). Reactor's `AppWindow` bindings are `pub(crate)`, so both visibility and the close intercept are driven through the HWND with `windows-sys`.
- **`src/widget/`** — the Lua widget runtime (the tool mechanism). Tools live under `widgets/` beside the exe, one folder per widget, each with a `widget.toml` manifest (`name` required; `version`/`entry` optional, `entry` default `main.lua`; the folder name is the widget id).
  - **Load** — `package.rs` resolves `widgets/` via `current_exe()`, picks the first folder (sorted), parses the manifest, and reads the Lua entry.
  - **Sandbox** — `runtime.rs` runs it in a **sandboxed `mlua` VM**: safe libs only (`base`/`table`/`string`/`math`/`coroutine`/`utf8`/hardened `package`; no `os`/`io`/`debug`; `dofile`/`loadfile` stripped; `require` scoped to the widget folder, no native modules).
  - **Contract** — the chunk returns a `{ state, render, nav? }` table: `state` is a plain-data table the host holds (source of truth, kept for future persistence); `render(state)` returns a declarative UI table (`vstack`/`hstack`/`text`/`button`/`border`; hash keys = props, array entries = children) mapped to reactor elements each render; optional `nav(state)` returns a **display-only** preview for the nav card, sharing `render`'s `state` handle (mapped without callbacks — a `button` in a nav tree is an inline error). The `border` node wraps one child (multiple children → implicit `vstack`).
  - **Callbacks** — a button's Lua `on_click` is captured as an owned `mlua::Function`; invoking it mutates Lua state, then a `use_state` tick re-renders.
  - **Errors** — load/render/callback errors surface as **visible text — nothing panics** (release aborts into WinUI).
  - **Reload** — the config view's **Reload widget(s)** button (label pluralized by `widget::count()`) re-runs the loader into a `use_ref` on click: a fresh VM, so in-memory state resets and on-disk edits are picked up, active view unchanged.
  - **Shipped** — `widgets/counter/` is the first widget (the ported demo counter, with a `nav` preview mirroring its click count).

### Node / scripts layer — dev-only

`scripts/winrun.py` drives the Windows build-stage-run (and a copy-only `--sync-widgets` mode); `nodemon` + `concurrently` provide Windows-app hot-reload (`yarn dev` re-runs `yarn win` on `src/` changes and mirrors `widgets/` into the run dir on widget edits); `@fission-ai/openspec` provides the spec workflow. Vendored agent skills under `.agents/skills/` and `.claude/skills/` are tooling, not product — not shipped, not imported by Rust.

## OpenSpec workflow

This repo uses OpenSpec (`schema: spec-driven`) to plan changes before code.

- `openspec/specs/` — current capability specs (source of truth for what exists).
- `openspec/changes/` — in-flight change proposals; `openspec/changes/archive/` holds completed ones.
- `openspec/config.yaml` — project context and per-artifact rules fed to AI when generating proposals; keep `context:` current as the stack grows.

Prefer the OpenSpec skills (`openspec-propose`, `openspec-apply-change`, `openspec-archive-change`, `openspec-explore`, `openspec-sync-specs`, also `/opsx:propose`, `/opsx:apply`, `/opsx:archive`, `/opsx:explore`, `/opsx:sync`) over editing `openspec/` files by hand.

## Provenance

The originating change proposal for every capability above lives in `openspec/changes/archive/` (named chronologically) — read those, plus git history, for the *why* behind a given piece. Reactor API + cross-build details are in `docs/reactor-notes.md`.
