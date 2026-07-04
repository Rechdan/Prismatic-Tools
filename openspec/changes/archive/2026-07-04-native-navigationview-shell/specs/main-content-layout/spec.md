## MODIFIED Requirements

### Requirement: Window content is inset from the window border

The shell SHALL render the hosted view content (the home view, the active widget, or
the config view) inside the navigation content area with padding from that area's
edges, so no view content sits flush against the window border. The navigation pane
and its toggle follow the platform NavigationView layout.

#### Scenario: View content does not touch the window edges
- **WHEN** the main window is shown
- **THEN** there is visible padding between the content area's edges and the rendered
  view content, so the view content is not flush against the window border

### Requirement: Right container shows the selected active view

The navigation content area SHALL display exactly one active view at a time: the
**home view** (see the `home-screen` capability), the active widget (see the
`tool-surface` capability), or the config view. The active view SHALL be selected by
the navigation selection — selecting the **Home** item activates the home view,
selecting the **widget** item activates the widget view, and selecting the **Configs**
menu item activates the config view. The shell SHALL show the **home view by
default** when the window is first shown. Selecting a view SHALL NOT disturb the
navigation pane.

#### Scenario: Home view is active by default

- **WHEN** the main window is first shown
- **THEN** the content area displays the home view (the rendered README), not the
  widget view or the config view, and the Home navigation item is selected

#### Scenario: Selecting the Configs item activates the config view

- **WHEN** the Configs menu item is selected in the navigation pane
- **THEN** the content area displays the config view and neither the home view nor the
  widget is rendered in the content area

#### Scenario: Selecting the widget item activates the widget view

- **WHEN** the user selects the widget's item in the navigation pane
- **THEN** the content area displays the active widget, replacing whichever view was
  active

#### Scenario: Selecting the Home item activates the home view

- **WHEN** another view is active and the user selects the Home item in the navigation
  pane
- **THEN** the content area returns to displaying the home view

#### Scenario: Only one active view at a time

- **WHEN** the content area displays one view (home, widget, or config)
- **THEN** the other views are not simultaneously rendered (the active view replaces
  the others)

### Requirement: Config view is a placeholder in the right container

The shell SHALL provide a config view that renders inside the navigation content area
as a visual placeholder, activated by the **Configs** navigation menu item. The
config view reads and writes no settings and performs no configuration, with one
exception: it SHALL provide a reload action that reloads the widget from disk (see the
`tool-surface` capability). The reload action's label SHALL be pluralized by the count
of installed widget folders — `Reload widget` when exactly one widget folder is
installed, and `Reload widgets` otherwise (including when none is installed). The
config view exists to occupy the content area in place of the active widget and to
host that reload action.

#### Scenario: Config view renders placeholder content

- **WHEN** the config view is shown
- **THEN** the content area displays placeholder config content in place of the active
  widget, while the navigation pane stays visible

#### Scenario: Config view provides a reload action

- **WHEN** the config view is shown
- **THEN** it displays a reload control whose label reads `Reload widget` when one
  widget folder is installed and `Reload widgets` otherwise

#### Scenario: Reload action reloads the widget without changing the view

- **WHEN** the user activates the reload control in the config view
- **THEN** the shell reloads the widget from disk (see the `tool-surface` capability),
  the active view stays on the config view, and the navigation pane refreshes to
  reflect the reloaded widget

#### Scenario: Config view changes no settings

- **WHEN** the config view is shown
- **THEN** no configuration is read from or written to disk or persisted state (its
  only action is the placeholder reload; it persists nothing)

### Requirement: Left navigation column offers a Home entry

The navigation pane SHALL display a **Home** item as an icon+label
`NavigationView` menu item, positioned above the widget item. The Home item SHALL
activate the home view in the content area (see the `home-screen` capability). The
Home item SHALL always be present, independent of whether a widget is loaded — unlike
the widget item, which is absent when no widget is available.

#### Scenario: Home item is shown in the navigation pane

- **WHEN** the main window is shown
- **THEN** the navigation pane displays a Home item (icon + `Home` label), positioned
  above any widget item

#### Scenario: Selecting the Home item activates the home view

- **WHEN** the user selects the Home item in the navigation pane
- **THEN** the content area displays the home view

#### Scenario: Home item is present even with no widget

- **WHEN** there is no widget to load, or the widget failed to load
- **THEN** the navigation pane still shows the Home item (the Home item does not depend
  on a widget being available)

### Requirement: Left navigation column offers the active widget as a selectable entry

The navigation pane SHALL surface a **loaded** widget as a single selectable
`NavigationView` menu item whose label is the widget's manifest-declared **name** and
whose icon is a shell-assigned symbol. Selecting the item SHALL activate the widget
view in the content area. The item's content SHALL be the widget's name only (no
custom widget-drawn preview). When a widget folder is present but **fails to load**,
the navigation pane SHALL instead show a selectable **error item** labeled with the
widget's folder id and carrying a distinct (attention) icon; selecting it SHALL show
the load error in the content area. When there is **no widget folder at all**, the
navigation pane SHALL show **no widget item** (an absent widget is not an error).

#### Scenario: Nav shows the widget as a named menu item

- **WHEN** a widget loads successfully and the main window is shown
- **THEN** the navigation pane displays a menu item labeled with the widget's manifest
  name (for example, `Counter`) with an icon

#### Scenario: Selecting the widget item activates the widget view

- **WHEN** the user selects the widget item in the navigation pane
- **THEN** the content area displays that widget (activating the widget view)

#### Scenario: A broken widget shows a selectable error item

- **WHEN** a widget folder is present but fails to load (a bad manifest or Lua error)
- **THEN** the navigation pane shows a selectable error item labeled with the widget's
  folder id and an attention icon, and selecting it shows the load error in the
  content area

#### Scenario: No widget item when no widget folder exists

- **WHEN** there is no widget folder to load
- **THEN** the navigation pane shows no widget item (an absent widget is not surfaced
  as an error)

## ADDED Requirements

### Requirement: Navigation uses the native NavigationView with a burger toggle

The shell SHALL render its navigation with reactor's native `NavigationView` rather
than a custom two-pane layout. The NavigationView SHALL show a collapsible left pane
with a visible pane-toggle (burger) button that expands and collapses the pane, and
the pane SHALL display the app title `Prismatic Tools`. The NavigationView SHALL own
both the navigation pane and the content area (the active view renders in the content
area). Navigation items SHALL be rendered as icon+label entries.

#### Scenario: NavigationView pane with a burger toggle is shown
- **WHEN** the main window is shown
- **THEN** a NavigationView is rendered with a left pane, a visible burger toggle
  button, and the pane title `Prismatic Tools`

#### Scenario: Burger toggle collapses and expands the pane
- **WHEN** the user activates the burger toggle button
- **THEN** the navigation pane collapses; activating it again expands the pane

#### Scenario: Navigation items show an icon and a label
- **WHEN** the navigation pane is expanded
- **THEN** each navigation item (Home and the widget) shows an icon alongside its
  text label

### Requirement: Config is reached through a Configs menu item

The shell SHALL surface **Configs** as a normal, tag-routed `NavigationView` menu item
(labeled `Configs`, with an icon), placed last in the menu, and use it as the entry
point to the config view. Selecting the Configs item SHALL activate the config view in
the content area. The NavigationView's built-in **Settings** gear SHALL be **disabled**
(not shown), because its selection does not route through the shell's tag mechanism.

#### Scenario: Configs item is visible in the pane
- **WHEN** the main window is shown
- **THEN** the navigation pane shows a `Configs` menu item, and the built-in Settings
  gear is not shown

#### Scenario: Configs item activates the config view
- **WHEN** the user selects the Configs item
- **THEN** the content area displays the config view (the placeholder plus reload
  action), and the navigation pane remains visible

### Requirement: NavigationView highlights the active view's item

The NavigationView's built-in selection highlight SHALL indicate the active view: the
Home item when the home view is active, the widget item when the widget view is
active, and the Configs item when the config view is active. Exactly one of these
SHALL be selected at a time, tracking the active view. Because Configs is a real menu
item (not the built-in gear), the controlled `selected_tag` SHALL highlight it like any
other item.

#### Scenario: Active view's item is highlighted
- **WHEN** the home view is active
- **THEN** the Home item shows the NavigationView selection highlight, and neither the
  widget item nor the Configs item is selected

#### Scenario: Selection tracks the active view
- **WHEN** the user activates a different view (widget or config)
- **THEN** the NavigationView selection highlight moves to that view's item

## REMOVED Requirements

### Requirement: Main region is a persistent two-pane layout

**Reason**: The custom fixed 200-DIP-column-plus-star-container two-pane grid is
replaced by reactor's native `NavigationView`, which owns the pane and content area
and manages the pane width and collapse itself.
**Migration**: Navigation is now a `NavigationView` (see "Navigation uses the native
NavigationView with a burger toggle"). The pane is collapsible via the burger toggle
rather than a fixed 200-DIP column.

### Requirement: Navigation column pins the app title at the top

**Reason**: The pinned app-title label is replaced by the NavigationView's
`pane_title`, which shows `Prismatic Tools` in the pane.
**Migration**: The app title is set via the NavigationView pane title (see
"Navigation uses the native NavigationView with a burger toggle").

### Requirement: Navigation column scrolls its tool list and pins a bottom action group

**Reason**: The custom three-zone (pinned title / scrollable middle / pinned bottom
group) column is replaced by the NavigationView pane, which scrolls its own item list.
There is no custom pinned Configs/GitHub group.
**Migration**: Items live in the NavigationView menu; Configs is a normal menu item
placed last (see "Config is reached through a Configs menu item"). GitHub is removed
from navigation (see the removed GitHub requirement below).

### Requirement: Navigation column provides a Configs entry

**Reason**: The custom bottom-group Configs entry is replaced by a normal
`NavigationView` menu item (the built-in Settings gear is disabled, as its selection
does not route through the shell's tag mechanism).
**Migration**: Select the `Configs` menu item to open the config view (see "Config is
reached through a Configs menu item").

### Requirement: Navigation column highlights the active view's entry

**Reason**: The bespoke selection-fill / hover-fill overlays (crossfaded opacity
layers on custom entries and the widget card) are replaced by the NavigationView's
built-in selection highlight.
**Migration**: Selection highlighting is provided natively (see "NavigationView
highlights the active view's item"). The custom hover fill on the widget card is
removed along with the card.

### Requirement: Navigation column provides a link to the project's GitHub page

**Reason**: Reactor's reconciler does not re-assert an unchanged `selected_tag`, and
`select_nav_item_by_tag` never walks the built-in Settings item, so a nav item that
opens an external browser cannot cleanly revert its own selection — it would stay
visually highlighted or, when the prior view was config, fail to restore at all. A
browser-opening item would also require a new `windows-sys` `Win32_UI_Shell` feature
(no URL opener exists in the repo). GitHub is therefore removed from navigation.
**Migration**: The project's GitHub page remains linked from the README home view
(the `home-screen` capability); there is no GitHub navigation item.
