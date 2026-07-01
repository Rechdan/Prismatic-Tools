## Context

`src/window.rs::run()` builds the reactor `App` with `.title()` + `.backdrop(Backdrop::Mica)` and calls `.render(root)`. No size constraint is set, so the `OverlappedPresenter` lets the user drag the window down to a near-zero size where the app header (`app-header` capability) and the hosted tool clip and stop being usable.

The pinned `windows-reactor` (`crates/libs/reactor`) already exposes the needed API:

- `App::inner_constraints(InnerConstraints) -> Self` (builder), with `InnerConstraints { min_width, min_height, max_width, max_height }` all `Option<f64>` in DIPs (`crates/libs/reactor/src/style.rs`).
- The host applies these via `OverlappedPresenter::SetPreferredMinimumWidth/Height` (and Max equivalents), converting DIPs → physical px at the current DPI and adding the measured non-client offset (`apply_constraints_for_window`, `crates/libs/reactor/src/host.rs`). The presenter clamps interactive resize at the OS level and re-applies on DPI change.

`SetPreferredMinimum*` lives on `IOverlappedPresenter3`, available on Windows 11 22H2+ — the platform floor the app already requires for Mica.

## Goals / Non-Goals

**Goals:**
- Prevent interactive resize below a static 800 × 600 DIP floor.
- Keep the floor DPI-correct (expressed in DIPs, clamped in physical px by reactor).
- Zero new dependencies; reuse reactor's existing constraint path.

**Non-Goals:**
- No maximum size, fixed size, or non-resizable window.
- No change to startup/initial size, flash-free launch, close-to-tray, or tray lifecycle.
- No content-driven or per-tool dynamic minimum.
- No manual `WM_GETMINMAXINFO` in the HWND subclass.

## Decisions

**Decision: Use reactor's `App::inner_constraints` rather than a `WM_GETMINMAXINFO` subclass.**
`src/window.rs` already subclasses the HWND (flash suppression + close-to-tray), so a `WM_GETMINMAXINFO` handler was the obvious alternative. Rejected: reactor's presenter path is purpose-built, handles the DIP→px conversion and non-client offset, and re-applies on DPI change — all of which a hand-rolled `WM_GETMINMAXINFO` handler would have to replicate (and `MINMAXINFO` is in physical px, forcing manual DPI math). Using the builder keeps `window.rs`'s subclass focused on the two behaviors it already owns and avoids duplicating platform logic.

**Decision: Static 800 × 600 DIP floor held in a module constant.**
A single named constant (e.g. `MIN_INNER_SIZE: (f64, f64) = (800.0, 600.0)`) in `src/window.rs` keeps the value discoverable and tunable in one place, next to the other window constants (`WINDOW_TITLE`, `WINUI_CLASS`). Alternative — inlining the literals at the builder call — was rejected as less discoverable.

**Decision: Set only `min_width`/`min_height`; leave max unset.**
`InnerConstraints` defaults max fields to `None`, so omitting them preserves free growth and maximize. Matches the Non-goals.

## Risks / Trade-offs

- [`SetPreferredMinimum*` needs `IOverlappedPresenter3` (Win11 22H2+)] → The app already requires 22H2+ for Mica, so no new platform floor; on the supported target the cast succeeds. Reactor's `apply_constraints_for_window` returns `Result`, so a failed cast surfaces as an error rather than a silent no-op.
- [Floor larger than a small display's work area] → 800 × 600 DIPs is modest and fits typical laptop displays; if a future target has a smaller work area the presenter still clamps to the requested minimum. Out of scope to special-case here.
- [DIP vs px confusion] → Constraints are DIPs; reactor converts to px per DPI. Keeping the constant in DIPs (not px) matches the reactor API and avoids per-monitor drift.
