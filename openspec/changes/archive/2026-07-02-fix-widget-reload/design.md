## Context

The **Reload widget(s)** action in the config view re-runs `widget::load_first()` into the shell's `use_ref`: a fresh `mlua` VM (in-memory state reset) and a fresh `fs::read_to_string` of the entry. It is correct. The defect is the path it reads.

`src/widget/package.rs::widgets_root()` resolves `widgets/` as `current_exe().parent().join("widgets")`. On a `winrun.py` dev run that is the **staged copy**: `stage()` wipes the run dir and copies the repo `widgets/` into it once per full build-stage-run, then launches the exe from there. The developer edits the repo source on WSL (`/home/nrechdan/Prismatic Tools/widgets/...`, git-tracked); the running Windows process reads the NTFS staged copy (`C:\Users\NRechdan\PrismaticTools\run\widgets\...`); the two never reconverge mid-session, so Reload re-reads unchanged bytes.

This was verified: editing the *staged* copy directly and clicking Reload reflects the change immediately — the read/reload mechanism is sound over local NTFS. The only missing link is getting repo edits into the run dir without a relaunch.

Constraints:
- Product code is `cfg(windows)`-only, cross-built from WSL; the Linux host stub does not compile `src/widget/`. This change touches **no** product code, so the stub stays green and there is nothing new to typecheck on the Windows target.
- Reads over a `\\wsl.localhost` UNC share are unverified here, and `count_widgets()` runs on every render (shell.rs) — so redirecting the app to read the WSL repo over UNC would add a per-render UNC `read_dir`. The local NTFS read path is already proven fast and correct.
- `winrun.py::win_run_dir()` is the single source of truth for the run-dir path (respects `PRISMATIC_RUN_DIR`, else `%USERPROFILE%/PrismaticTools/run`, returned as a WSL path via `wslpath -u`). Any sync must reuse it, not re-derive it.
- The current dev loop is `yarn dev` = `nodemon -w "./src" -e "*" -x "yarn win"`; it watches only `./src`, so widget edits trigger nothing today.

## Goals / Non-Goals

**Goals:**
- Make a repo widget edit reach the run dir the app reads, without a rebuild or relaunch, so an in-app Reload reflects it.
- Reuse existing run-dir logic; keep everything on local NTFS (fast, no UNC).
- Zero product-code and zero spec-requirement changes — a dev-tooling change only.

**Non-Goals:**
- No app-internal file-watcher / auto-reload; Reload stays a manual click.
- No relaunch on widget edit (preserve window/tray state).
- No redirect of the running app to the WSL repo over UNC.
- No change to production resolution, the `stage()` widget copy, or the reload action.

## Decisions

### Decision: A copy-only `--sync-widgets` mode in `winrun.py`

Add a branch in `main()` that, when `--sync-widgets` is present, calls `win_run_dir()`, wipes `<run-dir>/widgets`, and `copytree`s the repo `widgets/` there — then **returns before** the `cargo build`, `stage()`, and launch steps. It performs only the widget-copy portion of `stage()`, in isolation.

Details:
- **Mirror, not merge**: `shutil.rmtree(dest/widgets)` (if present) then `copytree`, so deleted/renamed widget files also disappear from the staged copy. The folder is tiny; cost is negligible.
- **Defensive**: `mkdir` the run dir / `widgets` parent if absent, so a sync that fires before the app has been launched (run dir not yet created) self-heals instead of erroring.
- **Guarded**: if the repo `widgets/` folder does not exist, no-op (mirrors `stage()`'s `isdir` guard).
- **No profile argument**: widgets are profile-independent; the mode needs only `win_run_dir()`.

**Why over alternatives:**
- *`PRISMATIC_WIDGETS_DIR` env override → app reads WSL repo over UNC*: unverified UNC reads, a per-render UNC `read_dir` via `count_widgets()`, and a `require` `package.path` carrying a UNC path with a space (`Prismatic Tools`). Rejected — trades a proven NTFS read for unproven UNC behavior.
- *Full restage + relaunch on widget edit (extend `dev` to `-w "./widgets"` → `yarn win`)*: works but runs cargo and relaunches the app on every save, losing window/tray state and defeating the point of an in-app reload. Rejected.
- *Edit the staged copy directly*: no code, but the staged copy is a build artifact — untracked and wiped on the next `yarn win`. Rejected.
- *Copy-only sync + manual Reload*: keeps the app on fast NTFS, no relaunch, edits stay in the git-tracked repo, and it reuses `win_run_dir()`. Chosen.

### Decision: Drive the sync from a second nodemon watcher, composed with `concurrently`

Restructure `package.json` scripts:
- `dev:app` = `nodemon -w "./src" -e "*" -x "yarn win"` (unchanged behavior; the src-relaunch loop).
- `dev:widgets` = `nodemon -w "./widgets" -e "*" --on-change-only -x "python3 scripts/winrun.py --sync-widgets"`.
- `dev` = `concurrently -n app,widgets "yarn dev:app" "yarn dev:widgets"`.
- Add `concurrently` as a pinned devDependency.

**Run-on-change-only on the widgets watcher** is load-bearing: nodemon otherwise runs its exec once at startup, so `dev:widgets`'s initial sync would race `dev:app`'s `yarn win` (whose `stage()` `rmtree`s the *entire* run dir) — concurrent wipe/copy on the same dir. With `--on-change-only`, the widgets watcher stays idle at launch (where `yarn win` already stages widgets fresh) and fires only on a real edit, after the app is up. The race is structurally impossible.

**Why `concurrently` over two terminals**: one `yarn dev` command runs both loops with prefixed, colorized output; the alternative (a separate `yarn dev:widgets` in a second terminal) works but is easy to forget to start. The dependency is dev-only.

## Risks / Trade-offs

- **Sync/read race** (user clicks Reload mid-copy → app reads a half-written file) → surfaces as a non-fatal visible widget error (the runtime never panics); click Reload again. Files are tiny, window is milliseconds.
- **Two-step UX** (save auto-syncs, but Reload is still a manual click) → accepted: it preserves the app's running state and matches the existing reload affordance; auto-reload was an explicit non-goal.
- **drvfs write latency** (WSL writing to the `/mnt/c` run dir) → same path `stage()` already uses every run; a handful of small files, negligible.
- **Startup sync before app launch** (run dir absent) → the defensive `mkdir` makes it a harmless recreate; the next full `yarn win` re-stages regardless.
- **`concurrently` masks a watcher crash** → both are simple long-running watchers; prefixed output makes a failed loop visible.

## Migration Plan

No product build or data migration. Land the `winrun.py --sync-widgets` branch, the `package.json` script/devDependency change, and the CLAUDE.md Commands update together. Verify: `yarn install` (pull `concurrently`); `yarn dev`; confirm the app launches once and does **not** relaunch on a widget edit; edit `widgets/counter/main.lua`, click in-app **Reload**, confirm the change shows. Rollback is reverting the three edits; `yarn win` and the in-app reload behave exactly as before.

## Open Questions

- None blocking. Whether to also offer `dev:widgets` standalone (second-terminal use) is a convenience, not a requirement; the `concurrently`-composed `yarn dev` is the primary path.
