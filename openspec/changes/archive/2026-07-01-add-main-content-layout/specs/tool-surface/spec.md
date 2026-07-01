## MODIFIED Requirements

### Requirement: Host renders a tool inside the window

The shell SHALL render exactly one loaded widget's UI inside the right container
of the main region's persistent two-pane layout (see the `main-content-layout`
capability), whenever the config view is not active, establishing the host→tool
rendering surface. The tool is a Lua widget loaded from the `widgets/` folder
(see the `widget-package` and `widget-runtime` capabilities); no hardcoded tool
remains.

#### Scenario: Loaded widget is visible

- **WHEN** the main window is shown and the config view is not active
- **THEN** the loaded widget's UI is rendered inside the right container of the
  two-pane layout using native Reactor controls
