## MODIFIED Requirements

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
