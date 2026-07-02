## Why

The config view's **Reload widget(s)** button exists so a developer can edit a widget's Lua and see the change without restarting the app. The button itself works: it rebuilds a fresh `mlua` VM and re-reads the entry file from disk. The problem is *which* file it reads. `widget-package` resolves `widgets/` beside the running exe — on a `winrun.py` dev run that is the **staged copy** in the run dir (`%USERPROFILE%/PrismaticTools/run/widgets`), which `stage()` writes only on a full build-stage-run. The developer edits the repo's git-tracked `widgets/` source (on WSL); the running Windows app reads the untouched NTFS staged copy; Reload re-reads that unchanged copy, so nothing appears to change. (Confirmed empirically: editing the *staged* copy directly and clicking Reload works — the gap is only that repo edits never reach the staged copy mid-session.)

## What Changes

- Add a copy-only `--sync-widgets` mode to `scripts/winrun.py` (dev-only): reuse `win_run_dir()` to locate the run dir, then mirror the repo `widgets/` folder into `<run-dir>/widgets` (wipe + copytree, same as `stage()`'s widget step) — **no cargo build, no runtime staging, no app launch**. It is its own branch in `main()` that early-returns before the build/stage/launch path.
- Restructure the `package.json` dev scripts so `yarn dev` runs two watchers concurrently:
  - `dev:app` — today's `nodemon -w "./src" -e "*" -x "yarn win"` (relaunch on `src/` change), unchanged.
  - `dev:widgets` — `nodemon -w "./widgets" -e "*"` in **run-on-change-only** mode, executing `winrun.py --sync-widgets` (copy-only) on a widget edit.
  - `dev` — `concurrently` running both.
- Add `concurrently` as a pinned devDependency.
- Update the CLAUDE.md Commands section to describe the new two-watcher `yarn dev`.

Dev loop after the change: edit repo widget → `dev:widgets` mirrors it into the run dir (no relaunch) → click in-app **Reload** → app re-reads the updated staged copy.

## Capabilities

### New Capabilities

<!-- none — dev-only tooling change; introduces no product capability -->

### Modified Capabilities

<!-- none — product behavior is unchanged. The in-app reload action and its
`tool-surface` / `main-content-layout` requirements stay exactly as specified;
this change only makes them effective in the dev loop. Widget resolution stays
exe-relative (`widget-package` unchanged). -->

## Impact

- **Dev-only scripts / Node layer**: `scripts/winrun.py` (new `--sync-widgets` branch), `package.json` (`dev`/`dev:app`/`dev:widgets` scripts + `concurrently` devDependency), `CLAUDE.md` (Commands section).
- **No product code**: `src/` is untouched — no `cfg(windows)` code, no widget runtime, no reactor UI, no spec requirements change. Widget resolution stays beside the exe.
- **No new runtime dependencies**; `concurrently` is dev-only. There are no tests.

## Non-goals

- Not redirecting the running app to read the WSL repo over a `\\wsl.localhost` UNC share (rejected: unverified UNC reads, per-render `read_dir` cost, `require`-path fragility). The app keeps reading the fast NTFS staged copy.
- Not a file-watcher *inside* the app and not auto-reload — Reload stays a manual button press; the watcher only syncs files.
- Not relaunching the app on a widget edit — the sync is copy-only so window/tray state is preserved.
- Not changing production widget resolution, the `stage()` widget copy, or the in-app reload action.
