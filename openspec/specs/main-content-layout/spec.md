# main-content-layout Specification

## Purpose
Defines the window body layout below the app header: a persistent two-pane main region (a
fixed-width left navigation column plus a flexible right container) and the explicit
selection between the active views the right container can show (the home view by default,
the active widget, or a placeholder config view). The left navigation column surfaces a
Home entry and the loaded widget as selectable entries so the user can switch between the
home view and the widget view after opening config.

## Requirements
### Requirement: Window body is a header on top plus a main region

The shell SHALL lay out the window content as a fixed header row on top and a
**main** region directly below it that fills all remaining vertical space. The
header row SHALL size to its own content (it does not grow), and the main region
SHALL absorb the rest of the window height so it grows and shrinks with the
window.

#### Scenario: Main region fills the space below the header
- **WHEN** the main window is shown at any size at or above the minimum
- **THEN** the header row sits at the top sized to its content, and the main
  region occupies all vertical space beneath it up to the window's padded edges

#### Scenario: Main region tracks window resizing
- **WHEN** the window is resized taller or shorter
- **THEN** the header keeps its height and the main region grows or shrinks to
  take up the changed remaining space

### Requirement: Main region is a persistent two-pane layout

The main region SHALL be a two-column layout that is always present: a
fixed-width left navigation column of **200 DIP** and, to its right, a flexible
container that takes the remaining horizontal space. Both panes SHALL remain
visible regardless of what the right container currently shows.

#### Scenario: Left column is a fixed 200 DIP width
- **WHEN** the main window is shown
- **THEN** the left navigation column is rendered at a fixed width of 200 DIP and
  does not grow or shrink when the window is resized horizontally

#### Scenario: Right container fills the remaining width
- **WHEN** the main window is shown and the window is resized wider or narrower
- **THEN** the left column keeps its 200 DIP width and the right container grows
  or shrinks to fill the remaining horizontal space

#### Scenario: Both panes stay visible while the right content changes
- **WHEN** the right container's content changes (active widget vs. config view)
- **THEN** the left navigation column remains visible beside the right container

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

### Requirement: Config view is a placeholder in the right container

The shell SHALL provide a config view that renders inside the right container as a
visual placeholder. The config view reads and writes no settings and performs no
configuration, with one exception: it SHALL provide a reload action that reloads
the widget from disk (see the `tool-surface` capability). The reload action's
label SHALL be pluralized by the count of installed widget folders — `Reload
widget` when exactly one widget folder is installed, and `Reload widgets`
otherwise (including when none is installed). The config view exists to occupy the
right container in place of the active widget and to host that reload action.

#### Scenario: Config view renders placeholder content

- **WHEN** the config view is shown
- **THEN** the right container displays placeholder config content in place of the
  active widget, while the left navigation column stays visible

#### Scenario: Config view provides a reload action

- **WHEN** the config view is shown
- **THEN** it displays a reload control whose label reads `Reload widget` when one
  widget folder is installed and `Reload widgets` otherwise

#### Scenario: Reload action reloads the widget without changing the view

- **WHEN** the user activates the reload control in the config view
- **THEN** the shell reloads the widget from disk (see the `tool-surface`
  capability), the active view stays on the config view, and the navigation column
  refreshes to reflect the reloaded widget

#### Scenario: Config view changes no settings

- **WHEN** the config view is shown
- **THEN** no configuration is read from or written to disk or persisted state
  (its only action is the placeholder reload; it persists nothing)

### Requirement: Left navigation column offers the active widget as a selectable entry

The left navigation column SHALL display the loaded widget as a single full-width
clickable entry rendered as a card that fills the width of the navigation column.
Activating (clicking) the card SHALL activate the widget view in the right
container. By default the card SHALL show the loaded widget's manifest-declared
name. When the loaded widget provides a custom navigation preview (see the
`widget-runtime` capability's `nav` function), the card SHALL instead show that
widget-drawn preview, which shares the widget's state so the preview stays live as
the widget's main view updates the shared state. The preview is display-only; it
carries no interactive controls of its own. The card SHALL show an animated hover
highlight — a fill that fades in when the pointer enters the card and fades out
when it leaves — to signal that the card is clickable. When no widget is loaded,
or the widget failed to load, the navigation column SHALL show its heading only,
with no card, and the right container SHALL continue to show the existing notice
or error.

#### Scenario: Nav shows the widget as a full-width clickable card

- **WHEN** a widget loads successfully and the main window is shown
- **THEN** the navigation column displays a single card that fills the column
  width, showing the widget's manifest name (for example, `Counter`) when the
  widget provides no custom preview

#### Scenario: Clicking the card activates the widget view

- **WHEN** the user clicks the widget card in the navigation column
- **THEN** the right container displays that widget (activating the widget view)

#### Scenario: Card shows an animated hover highlight

- **WHEN** the pointer enters the widget card and later leaves it
- **THEN** a hover fill fades in on enter and fades out on leave (an animated
  transition), signaling the card is clickable

#### Scenario: Custom nav preview renders inside the card with shared state

- **WHEN** the loaded widget provides a custom `nav` preview
- **THEN** the card shows that widget-drawn preview, and a change the widget makes
  to its shared state (from the main view) is reflected in the preview

#### Scenario: Card shows the name when the widget provides no custom preview

- **WHEN** a widget that provides no `nav` preview loads successfully
- **THEN** the full-width card shows the widget's manifest name

#### Scenario: Nav shows heading only when no widget is available

- **WHEN** there is no widget to load, or the widget failed to load
- **THEN** the navigation column shows its heading with no card, and the right
  container shows the existing notice or error text

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
