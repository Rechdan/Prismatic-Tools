# themed-window Specification

## Purpose
TBD - created by archiving change bootstrap-tray-shell. Update Purpose after archive.
## Requirements
### Requirement: Windows 11 themed window

The main window SHALL be created with `windows-reactor` using a Mica system backdrop and Windows 11 rounded styling.

#### Scenario: Window renders with Mica backdrop
- **WHEN** the main window is shown on Windows 11 (22H2 or later)
- **THEN** the window displays a Mica backdrop and rounded corners consistent with native Windows 11 windows

### Requirement: OS light/dark theme match

The window SHALL match the operating system's light or dark app theme at startup, using Reactor's `ThemeRef` brushes for foreground, background, and accent surfaces.

#### Scenario: Dark OS theme
- **WHEN** the OS app theme is dark at the time the window is created
- **THEN** the window and its controls render with the dark theme

#### Scenario: Light OS theme
- **WHEN** the OS app theme is light at the time the window is created
- **THEN** the window and its controls render with the light theme

### Requirement: Window fidelity rung selected by spike

The window's presentation mode (borderless anchored flyout that hides on deactivation, versus a plain toggled Mica window) SHALL be chosen by the Phase 0 spike's findings on `presenter`/HWND access. The change SHALL ship at least the plain toggled Mica window rung.

#### Scenario: Fallback rung always available
- **WHEN** borderless/HWND access is unavailable from `windows-reactor`
- **THEN** the application ships a normal small Mica window toggled from the tray, and the flyout behavior is deferred

#### Scenario: Flyout rung when access available
- **WHEN** the spike confirms borderless and window-positioning access
- **THEN** the window is presented as a borderless flyout that hides when it loses focus

### Requirement: Minimum window size

The main window SHALL enforce a minimum interactive inner size of 800 × 600 DIPs. The user SHALL NOT be able to resize the window smaller than this floor via edge or corner drags. The floor SHALL be applied through `windows-reactor`'s `App::inner_constraints` (`InnerConstraints { min_width, min_height, .. }`), which drives the window's `OverlappedPresenter` preferred-minimum size; no manual `WM_GETMINMAXINFO` handling is added.

#### Scenario: Drag smaller than the floor is clamped

- **WHEN** the user drags a window edge or corner toward a size below 800 × 600 DIPs
- **THEN** the window stops shrinking at 800 DIPs wide and 600 DIPs tall, so the app header and hosted tool remain fully visible

#### Scenario: Resize at or above the floor is unaffected

- **WHEN** the user resizes the window to any size at or above 800 × 600 DIPs (including maximize)
- **THEN** the window resizes freely with no clamping

#### Scenario: Floor holds across DPI scaling

- **WHEN** the window is shown on a display whose DPI scale is not 100%
- **THEN** the 800 × 600 DIP minimum is honored in device-independent terms (scaled to physical pixels for the current DPI), keeping the usable content area constant across monitors

