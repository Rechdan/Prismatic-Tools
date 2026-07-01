## ADDED Requirements

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

The right container SHALL display exactly one active view at a time: the active
widget (see the `tool-surface` capability) or the config view. The active view
SHALL be selected explicitly — activating the config view via the header Configs
action (see the `app-header` capability), and activating the widget view by
selecting the widget's entry in the navigation column — rather than by toggling a
single control. The shell SHALL show the widget view by default. Selecting a view
SHALL NOT disturb the left navigation column.

#### Scenario: Widget view is active by default
- **WHEN** the main window is first shown
- **THEN** the right container displays the active widget (or its notice/error),
  not the config view

#### Scenario: Selecting the config view activates it
- **WHEN** the config view is activated from the header
- **THEN** the right container displays the config view and the widget is no
  longer rendered in the right container

#### Scenario: Selecting the widget entry returns to the widget view
- **WHEN** the config view is active and the user selects the widget's entry in
  the navigation column
- **THEN** the right container returns to displaying the active widget

#### Scenario: Only one active view at a time
- **WHEN** the right container displays one view (widget or config)
- **THEN** the other view is not simultaneously rendered in the right container
  (the active view replaces the other)

### Requirement: Config view is a placeholder in the right container

The shell SHALL provide a config view that renders inside the right container as
a visual placeholder. For this change the config view reads and writes no
settings and performs no configuration; it exists to occupy the right container
in place of the active widget.

#### Scenario: Config view renders placeholder content
- **WHEN** the config view is shown
- **THEN** the right container displays placeholder config content in place of the
  active widget, while the left navigation column stays visible

#### Scenario: Config view changes no settings
- **WHEN** the config view is shown
- **THEN** no configuration is read from or written to disk or persisted state
  (placeholder behavior)

### Requirement: Left navigation column offers the active widget as a selectable entry

The left navigation column SHALL display the loaded widget's manifest-declared
name as a selectable entry that, when selected, activates the widget view in the
right container. When no widget is loaded, or the widget failed to load, the
navigation column SHALL show its heading only, with no widget entry, and the right
container SHALL continue to show the existing notice or error.

#### Scenario: Nav shows the loaded widget's name as a selectable entry
- **WHEN** a widget loads successfully and the main window is shown
- **THEN** the navigation column displays that widget's manifest name (for
  example, `Counter`) as a selectable entry

#### Scenario: Selecting the widget entry activates the widget view
- **WHEN** the user selects the widget's entry in the navigation column
- **THEN** the right container displays that widget (activating the widget view)

#### Scenario: Nav shows heading only when no widget is available
- **WHEN** there is no widget to load, or the widget failed to load
- **THEN** the navigation column shows its heading with no widget entry, and the
  right container shows the existing notice or error text
