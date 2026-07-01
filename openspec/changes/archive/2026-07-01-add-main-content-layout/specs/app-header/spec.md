## MODIFIED Requirements

### Requirement: Header row provides a Configs action in the same row

The shell SHALL render a **Configs** button within the same header row as the app
name. Clicking the button SHALL activate the config view in the main region's
right container (see the `main-content-layout` capability). The button SHALL
select the config view rather than toggle it — the widget view is returned to by
selecting the widget's entry in the navigation column, not by clicking Configs
again. The left navigation column stays visible either way. The config view
itself remains a placeholder that reads and writes no settings.

#### Scenario: Configs button is present alongside the app name
- **WHEN** the main window is shown
- **THEN** a **Configs** button is rendered in the header row, in the same row as
  the app name

#### Scenario: Configs button activates the config view
- **WHEN** the user clicks the **Configs** button
- **THEN** the right container displays the config view in place of the widget,
  and the left navigation column remains visible

#### Scenario: Clicking Configs while config is active keeps config active
- **WHEN** the config view is already active and the user clicks the **Configs**
  button
- **THEN** the config view stays active (the button does not toggle back to the
  widget)
