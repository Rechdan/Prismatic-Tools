## ADDED Requirements

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
