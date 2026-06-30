# tray-presence Specification

## Purpose
TBD - created by archiving change bootstrap-tray-shell. Update Purpose after archive.
## Requirements
### Requirement: Background tray process

The application SHALL run as a background process that places an icon in the Windows system tray and SHALL NOT display any window on launch. No window SHALL be visible on screen at any point between process start and the user's first invocation from the tray — including no transient, partial, or single-frame appearance during window creation and activation.

#### Scenario: Launch shows tray icon, no window
- **WHEN** the application is started
- **THEN** an icon appears in the system tray
- **AND** no window is shown until the user invokes it

#### Scenario: No startup flash
- **WHEN** the application is started
- **THEN** the main window never becomes visible on screen before the user invokes it from the tray
- **AND** no transient window flash or partial frame is rendered during startup

#### Scenario: Process persists without window
- **WHEN** the main window is hidden or has never been opened
- **THEN** the process keeps running and the tray icon remains present

#### Scenario: First reveal shows the window normally
- **WHEN** the user first invokes the window from the tray after a flash-free launch
- **THEN** the window becomes visible and is brought to the foreground

### Requirement: Tray menu controls

Left-clicking the tray icon SHALL toggle the main window's visibility directly, without opening a menu. Right-clicking the tray icon SHALL open a context menu whose only item is "Exit", which terminates the application.

#### Scenario: Left-click toggles window
- **WHEN** the user left-clicks the tray icon
- **THEN** the main window is shown if hidden, or hidden if shown
- **AND** no context menu is opened

#### Scenario: Right-click opens Exit-only menu
- **WHEN** the user right-clicks the tray icon
- **THEN** a context menu opens containing only the "Exit" item

#### Scenario: Exit from tray menu
- **WHEN** the user selects the "Exit" menu item
- **THEN** the tray icon is removed and the process terminates cleanly

### Requirement: Closing the window hides it to the tray

The window's close affordance (the title-bar X / `WM_CLOSE`) SHALL hide the window to the tray rather than terminate the process. The process SHALL keep running with its tray icon present, and the window SHALL remain re-showable from the tray.

#### Scenario: Close button hides instead of exiting
- **WHEN** the user clicks the window's close (X) button
- **THEN** the window is hidden
- **AND** the process keeps running with the tray icon present

#### Scenario: Re-show after close
- **WHEN** the window has been closed to the tray
- **THEN** activating the tray's show/hide action shows the window again

#### Scenario: Only Exit terminates
- **WHEN** the window is closed to the tray
- **THEN** the application terminates only via the tray "Exit" item

### Requirement: Single running instance

The application SHALL allow only one running instance. A second launch SHALL detect the existing instance and exit without adding a duplicate tray icon.

#### Scenario: Second launch exits
- **WHEN** the application is already running
- **AND** the user launches it again
- **THEN** the second process exits without creating a second tray icon

