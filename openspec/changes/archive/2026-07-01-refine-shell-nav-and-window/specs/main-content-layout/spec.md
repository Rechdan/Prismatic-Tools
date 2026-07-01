## MODIFIED Requirements

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
