## MODIFIED Requirements

### Requirement: Host renders a tool inside the window

The shell SHALL render exactly one loaded widget's UI inside the main window's content area, establishing the host→tool rendering surface. The tool is a Lua widget loaded from the `widgets/` folder (see the `widget-package` and `widget-runtime` capabilities); no hardcoded tool remains.

#### Scenario: Loaded widget is visible

- **WHEN** the main window is shown
- **THEN** the loaded widget's UI is rendered inside the window's content area using native Reactor controls

### Requirement: Tool interactivity proves the surface

The loaded widget SHALL include at least one interactive control whose state updates the rendered UI, proving the host→widget render/state loop works end to end.

#### Scenario: Interaction updates UI

- **WHEN** the user interacts with the loaded widget's control (e.g., a button)
- **THEN** the displayed state changes in response, confirming the render/state loop runs through the widget runtime
