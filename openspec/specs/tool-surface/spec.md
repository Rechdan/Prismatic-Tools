# tool-surface Specification

## Purpose
TBD - created by archiving change bootstrap-tray-shell. Update Purpose after archive.
## Requirements
### Requirement: Host renders a tool inside the window

The shell SHALL render exactly one tool's UI inside the main window's content area, establishing the host→tool rendering surface. For this change the tool is hardcoded; no plugin loading is in scope.

#### Scenario: Demo tool is visible
- **WHEN** the main window is shown
- **THEN** the hardcoded demo tool's UI is rendered inside the window's content area using native Reactor controls

### Requirement: Tool interactivity proves the surface

The demo tool SHALL include at least one interactive control whose state updates the rendered UI, proving the host's render/state loop works.

#### Scenario: Interaction updates UI
- **WHEN** the user interacts with the demo tool's control (e.g., a button or input)
- **THEN** the displayed state changes in response, confirming the render/state loop

