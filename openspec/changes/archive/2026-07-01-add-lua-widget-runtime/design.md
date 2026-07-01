## Context

Today `src/shell.rs::tool_surface()` renders one hardcoded Rust demo counter with reactor's `use_state`. Tools are meant to be added without recompiling, so a tool must be authored as a script that ships beside the exe and is loaded at runtime. This change stands up the first widget: Lua tools in a `widgets/` folder, and ports the demo counter to prove the loop.

Key constraints from the existing app:
- The product is `cfg(windows)`-only, cross-built for `x86_64-pc-windows-gnu` from WSL with the mingw toolchain (no MSVC). Any new dependency must cross-build under that target.
- Release builds use `panic = "abort"`; a panic unwinding into WinUI's C++ frames is UB. Lua/manifest errors therefore must be **caught and rendered**, never allowed to panic across the reactor boundary.
- `windows-reactor` is declarative and reactive: the render fn re-runs when reactor state changes. Verified against the checked-out reactor source: `vstack/hstack(impl IntoElements)` accept a `Vec<Element>` (dynamic child lists work); `on_click(impl IntoUnitCallback)` requires `Fn() + 'static` (stored as `Rc<dyn Fn()>`, no `Send`); `use_state<T>` requires `T: Clone + PartialEq + 'static`; `use_ref<T: 'static>` persists a value across renders with no `Send`/`Clone` bound.
- All reactor render + event handling runs on a single UI thread, so a `!Send` Lua VM is safe.

## Goals / Non-Goals

**Goals:**
- Embed a **sandboxed** Lua runtime in the Windows build and expose a small declarative UI vocabulary to Lua.
- Define a stable on-disk widget package layout (`widgets/<id>/widget.toml` + `main.lua`) resolved relative to the exe.
- Establish a `{ state, render }` widget contract where widget state is a plain-data Lua table that Rust holds a handle to (for future persistence), and translate a widget's `render(state)` table into reactor elements, dispatching events back to Lua and re-rendering when state changes.
- Load exactly one widget at startup and render it where the demo counter was; ship the counter as that widget.
- Keep the Linux host stub building green and keep the cross-build/staging path working (stage `widgets/` next to the exe).

**Non-Goals:**
- Discovering/loading more than one widget, or a picker to switch between them (follow-up).
- Actually serializing/saving/restoring widget state — this change builds only the *contract* (Rust holds the state handle); save/load is deferred.
- A complete UI vocabulary — only stack/text/button as needed for the counter; no text styling, padding, or other controls.
- Full sandboxing: execution-time/step limits and memory caps are out of scope (a widget can still hang the UI thread).

## Decisions

### D1: Lua runtime — `mlua` with vendored Lua 5.4, cross-build proven first
Embed via `mlua` (features `lua54`, `vendored`) under `[target.'cfg(windows)'.dependencies]`. `mlua` is the maintained successor to `rlua`, has an ergonomic conversion layer, and since 0.9 uses **owned, `'static` handles** (the `'lua` lifetime was dropped; `Lua`/`Function`/`Table` are refcounted to the VM) — which is what lets a Lua `Function` be captured directly in a reactor `Fn() + 'static` closure.

Because the whole change rests on this compiling, a **standalone cross-build spike runs before any real code** (task group 1): a throwaway `mlua` eval built via `winrun.py --no-run` that also confirms (a) the owned-handle model and (b) that the restricted-stdlib construction compiles.

- **Alternatives considered:** `rlua` (superseded); pure-Rust `piccolo`/`hematita` (no C toolchain risk but immature APIs); `luau` via `mlua`'s `luau` feature (same Rust API — the chosen fallback if vendored 5.4 won't link under mingw). At this vocabulary (tables, functions, numbers, strings) authors won't hit 5.4-vs-luau differences, so `luau` is an acceptable fallback; `piccolo` is a last resort that would force spec rewrites.

### D2: Declarative UI bridge — Lua returns a tagged table, Rust maps to reactor
The runtime injects Lua globals `vstack`, `hstack`, `text`, and `button`. They normalize their arguments into `kind`-tagged tables so Rust always parses a uniform node:
- **Encoding: hash keys are props, array entries are children.** `vstack{ spacing = 12, a, b }` → `{kind='vstack', spacing=12, children={a, b}}`.
- `text('...')` → `{kind='text', value='...'}`; `button('label', fn)` → `{kind='button', label='label', on_click=fn}` (positional; `on_click` optional).

On each reactor render the runtime calls `render(state)`, walks the returned table, and builds the reactor `Element` tree (`vstack`/`hstack` → stacks with `spacing` over a `Vec<Element>`; `text` → `text_block`; `button` → `button(label).on_click(...)`). An unrecognized `kind` renders a visible inline error node in that slot rather than panicking.

Shipping subset: `vstack`/`hstack`/`text`/`button`, `spacing` only. Text styling, padding, alignment, grid, and all other controls are deferred until a widget needs them. The app header (bold app name, window padding) stays **native Rust in `shell.rs`**; the widget renders plain body content below it.

- **Why declarative over an imperative `ui.*` API:** Lua never holds reactor handles or participates in reactor's hook ordering; the boundary is one `render(state) -> table` call plus event callbacks.

### D3: State contract + reactive glue
The widget's Lua chunk **returns a module table** with two fields:
- `state` — a **plain-data** table (nil/bool/number/string/nested tables), the source of truth for widget state. Optional; defaults to `{}`.
- `render` — a function `render(state) -> UI table`. Required.

Load is one-time: execute the chunk once, then hold **owned handles** to the module table, the `state` table, and the `render` function. Rust keeps the `state` handle specifically so a future persistence/config feature can serialize it — that handle is the whole point of the contract; **no serialization is built this change.**

Each reactor render calls `render(state)` passing the Rust-held `state` table (so Rust's serializable handle and render's view are the same table), then maps the result (D2).

Reactive glue (grounded in the reactor API):
- The VM + handles live in `cx.use_ref(None::<LoadedWidget>)`, filled on first render (`use_ref` needs no `Send`/`Clone`, fitting `!Send` `Lua`; its arg is eager, so we store `None` and lazily load once).
- One `cx.use_state(0u32)` `tick` cell is the re-render trigger.
- Building a `button`, its `on_click` is captured as an **owned `mlua::Function`** alongside a clone of the tick `SetState` in the reactor closure. On click: call the Lua function (which mutates the `state` table), then bump `tick` → reactor re-runs the render fn → `render(state)` reflects the new state.
- Callbacks are re-extracted every render (fresh owned `Function` handles; previous ones dropped), so no stale references accumulate. Everything runs on the UI thread.

- **Alternative considered:** state opaque in Lua with Rust oblivious (lighter, but forecloses persistence). Rejected per the persistence/config goal — paying the small contract cost now so persistence is a later drop-in, not a bridge rewrite.

### D4: Sandboxed Lua VM
Each widget gets a VM opened with **safe libraries only**: `base`, `table`, `string`, `math`, `coroutine`, `utf8`, and a **hardened `package`**. `os`, `io`, and `debug` are not opened. Additionally: base `dofile`/`loadfile` are removed; `package.path` is set to `"<widget folder>/?.lua"`; `package.cpath = ""` and the native/C searcher + `package.loadlib` are removed. `load` (string compile, no filesystem) is kept.

Net: a widget can compute, build UI, and `require` its own sibling `.lua` files, but cannot touch the filesystem, run processes, load native code, or `require` outside its folder.

- **Why now, given sandboxing is a non-goal:** which libs are opened is a default we set regardless; opening the safe set from day one is cheap and establishes the boundary before third-party widgets exist. The deeper sandbox work (exec-time/step/memory limits) stays deferred.

### D5: Widget package layout + manifest
`widgets/<id>/` contains `widget.toml` and the Lua entry. **The folder name is the widget id** — there is no `id` field. The manifest (parsed with the pure-Rust `toml` crate) declares `name` (required — the future-metadata payload), `version` (optional), and `entry` (optional, default `main.lua`). `widget.toml`'s presence is what marks a folder as a widget.

- **Why a manifest over convention-only:** an explicit `name`/`version` gives the future picker/config surfaces real metadata; `entry` leaves room for multi-file widgets. Dropping `id` removes a redundant field and its mismatch failure mode (folder name is the single source of truth).

### D6: Widget selection + failure surface
The runtime resolves `widgets/` relative to `std::env::current_exe()`'s directory and loads the **first widget folder (sorted)**; extra folders are ignored with no on-screen note (the selection logic is replaced when multi-widget discovery lands). The **app header (app name + `Configs` + `GitHub`) is always native and always visible** — a broken widget only replaces the content area below it.

Failure granularity:
- Missing `widgets/` dir, zero widgets, missing/invalid manifest, missing entry file, or a Lua error during load or `render()` → **one error/empty message** in the content area.
- An unrecognized node *kind* mid-tree → an **inline error node** in that slot; the rest of the tree still renders.

Nothing in the load/render/dispatch path may `panic!` (release aborts into WinUI).

## Risks / Trade-offs

- **`mlua` vendored C fails to cross-compile under `x86_64-w64-mingw32`** → the standalone spike (D1) proves it before real code; fall back to `mlua`'s `luau` feature, then `piccolo`. Isolate the runtime behind a thin module so a swap is contained.
- **Owned-handle assumption wrong** → if `mlua` handles aren't `'static`/owned, the callback path falls back to a `RegistryKey` + index indirection; the spike verifies this up front.
- **Unsandboxed execution** → a widget can `while true do end` and hang the UI thread, or exhaust memory; no limits this change (documented, future work). The lib sandbox (D4) blocks fs/process/native access but not runaway compute.
- **State must be serializable to persist** → only plain-data lives in `state`; functions/userdata there won't serialize later. This is a documented constraint on widget authors, and the persistence boundary is exactly the `state` table.
- **Lua state mutation doesn't repaint** → the explicit `tick` `use_state` bump after every callback is the single re-render source; UI-only widgets with no callbacks simply never bump.
- **Holding Lua callbacks across renders leaks/dangles** → re-extract owned `Function` handles every render and drop the previous set; never store them in long-lived Rust state.
- **Staging drift** → `winrun.py::stage()` wipes and rebuilds the run dir each run, so it must copy `widgets/` every time; the exe-relative resolver then finds it identically.

## Migration Plan

Additive and isolated. The only behavior swap is `tool_surface()` moving from the hardcoded counter to rendering the loaded widget below the (unchanged) native header. Rollback = revert that swap and the new module/deps; the header, tray, window lifecycle, and their specs are untouched. Verification is Windows-only via `python3 scripts/winrun.py` (build + stage `widgets/` + run, click the counter, exercise the sandbox and error paths).

## Open Questions

- ~~Final runtime crate is contingent on the D1 cross-build spike (Lua 5.4 vendored vs `luau` vs `piccolo`).~~ **Resolved:** the task-1.2 spike proved `mlua` 0.10 (`lua54`, `vendored`) compiles its vendored Lua C via mingw and links clean under `x86_64-pc-windows-gnu`; owned `Function` handles capture into `Fn() + 'static` closures. No `luau`/`piccolo` fallback needed.
- Serialization format for the eventual persistence feature (TOML vs JSON) — deferred; only the `state` handle is established now.
- Long-term: how per-widget config and the `Configs` button relate to widget manifests and the `state` table (out of scope here).
