## MODIFIED Requirements

### Requirement: Loaded widget is reloadable from disk on demand

The shell SHALL provide an action (see the `main-content-layout` capability's config-view reload action) that re-discovers and reloads the widget from the `widgets/` folder on disk, replacing the currently loaded widget with a freshly loaded one. The reload SHALL rebuild the widget's runtime from scratch, so its in-memory Lua `state` is recreated from the widget's module definition (in-memory state is discarded) and any on-disk edits to the widget's manifest or Lua files are picked up without restarting the app. The reload SHALL NOT change which view is active (the user stays on whichever view they triggered it from); the reloaded widget SHALL be reflected in the always-visible navigation pane. A reload that fails SHALL be surfaced the same way as an initial load failure: when a widget folder is present but broken, the navigation pane shows a selectable error item whose selection displays the load error in the content area; when no widget folder exists, no widget item is shown — without crashing.

#### Scenario: Reload picks up on-disk changes

- **WHEN** the user triggers the reload action after a widget's files have changed on disk
- **THEN** the shell re-discovers and reloads the widget, and the navigation pane reflects the reloaded widget

#### Scenario: Reload resets in-memory widget state

- **WHEN** the user has interacted with the widget (changing its in-memory `state`) and then triggers the reload action
- **THEN** the widget is rebuilt from disk with its `state` recreated from the module definition (the prior in-memory state is discarded)

#### Scenario: Reload leaves the active view unchanged

- **WHEN** the user triggers the reload action from the config view
- **THEN** the active view stays on the config view, and the navigation pane refreshes to reflect the reloaded widget

#### Scenario: A failing reload is surfaced, not fatal

- **WHEN** the reload action loads a widget folder that is now broken
- **THEN** the navigation pane shows a selectable error item and selecting it displays the load error in the content area, without crashing

#### Scenario: A reload of a now-absent widget shows no item

- **WHEN** the reload action runs and no widget folder exists any longer
- **THEN** the navigation pane shows no widget item and nothing crashes
