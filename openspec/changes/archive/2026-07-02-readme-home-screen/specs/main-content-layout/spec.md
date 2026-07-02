## MODIFIED Requirements

### Requirement: Right container shows the selected active view

The right container SHALL display exactly one active view at a time: the **home
view** (see the `home-screen` capability), the active widget (see the
`tool-surface` capability), or the config view. The active view SHALL be selected
explicitly — activating the home view by selecting the Home entry in the
navigation column, activating the config view via the header Configs action (see
the `app-header` capability), and activating the widget view by selecting the
widget's entry in the navigation column — rather than by toggling a single
control. The shell SHALL show the **home view by default** when the window is
first shown. Selecting a view SHALL NOT disturb the left navigation column.

#### Scenario: Home view is active by default

- **WHEN** the main window is first shown
- **THEN** the right container displays the home view (the rendered README), not
  the widget view or the config view

#### Scenario: Selecting the config view activates it

- **WHEN** the config view is activated from the header
- **THEN** the right container displays the config view and neither the home view
  nor the widget is rendered in the right container

#### Scenario: Selecting the widget entry activates the widget view

- **WHEN** the user selects the widget's entry in the navigation column
- **THEN** the right container displays the active widget, replacing whichever
  view was active

#### Scenario: Selecting the Home entry activates the home view

- **WHEN** another view is active and the user selects the Home entry in the
  navigation column
- **THEN** the right container returns to displaying the home view

#### Scenario: Only one active view at a time

- **WHEN** the right container displays one view (home, widget, or config)
- **THEN** the other views are not simultaneously rendered in the right container
  (the active view replaces the others)

## ADDED Requirements

### Requirement: Left navigation column offers a Home entry

The left navigation column SHALL display a **Home** entry at the top of the
column, above the `Tools` heading and the widget card. The Home entry SHALL be a
clickable entry that activates the home view in the right container (see the
`home-screen` capability). The Home entry SHALL always be present, independent of
whether a widget is loaded — unlike the widget card, which is absent when no
widget is available.

#### Scenario: Home entry is shown at the top of the navigation column

- **WHEN** the main window is shown
- **THEN** the navigation column displays a Home entry at the top, above the
  `Tools` heading and any widget card

#### Scenario: Clicking the Home entry activates the home view

- **WHEN** the user clicks the Home entry in the navigation column
- **THEN** the right container displays the home view

#### Scenario: Home entry is present even with no widget

- **WHEN** there is no widget to load, or the widget failed to load
- **THEN** the navigation column still shows the Home entry (the Home entry does
  not depend on a widget being available)
