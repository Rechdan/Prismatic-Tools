## 1. Trim the tray menu to Exit only

- [x] 1.1 In `src/shell.rs::run()`, remove the "Show / hide window" `MenuItem`, its `menu.append(&open)` call, and the `open_id` binding.
- [x] 1.2 Remove the `open_id` branch from the `MenuEvent` handler, leaving only the `quit_id` → `std::process::exit(0)` branch.

## 2. Left-click toggles the window

- [x] 2.1 Set `.with_menu_on_left_click(false)` on the `TrayIconBuilder` so left-click no longer opens the context menu.
- [x] 2.2 Add the imports for the confirmed (0.19.3) API: `use tray_icon::{TrayIconEvent, MouseButton, MouseButtonState};`.
- [x] 2.3 Register `TrayIconEvent::set_event_handler` that matches `TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }` and calls `window::toggle()`. Match `Up` only (a left-click also emits a `Down` event — matching both toggles twice).

## 3. Verify

- [x] 3.1 Keep the Linux host stub green: `cargo build` and `cargo test` succeed (tray code is `cfg(windows)`-gated, so this only confirms the stub still compiles).
- [x] 3.2 On Windows, run `python3 scripts/winrun.py` and verify: left-click toggles the window (no menu), right-click shows only "Exit", and "Exit" terminates the app.
