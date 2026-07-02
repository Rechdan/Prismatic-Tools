## 1. Copy-only sync mode (`scripts/winrun.py`)

- [x] 1.1 Add a `--sync-widgets` branch at the top of `main()` that early-returns before the `cargo build` / `stage()` / launch path (so it can never rebuild or relaunch).
- [x] 1.2 In that branch, call `win_run_dir()` to locate the run dir (reuse — do not re-derive the path), then mirror repo `widgets/` into `<run-dir>/widgets`: `mkdir` the run dir if absent, `shutil.rmtree(<run-dir>/widgets)` if present, then `shutil.copytree` the repo `widgets/`.
- [x] 1.3 Guard on the repo `widgets/` existing (mirror `stage()`'s `os.path.isdir` check); no-op with a clear printed message when it does not.
- [x] 1.4 Update the module docstring / `Usage` line to document `--sync-widgets` (copy-only; no build, no launch).

## 2. Dev watchers (`package.json`)

- [x] 2.1 Rename the current `dev` script to `dev:app` (`nodemon -w "./src" -e "*" -x "yarn win"`), unchanged behavior.
- [x] 2.2 Add `dev:widgets`: `nodemon -w "./widgets" -e "*" --on-change-only -x "python3 scripts/winrun.py --sync-widgets"` (run-on-change-only, so it does not sync at startup).
- [x] 2.3 Redefine `dev` as `concurrently -n app,widgets "yarn dev:app" "yarn dev:widgets"`.
- [x] 2.4 Add `concurrently` as a pinned devDependency and run `yarn install` so the lockfile updates. (`concurrently@10.0.3`, exact-pinned.)

## 3. Docs

- [x] 3.1 Update the CLAUDE.md Commands section: `yarn dev` now runs the app-relaunch watcher and the widget-sync watcher concurrently (the latter copy-only, paired with the in-app Reload).

## 4. Verification

- [x] 4.1 Keep the Linux host stub green: `cargo build` and `cargo test` still succeed (this change touches no `src/`, so nothing new to compile — confirm no regression). ✓ both green.
- [x] 4.2 Windows-only end-to-end via `yarn dev` (drives `python3 scripts/winrun.py`): the app builds and launches once; editing `widgets/counter/main.lua` triggers a `--sync-widgets` copy and does **not** relaunch the app. ✓ confirmed on Windows.
- [x] 4.3 Windows-only: after a widget edit syncs, click the in-app **Reload** and confirm the change is reflected without restarting the app. ✓ confirmed on Windows.
- [x] 4.4 Windows-only: confirm a widget file *deleted* in the repo is also removed from `<run-dir>/widgets` after the next sync (mirror semantics), and that a sync fired before the app is launched creates `<run-dir>/widgets` without error. ✓ verified headless via `PRISMATIC_RUN_DIR` (`win_run_dir()` skips Windows interop when it is set): absent-dir create, orphan/rename removal, and staged==repo all pass.
