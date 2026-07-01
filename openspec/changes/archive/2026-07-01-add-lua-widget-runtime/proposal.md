## Why

The tool surface currently renders one hardcoded Rust demo counter — there is no way to add tools without recompiling the app. Prismatic Tools is meant to host many small tools, so tools need to be authored as data/scripts that ship beside the exe. This change introduces the first widget: tools written in Lua, loaded from a `widgets/` folder, starting by porting the existing demo counter to prove the runtime end to end.

## What Changes

- Embed a **sandboxed** Lua runtime in the Windows app (new `cfg(windows)` dependency; the Linux host stub stays untouched and green). The VM opens safe libraries only (no `os`/`io`/`debug`), and `require` is scoped to the widget's own folder with no native-module loading.
- Define an on-disk **widget package** layout: a `widgets/` folder next to the exe, one folder per widget (the folder name is the widget id), each with a `widget.toml` manifest (`name` required; `version`/`entry` optional, `entry` defaulting to `main.lua`) plus its Lua entry file.
- Establish a **`{ state, render }` widget contract**: a widget's Lua chunk returns a table with a plain-data `state` table and a `render(state)` function. Widget state lives in Lua as the source of truth; Rust holds a handle to the `state` table so a **future** persistence/config feature can serialize it (this change builds the contract only — no save/load).
- Add a **declarative UI bridge**: `render(state)` returns a UI-description table (`vstack`/`hstack`/`text`/`button`, hash keys = props, array entries = children). Rust translates that tree into `windows-reactor` elements each render, dispatches control events back to Lua callbacks, and re-renders when state changes. The widget never touches reactor internals.
- Load exactly **one** widget at startup (the first folder, sorted) and render it in the content area below the persistent native app header, replacing the hardcoded Rust demo counter.
- Port the existing demo counter to a Lua widget (`widgets/counter/`) as the first widget and the runtime's proof.
- Prove the runtime cross-builds **before** implementing the bridge: a standalone `x86_64-pc-windows-gnu` build spike for the Lua dependency (gate on the whole change).
- Stage the repo's `widgets/` folder into the run directory next to the exe in `scripts/winrun.py` so the app finds it at runtime (dev-only scripts layer).

## Capabilities

### New Capabilities
- `widget-package`: the on-disk widget layout — an exe-relative `widgets/` folder, one folder per widget (folder name = id), a `widget.toml` manifest schema, and resolving + reading a widget's Lua entry from disk.
- `widget-runtime`: embedding a sandboxed Lua VM, the `{ state, render }` contract, exposing the declarative UI vocabulary to Lua, translating `render(state)` into reactor elements, dispatching events to Lua callbacks, and driving each widget's re-render loop.

### Modified Capabilities
- `tool-surface`: the host now renders a **loaded Lua widget** in the content area instead of a hardcoded Rust demo tool; the interactivity requirement is satisfied by the ported Lua counter rather than native Rust state.

## Impact

- **Windows-only (`cfg(windows)`) code**: new Lua runtime dependency (`mlua`, vendored) and `toml` in `Cargo.toml`; new module(s) for widget loading + the sandboxed Lua↔reactor bridge; `src/shell.rs` `tool_surface()` switches from the hardcoded counter to rendering the loaded widget below the unchanged native header.
- **Cross-build/staging path (`scripts/winrun.py`)**: `stage()` copies the repo `widgets/` folder into the run dir next to the exe. Cross-compiling the Lua runtime for `x86_64-pc-windows-gnu` (vendored C built with the mingw toolchain) is a build risk, proven by a standalone spike before other work.
- **Repo content**: new top-level `widgets/counter/` (manifest + `main.lua`) as the first shipped widget.
- **Dev-only Node layer**: unaffected (no JS app code; nodemon/openspec unchanged).

## Non-goals

- Multi-widget discovery, loading more than one widget, or a widget picker/navigation UI (a follow-up change; this loads exactly one widget).
- Actually serializing/saving/restoring widget state — only the `state`-handle contract is built; save/load and its format are deferred.
- A full UI vocabulary — only stack/text/button (with `spacing`) are in scope; text styling, padding, and other controls come as widgets need them.
- Full sandboxing: the VM restricts libraries and `require` scope, but there are no execution-time/step or memory limits (a widget can still hang the UI thread).
- Per-widget config surfaces, hot-reload, or installing widgets from outside the repo.
- Any change to tray behavior, window lifecycle, flash-free startup, the app header, or the minimum window size.
