## Why

Today the tray icon's left- and right-click both open the same context menu, and that menu carries a redundant "Show / hide window" item alongside "Exit". The fastest, most conventional gesture for a tray app — a single left-click to toggle the window — does nothing useful. Splitting the two clicks (left-click toggles, right-click is just the menu) matches Windows convention and removes the redundant menu item.

## What Changes

- Left-clicking the tray icon toggles the main window (show if hidden, hide if shown) directly, without opening a menu.
- Right-clicking the tray icon opens the context menu, which now contains **only** "Exit".
- The "Show / hide window" menu item is **removed** (left-click replaces it).
- The context menu no longer opens on left-click.

This touches Windows-only (`cfg(windows)`) code — `src/shell.rs` tray wiring. No change to the cross-build/staging path (`scripts/winrun.py`) or the dev-only Node layer.

## Non-goals

- No change to the `WM_CLOSE` close-to-tray behavior, the single-instance guard, or the window/tool surface.
- No double-click handling, hover/tooltip changes, or additional menu items.
- No new dependencies; uses the `tray-icon` crate already present.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-presence`: The "Tray menu controls" requirement changes — toggling the window moves from a menu item to a left-click on the icon, and the context menu (right-click) is reduced to "Exit" only.

## Impact

- `src/shell.rs`: drop the "Show / hide window" `MenuItem` and its `MenuEvent` branch; disable menu-on-left-click on the `TrayIconBuilder`; add a `TrayIconEvent` handler that calls `window::toggle()` on a left click.
- Dependencies: `tray-icon` (already a dependency) — uses its `TrayIconEvent` API.
- No change to `src/window.rs` (the `toggle()` entrypoint is reused as-is).
