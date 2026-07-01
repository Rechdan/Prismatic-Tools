## MODIFIED Requirements

### Requirement: Minimum window size

The main window SHALL enforce a minimum interactive inner size of 1024 × 768 DIPs. The user SHALL NOT be able to resize the window smaller than this floor via edge or corner drags. The floor SHALL be applied through `windows-reactor`'s `App::inner_constraints` (`InnerConstraints { min_width, min_height, .. }`), which drives the window's `OverlappedPresenter` preferred-minimum size; no manual `WM_GETMINMAXINFO` handling is added.

#### Scenario: Drag smaller than the floor is clamped

- **WHEN** the user drags a window edge or corner toward a size below 1024 × 768 DIPs
- **THEN** the window stops shrinking at 1024 DIPs wide and 768 DIPs tall, so the app header and hosted tool remain fully visible

#### Scenario: Resize at or above the floor is unaffected

- **WHEN** the user resizes the window to any size at or above 1024 × 768 DIPs (including maximize)
- **THEN** the window resizes freely with no clamping

#### Scenario: Floor holds across DPI scaling

- **WHEN** the window is shown on a display whose DPI scale is not 100%
- **THEN** the 1024 × 768 DIP minimum is honored in device-independent terms (scaled to physical pixels for the current DPI), keeping the usable content area constant across monitors

## ADDED Requirements

### Requirement: Initial window size

The main window SHALL open at an initial interactive inner size of 1024 × 768 DIPs. The initial size SHALL be applied through `windows-reactor`'s `App::inner_size(width, height)` builder call, and SHALL be at least the enforced minimum so the window never opens below its floor. Setting the initial size SHALL NOT force the window to show at startup — the flash-free startup behavior (see the `tray-presence` capability) is preserved.

#### Scenario: Window opens at 1024 × 768

- **WHEN** the window is first revealed from the tray after launch
- **THEN** its interactive inner size is 1024 × 768 DIPs (unless the user or OS has since resized or maximized it)

#### Scenario: Initial size does not reintroduce a startup flash

- **WHEN** the application launches
- **THEN** setting the initial window size does not cause the window to paint before the first tray reveal (the window still starts hidden)
