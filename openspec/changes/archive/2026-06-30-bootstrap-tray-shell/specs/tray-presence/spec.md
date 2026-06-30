## ADDED Requirements

### Requirement: Background tray process

The application SHALL run as a background process that places an icon in the Windows system tray and SHALL NOT display any window on launch.

#### Scenario: Launch shows tray icon, no window
- **WHEN** the application is started
- **THEN** an icon appears in the system tray
- **AND** no window is shown until the user invokes it

#### Scenario: Process persists without window
- **WHEN** the main window is hidden or has never been opened
- **THEN** the process keeps running and the tray icon remains present

### Requirement: Tray menu controls

The tray icon SHALL expose a menu that lets the user toggle the main window's visibility and exit the application.

#### Scenario: Toggle window from tray
- **WHEN** the user activates the tray icon (click or the "Open" menu item)
- **THEN** the main window is shown if hidden, or hidden if shown

#### Scenario: Exit from tray
- **WHEN** the user selects the "Exit" menu item
- **THEN** the tray icon is removed and the process terminates cleanly
