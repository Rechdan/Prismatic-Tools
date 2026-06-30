## MODIFIED Requirements

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
