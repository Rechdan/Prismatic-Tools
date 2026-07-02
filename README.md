# Prismatic Tools

A Windows 11 tray app that hosts small tools, called widgets, inside a single WinUI 3 window. Widgets are written in Lua and loaded from a folder next to the app, so you can add or change one without rebuilding the app.

## What it is

Prismatic Tools runs quietly in the system tray. Click the tray icon and a window opens with a Mica backdrop that follows your light or dark theme. Inside, a left-hand nav lists the tools you have installed and the right pane shows the active one.

The window belongs to the tray, not the taskbar, so it behaves the way a tray app should:

- It starts hidden and does not flash on launch. A startup hook swallows the first paint, so nothing appears on screen until you open it.
- Left-clicking the tray icon toggles the window.
- Closing the window with the X hides it back to the tray instead of quitting.
- A named-mutex guard means a second launch does nothing, so you never end up with two copies running.
- Only the tray's Exit item actually shuts the process down.

Under the hood it is built on [`windows-reactor`](docs/reactor-notes.md), a pure-Rust way to drive WinUI 3 declaratively with no XAML. There is no JavaScript application code. Node is here only to run the dev loop and the spec workflow.

## Status

Early and moving. Windows 11 only. One widget ships today (`counter`), which exists mostly to prove the host-to-widget render loop works. The widget API is usable but not frozen, so expect it to change.

## Requirements

To run the app:

- Windows 11, version 22H2 or newer (Mica needs it).
- Run it from a normal drive path. Launching from a `\\wsl.localhost` UNC path fails to load its DLLs and exits with code 53.

To build it:

- WSL2 with a Linux distro. The whole build runs from Linux. You do not need Visual Studio or MSVC.
- Node `v24.18.0` (see `.nvmrc`).
- Rust `1.96.0` (see `rust-toolchain.toml`). It installs the `x86_64-pc-windows-gnu` cross-target for you.

## Quick start

```bash
nvm use          # switch to the pinned Node version
yarn             # install the dev tooling
yarn win         # cross-build, stage the runtime, and launch on Windows
```

`yarn win` cross-compiles the app from WSL, stages a self-contained Windows App SDK runtime next to the exe, and launches it through WSL interop. The first run downloads the runtime and caches it, so later runs are quicker. Add `yarn win:release` for a release build.

By default the app runs out of `%USERPROFILE%\PrismaticTools\run`. Set `PRISMATIC_RUN_DIR` to change that.

## Development

The product is Windows-only, but the host you build on is Linux. That split shapes the commands.

```bash
yarn dev         # watch src/ and widgets/, rebuild or hot-reload on change
yarn win         # one full build-stage-run
cargo check --target x86_64-pc-windows-gnu   # typecheck the real UI
cargo build      # fast compile of the Linux stub only
```

`yarn dev` runs two watchers at once. One watches `src/` and re-runs the full build-stage-run whenever Rust code changes. The other watches `widgets/` and, on a widget edit, copies the folder into the run directory without rebuilding. Press Reload in the app's config view to pick up widget changes live.

A word on `cargo build` and `cargo check`. All of the UI code lives behind `cfg(windows)`, so a plain `cargo build` on Linux compiles a small host stub and never sees it. That is fine for a quick check of non-UI logic, but it will not catch a type error in the window, shell, or widget code. To typecheck the real thing, build against the Windows target: `cargo check --target x86_64-pc-windows-gnu`. There are no tests, so `cargo test` only confirms the stub compiles.

For the change workflow, use `yarn openspec`, not any globally installed `openspec`.

## Gotchas

These are the things that will break your build or your run if you do not know them.

- The build target is `x86_64-pc-windows-gnu`, not MSVC. There is no `link.exe` in this setup, and WinUI 3 is activated at runtime, so the GNU toolchain is all you need.
- `cargo build` on Linux only compiles the stub. Use `cargo check --target x86_64-pc-windows-gnu` to actually typecheck the UI.
- Run from an NTFS path. A UNC path like `\\wsl.localhost\...` fails DLL load and exits 53.
- Release builds set `panic = "abort"`. Unwinding a panic into WinUI's C++ frames is undefined behavior, so a panic has to abort instead.
- Use `yarn openspec`. The global CLI resolves a stale path and crashes.
- The crate is named `primatic-tools` in `Cargo.toml`, missing the first `s`, while the npm package is `prismatic-tools`. Both are intentional. Leave them as they are unless you mean to rename.
- Reactor has its own sharp edges, written up in [`docs/reactor-notes.md`](docs/reactor-notes.md). The short version: clickable widgets take string content only (wrap a `border` or stack with `on_tapped` if you want a clickable container with custom content), implicit transitions animate opacity, scale, and translation but not brush color (so crossfade a hover overlay instead of tweening a color), and `ThemeRef` brushes are theme-aware and clone rather than copy.

## Architecture

Two thin layers. The Rust crate is the product. The Node and Python scripts only exist to build and run it.

All of the Windows code sits behind `cfg(windows)`, and its dependencies (`windows-reactor`, `tray-icon`, `windows-sys`, `mlua` with `lua54` and `vendored`, and `toml`) live under a `cfg(windows)` block in `Cargo.toml`. On Linux none of it compiles, which is what keeps the stub build fast and green.

### The Rust crate

`src/main.rs` is a thin entry point: on Windows it hands off to `shell::run()`, otherwise it is a stub.

`src/shell.rs` owns the single-instance guard, the tray, and the rendered surface. The guard is a named mutex, `Local\PrismaticTools.SingleInstance`, and a second launch returns early. The tray has an Exit item and a left-click that toggles the window. The layout is a two-row grid inset from the window edge: a header row with the app name on the left and Configs and GitHub buttons on the right, and a main row split into a fixed 200-pixel nav column and a flexible right pane. The nav shows each widget as a full-width card that renders the widget's optional preview, or its name by default, with a subtle fill that fades in on hover. A small piece of view state decides whether the right pane shows the active widget or the config placeholder.

`src/window.rs` owns the reactor window. It caches the top-level window handle so it can show, hide, and toggle through Win32, sets an initial and minimum size of 1024 by 768, and subclasses the window to do two jobs. First, it catches the window's close message and hides to the tray instead of exiting. Second, while a startup hook holds a suppress flag, it swallows reactor's initial window activation so nothing paints until the first tray reveal. That is what gives the flash-free launch.

`src/widget/` is the Lua runtime, which is how tools plug in. `package.rs` finds the `widgets/` folder next to the exe, picks a widget, reads its manifest, and loads its Lua entry file. `runtime.rs` runs that code in a sandboxed `mlua` VM with only the safe standard libraries. There is no `os`, `io`, or `debug`, `dofile` and `loadfile` are removed, and `require` is scoped to the widget's own folder with no native modules. A widget returns a table with `state`, `render`, and an optional `nav`. The host holds `state` as the source of truth, calls `render(state)` to build the UI, and calls `nav(state)` for the display-only card preview. Button clicks run a captured Lua function that mutates state, then the host re-renders. If loading, rendering, or a callback fails, the error shows up as text on screen. Nothing panics, because a release build would abort.

### The scripts

`scripts/winrun.py` does the real work of building for Windows from Linux. It cross-compiles with the GNU toolchain, then stages a self-contained runtime next to the exe: the bootstrap DLL, the Windows App SDK runtime files, and an external manifest for registration-free WinUI 3 activation. It wipes and re-stages the run directory each time and copies the `widgets/` folder over, since the app resolves widgets relative to the exe. A separate `--sync-widgets` mode only copies widgets and returns, which is what powers live widget reload.

`nodemon` and `concurrently` wire up the watch loop, and `@fission-ai/openspec` runs the spec workflow.

## Writing a widget

A widget is a folder under `widgets/` with a `widget.toml` manifest and a Lua entry file. The folder name is the widget's id.

```toml
# widgets/counter/widget.toml
name = "Counter"     # required, shown in the UI
version = "0.1.0"    # optional
entry = "main.lua"   # optional, defaults to main.lua
```

The entry file returns a table with a plain-data `state`, a `render(state)` that returns a UI tree, and an optional `nav(state)` for the nav card. In a UI tree, hash keys are props and array entries are children. The builders are `vstack`, `hstack`, `text`, `button`, and `border`.

```lua
return {
  state = { count = 0 },
  render = function(state)
    return vstack {
      spacing = 12,
      text('clicks: ' .. state.count),
      button('Click me', function()
        state.count = state.count + 1
      end),
    }
  end,
}
```

That is the whole loop. The count lives in `state`, the button mutates it, and the host re-renders. The shipped `widgets/counter/` widget is the same idea with a `nav` preview added, so read it for a fuller example. Keep in mind that a `nav` tree is display-only: a `button` inside one is an error, because nav previews have no callbacks.

## Project layout

```
src/
  main.rs         thin entry point, Windows or stub
  shell.rs        single-instance guard, tray, layout, nav
  window.rs       reactor window, handle caching, subclass
  widget/
    mod.rs
    package.rs     finds and loads widgets from disk
    runtime.rs     sandboxed Lua VM and the render contract
widgets/
  counter/        the first shipped widget
    widget.toml
    main.lua
scripts/
  winrun.py       cross-build, stage the runtime, run
docs/
  reactor-notes.md
openspec/         spec-driven change workflow
```

## OpenSpec workflow

Changes are planned before they are coded. `openspec/specs/` holds the current capability specs, `openspec/changes/` holds proposals in flight, and `openspec/changes/archive/` holds the ones that shipped. Each archived proposal explains the why behind a piece of the app, so it is a good place to look when you want history rather than the current state. Run the workflow through `yarn openspec` or the OpenSpec skills.

## License

MIT. See [LICENSE](LICENSE).
