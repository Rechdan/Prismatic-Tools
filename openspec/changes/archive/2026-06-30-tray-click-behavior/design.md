## Context

The tray is wired in `src/shell.rs::run()` using the `tray-icon` crate. Today a `Menu` with two items ("Show / hide window", "Exit") is attached via `TrayIconBuilder::with_menu(...)`, and a single `MenuEvent::set_event_handler` dispatches on the item ids: the open id calls `window::toggle()`, the quit id calls `std::process::exit(0)`. Left- and right-click both surface this menu, because `tray-icon` opens the context menu on left-click by default. The window's `toggle()` Win32 entrypoint already exists in `src/window.rs` and is reused unchanged.

The goal is to make a left-click toggle the window directly and reduce the right-click menu to "Exit" only.

## Goals / Non-Goals

**Goals:**
- A single left-click on the tray icon toggles the window with no menu flash.
- The right-click context menu contains only "Exit".
- Keep the change confined to `src/shell.rs`; reuse `window::toggle()` as-is.

**Non-Goals:**
- No double-click handling, hover, or tooltip changes.
- No change to `WM_CLOSE` close-to-tray, the single-instance guard, or the window/tool surface.
- No new dependencies.

## Decisions

**Disable menu-on-left-click and handle the click event ourselves.**
`tray-icon`'s `TrayIconBuilder` opens the attached menu on left-click by default (`menu_on_left_click` defaults to `true`, confirmed in 0.19.3 source). Set `.with_menu_on_left_click(false)` so a left-click no longer pops the menu and instead is delivered as a `TrayIconEvent`. The right-click menu is unaffected: in the source the menu shows on `WM_RBUTTONDOWN` **unconditionally** (the flag only gates the left-button-down path), so right-click keeps opening the context menu. Alternative considered: keep menu-on-left-click and just trim the menu — rejected because it cannot give a one-click toggle.

**Toggle from a `TrayIconEvent::Click` handler, filtered to left button + up.**
Register `TrayIconEvent::set_event_handler` alongside the existing `MenuEvent` handler. Match `TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }` and call `window::toggle()`. Confirmed against the locked `tray-icon` 0.19.3 source: a left-click emits `Click` on **both** `WM_LBUTTONDOWN` (`button_state: Down`) and `WM_LBUTTONUP` (`button_state: Up`), so filtering on `Up` is required — without it the handler fires twice and the window toggles twice (net no-op). Both handlers run on the UI thread (reactor pumps the tray's hidden window on the same single thread), so the Win32 `toggle()` call is valid from there — same thread-safety basis as the current menu handler.

**Trim the menu to "Exit" only.**
Remove the "Show / hide window" `MenuItem`, its `open_id`, and the `open_id` branch in the `MenuEvent` handler. The menu keeps only the "Exit" item and its existing quit branch.

## Risks / Trade-offs

- [Discoverability: left-click toggle is not labeled anywhere] → Conventional for tray apps; the tooltip ("Prismatic Tools") already identifies the icon, and Exit remains visible on right-click.
- [Left-click delivers down and up events] → Match only `button_state == MouseButtonState::Up` so one click toggles once. (Verified against 0.19.3 source.)
- [Double-clicking the tray emits two `Up` events → toggles twice] → Accepted: the net effect is a no-op (window returns to its prior state); not worth coordinating click/double-click timing. `DoubleClick` events are ignored.
- [Window minimized or buried behind other windows] → Accepted limitation of the existing 2-state `window::toggle()` (`IsWindowVisible`-based): a minimized/buried window is "visible", so one click hides it and a second restores it. Out of scope for this change; a 3-state toggle would be a separate `window.rs` change.
- [Cannot validate on the Linux host] → Tray code is `cfg(windows)`-only; verify via `python3 scripts/winrun.py` on Windows.

## Migration Plan

Single in-place edit to `src/shell.rs`; no data, config, or dependency migration. Rollback is reverting the commit.

## Open Questions

- None. The `TrayIconEvent::Click` shape is confirmed against the locked `tray-icon` 0.19.3 source: `Click { id, position, rect, button: MouseButton, button_state: MouseButtonState }`, with `MouseButton::{Left,Right,Middle}` and `MouseButtonState::{Up,Down}`.
