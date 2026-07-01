## 1. Prove the runtime cross-builds (standalone gate — before any real code)

- [x] 1.1 Add `mlua` (`features = ["lua54", "vendored"]`) under `[target.'cfg(windows)'.dependencies]` in `Cargo.toml`. The Linux host stub build (`cargo build`/`cargo test`) stays green because the dep is `cfg(windows)`-gated.
- [x] 1.2 Throwaway spike, built via `python3 scripts/winrun.py --no-run`, proving under `x86_64-pc-windows-gnu` (mingw): (a) `mlua`'s vendored Lua links; (b) handles are owned/`'static` (capture a cloned `Function` in a `Fn() + 'static` closure and call it); (c) the restricted-stdlib construction compiles (open `base`/`table`/`string`/`math`/`coroutine`/`utf8`/`package`; not `os`/`io`/`debug`). **Windows-only build check.** If linking fails, switch to `mlua`'s `luau` feature before proceeding; `piccolo` only if both fail. — **Passed via `cargo build --target x86_64-pc-windows-gnu` (mlua-sys vendored C linked clean); no fallback needed.**
- [x] 1.3 Remove the spike code once green; record the outcome (crate/dialect chosen) in the design's Open Questions.

## 2. Widget package loading (`widget-package`)

- [x] 2.1 Add the pure-Rust `toml` dependency (cross-builds cleanly) for manifest parsing.
- [x] 2.2 New module (e.g. `src/widget/package.rs`, `cfg(windows)`): a `Manifest` struct (`name` required; `version`/`entry` optional, `entry` default `main.lua`) parsed from `widget.toml`. The widget id is the folder name — no id field.
- [x] 2.3 Resolve `widgets/` relative to `std::env::current_exe()`'s directory, enumerate subfolders, and select the first (sorted). No `widgets/` dir or zero folders → a typed "no widget" outcome (rendered as a message, not a crash).
- [x] 2.4 Read the resolved Lua entry source, returning typed, non-panicking load errors for: missing/unparseable `widget.toml`, missing required field, missing entry file.

## 3. Sandboxed Lua VM (`widget-runtime`)

- [x] 3.1 Construct the per-widget Lua opening safe libs only (`base`, `table`, `string`, `math`, `coroutine`, `utf8`, `package`); do not open `os`, `io`, `debug`.
- [x] 3.2 Harden the VM: remove base `dofile`/`loadfile`; set `package.path` to `"<widget dir>/?.lua"`; set `package.cpath = ""` and drop the native/C searcher + `package.loadlib`. Verify `require` resolves a sibling `.lua` and cannot escape the folder or load native modules.

## 4. Widget module contract + load (`widget-runtime`)

- [x] 4.1 Inject UI builder globals `vstack`/`hstack`/`text`/`button` that return `kind`-tagged tables (hash keys → props incl. `spacing`; array entries → `children`).
- [x] 4.2 Execute the widget chunk once; require it returns a table with `render` (function, required) and `state` (plain-data table, optional → default empty). Hold owned handles to the module, the `state` table, and `render`. Missing `render` or load errors → a rendered message.

## 5. Render + event bridge (`widget-runtime`)

- [x] 5.1 Each render: call `render(state)` with the held `state` table; walk the returned tagged table into a reactor `Element` tree (`vstack`/`hstack` → stacks with `spacing` over `Vec<Element>`; `text` → `text_block`; `button` → `button().on_click`). An unrecognized `kind` → an inline error node (rest of tree still renders).
- [x] 5.2 For each button, capture its `on_click` as an owned `mlua::Function`; the reactor `Fn() + 'static` closure calls it (mutating `state`), then bumps a reactor `tick` `use_state` to force re-render. Re-extract callbacks every render (drop prior handles).
- [x] 5.3 Persist the VM + handles across renders via `cx.use_ref(None)` filled on first render; the re-render `tick` via `cx.use_state(0u32)`.
- [x] 5.4 Catch Lua errors from `render(state)` and from callbacks and render error text; nothing in the load/render/dispatch path may `panic!` (release aborts into WinUI).

## 6. Host integration (`tool-surface`)

- [x] 6.1 Change `src/shell.rs::tool_surface()` to load the first widget and render it in the content area below the app header, replacing the hardcoded counter. The header (app name + `Configs` + `GitHub`) stays native and always visible.
- [x] 6.2 Non-fatal fallbacks: no `widgets/` dir, zero widgets, or any load/render error renders a single visible message in the content area (header intact), never a crash.

## 7. First widget + staging

- [x] 7.1 Add `widgets/counter/widget.toml` (`name = "Counter"`, a `version`, `entry = "main.lua"`) and `widgets/counter/main.lua` returning `{ state = { count = 0 }, render = function(state) ... end }` — a `text` showing `state.count` and a `button` that increments it.
- [x] 7.2 Update `scripts/winrun.py::stage()` to copy the repo `widgets/` folder into the run dir next to the exe (it wipes/rebuilds the dir each run, so copy every stage).

## 8. Verify

- [x] 8.1 `cargo build` and `cargo test` on the Linux host stay green (host stub still compiles; no Windows assumptions leaked into non-gated code).
- [x] 8.2 **Windows-only** (`python3 scripts/winrun.py`): the window shows the counter below the header, clicking increments it, and the count survives across re-renders — matches the `tool-surface` and `widget-runtime` scenarios. _(Confirmed on Windows.)_
- [x] 8.3 **Windows-only** sandbox check: a widget referencing `os`/`io` gets nil (blocked); `require` of a sibling `.lua` works while requiring outside the folder or a native module fails.
- [x] 8.4 **Windows-only** error paths: break the manifest (missing field) and raise a Lua error; confirm a visible message with the header intact and no crash (design D6).
