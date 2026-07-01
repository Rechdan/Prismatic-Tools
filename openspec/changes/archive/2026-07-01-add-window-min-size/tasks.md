## 1. Apply the minimum-size constraint

- [x] 1.1 In `src/window.rs`, add a module constant for the floor next to `WINDOW_TITLE`/`WINUI_CLASS` (e.g. `const MIN_INNER_SIZE: (f64, f64) = (800.0, 600.0);` — DIPs, width × height).
- [x] 1.2 Import `InnerConstraints` from `windows_reactor` (via the existing `use windows_reactor::*;` glob, or an explicit path if the glob does not re-export it). — Confirmed `pub use style::*` re-exports it; existing glob covers it, no new import needed.
- [x] 1.3 In `run()`, chain `.inner_constraints(InnerConstraints { min_width: Some(MIN_INNER_SIZE.0), min_height: Some(MIN_INNER_SIZE.1), ..Default::default() })` onto the `App` builder (before `.render(root)`), leaving `max_width`/`max_height` as `None`.

## 2. Verify

- [x] 2.1 `cargo build` — confirm the Linux host stub still compiles green (the change is `cfg(windows)`-gated; the stub is unaffected but must stay building). — Also ran `cargo check --target x86_64-pc-windows-gnu`: the actual `window.rs` edit compiles clean.
- [x] 2.2 Windows-only: run `python3 scripts/winrun.py`, then drag a window edge/corner inward — confirm the window stops shrinking at 800 × 600 DIPs and the app header + demo tool stay fully visible. — Confirmed on Windows.
- [x] 2.3 Windows-only: confirm resizing at/above the floor and maximize are unaffected, and (if a non-100% DPI display is available) that the floor holds in DIP terms across monitors. — Confirmed on Windows.
