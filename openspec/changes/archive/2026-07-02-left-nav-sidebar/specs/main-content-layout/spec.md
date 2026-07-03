## ADDED Requirements

### Requirement: Window content is inset from the window border

The shell SHALL render the window content (the two-pane navigation-plus-view body)
inside a container that is padded away from the window's outer edges, so no content
sits flush against the window border.

#### Scenario: Content does not touch the window edges
- **WHEN** the main window is shown
- **THEN** there is visible padding between the window border and both the
  navigation column and the right container on all sides

### Requirement: Navigation column pins the app title at the top

The shell SHALL render the app title (`Prismatic Tools`) as the top-most element of
the left navigation column, pinned above the column's scrollable region so it stays
visible regardless of scrolling. The title SHALL be a plain, non-clickable label (it
does not activate any view). It SHALL be present regardless of which view the right
container shows and regardless of whether a widget is loaded, and SHALL live in the
navigation column, not in the hosted view.

#### Scenario: App title is the pinned top element of the navigation column
- **WHEN** the main window is shown
- **THEN** the navigation column displays the text `Prismatic Tools` pinned at its
  top, above the scrollable tool list

#### Scenario: App title is a non-clickable label
- **WHEN** the user clicks the `Prismatic Tools` title
- **THEN** nothing is activated (no view change); the title is a label, not a
  control

#### Scenario: App title lives in the navigation column, not the view
- **WHEN** the right container renders a view (home, widget, or config)
- **THEN** that view's own content does not duplicate the `Prismatic Tools`
  app-title heading

### Requirement: Navigation column scrolls its tool list and pins a bottom action group

The left navigation column SHALL be a three-zone vertical layout: a pinned title at
the top (see the app-title requirement), a **scrollable middle region** that absorbs
the column's free height and holds the Home entry, the `Tools` heading, and the
widget card, and a **pinned bottom action group** holding the **Configs** entry and
the **GitHub** entry. The middle region SHALL scroll vertically when its content
exceeds the available height so that neither the pinned title nor the pinned bottom
group is displaced or clipped. The bottom action group SHALL stay anchored to the
bottom edge of the navigation column and remain fully visible at all times.

#### Scenario: Configs and GitHub are pinned at the bottom of the column
- **WHEN** the main window is shown
- **THEN** the **Configs** and **GitHub** entries are rendered at the bottom edge of
  the navigation column, below the scrollable tool list, and remain visible

#### Scenario: Bottom group stays pinned as the window resizes
- **WHEN** the window is resized taller or shorter
- **THEN** the bottom group (Configs + GitHub) stays anchored to the bottom of the
  navigation column and the pinned title stays anchored to the top; the scrollable
  middle region grows or shrinks to absorb the change

#### Scenario: Overlong tool list scrolls instead of overrunning the pinned zones
- **WHEN** the tool list (Home + `Tools` heading + widget card) is taller than the
  available middle-region height
- **THEN** the middle region shows a vertical scrollbar and scrolls its content,
  while the pinned title and the pinned bottom action group stay in place and are
  not clipped

#### Scenario: No scrollbar when the tool list fits
- **WHEN** the tool list fits within the available middle-region height
- **THEN** the middle region shows no scrollbar and the title, list, and bottom
  group are all visible without scrolling

### Requirement: Navigation column provides a Configs entry

The shell SHALL render a **Configs** entry within the navigation column's pinned
bottom group, rendered full-width. Activating the entry SHALL activate the config
view in the right container. The entry SHALL select the config view rather than
toggle it — the widget view is returned to by selecting the widget's entry in the
navigation column, not by activating Configs again. The navigation column stays
visible either way. The config view itself remains a placeholder that reads and
writes no settings (see the config view requirement).

#### Scenario: Configs entry is present in the navigation column
- **WHEN** the main window is shown
- **THEN** a **Configs** entry is rendered in the navigation column's pinned bottom
  group

#### Scenario: Configs entry activates the config view
- **WHEN** the user activates the **Configs** entry
- **THEN** the right container displays the config view in place of the previous
  view, and the navigation column remains visible

#### Scenario: Activating Configs while config is active keeps config active
- **WHEN** the config view is already active and the user activates the **Configs**
  entry
- **THEN** the config view stays active (the entry does not toggle back to the
  widget)

### Requirement: Navigation column provides a link to the project's GitHub page

The shell SHALL render a **GitHub** entry within the navigation column's pinned
bottom group, rendered full-width and positioned below the Configs entry, that, when
activated, opens the project's GitHub page
(`https://github.com/Rechdan/Prismatic-Tools`) in the user's default browser.

#### Scenario: GitHub entry opens the repository page
- **WHEN** the user activates the **GitHub** entry in the navigation column
- **THEN** the default browser navigates to `https://github.com/Rechdan/Prismatic-Tools`

### Requirement: Navigation column highlights the active view's entry

The navigation column SHALL visually highlight the entry corresponding to the active
view with a soft selected fill, so the sidebar indicates which view is current. The
three view entries participate: the Home entry (highlighted when the home view is
active), the widget card (highlighted when the widget view is active), and the
Configs entry (highlighted when the config view is active). Exactly one entry SHALL
be highlighted at a time, tracking the active view. The GitHub entry SHALL never be
highlighted (it opens an external page and is not a view). The highlight SHALL be a
persistent selected state distinct from the transient hover highlight; on the widget
card the selected fill and the hover fill SHALL be able to coexist (a hovered,
already-selected card shows both). Activating an entry SHALL move the highlight to
that entry as its view becomes active.

#### Scenario: Active view's entry is highlighted
- **WHEN** the home view is active
- **THEN** the Home entry shows the selected highlight and neither the widget card
  nor the Configs entry is highlighted

#### Scenario: Highlight follows the selected view
- **WHEN** the user activates the Configs entry (making the config view active)
- **THEN** the Configs entry becomes highlighted and the Home entry and widget card
  are no longer highlighted

#### Scenario: Re-activating the already-active entry keeps it highlighted
- **WHEN** an entry is highlighted (its view is active) and the user activates that
  same entry again
- **THEN** the entry stays highlighted and its view stays active (the highlight does
  not clear or flicker off)

#### Scenario: Widget card shows selection distinct from hover
- **WHEN** the widget view is active and the pointer is not over the card
- **THEN** the widget card shows the persistent selected fill (without the hover
  fill), and hovering it adds the hover fill on top of the selected fill

#### Scenario: GitHub entry is never highlighted
- **WHEN** any view is active
- **THEN** the GitHub entry shows no selected highlight (only the three view entries
  can be highlighted)

## MODIFIED Requirements

### Requirement: Main region is a persistent two-pane layout

The window body SHALL be a two-column layout that fills the whole padded window with
no header row above it: a fixed-width left navigation column of **200 DIP** and, to
its right, a flexible container that takes the remaining horizontal space. Both panes
SHALL remain visible regardless of what the right container currently shows.

#### Scenario: Two-pane layout fills the whole padded body
- **WHEN** the main window is shown
- **THEN** the navigation column and the right container together fill the whole
  padded window body, with no separate header row above them

#### Scenario: Left column is a fixed 200 DIP width
- **WHEN** the main window is shown
- **THEN** the left navigation column is rendered at a fixed width of 200 DIP and
  does not grow or shrink when the window is resized horizontally

#### Scenario: Right container fills the remaining width
- **WHEN** the main window is shown and the window is resized wider or narrower
- **THEN** the left column keeps its 200 DIP width and the right container grows
  or shrinks to fill the remaining horizontal space

#### Scenario: Both panes stay visible while the right content changes
- **WHEN** the right container's content changes (home vs. active widget vs. config
  view)
- **THEN** the left navigation column remains visible beside the right container

### Requirement: Right container shows the selected active view

The right container SHALL display exactly one active view at a time: the **home
view** (see the `home-screen` capability), the active widget (see the
`tool-surface` capability), or the config view. The active view SHALL be selected
explicitly — activating the home view by selecting the Home entry in the
navigation column, activating the config view by selecting the Configs entry in the
navigation column's bottom group, and activating the widget view by selecting the
widget's entry in the navigation column — rather than by toggling a single
control. The shell SHALL show the **home view by default** when the window is
first shown. Selecting a view SHALL NOT disturb the left navigation column.

#### Scenario: Home view is active by default

- **WHEN** the main window is first shown
- **THEN** the right container displays the home view (the rendered README), not
  the widget view or the config view

#### Scenario: Selecting the config view activates it

- **WHEN** the config view is activated from the navigation column's Configs entry
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

### Requirement: Left navigation column offers a Home entry

The left navigation column SHALL display a **Home** entry inside its scrollable
middle region, below the pinned app title and above the `Tools` heading and the
widget card. The Home entry SHALL be a clickable entry that activates the home view
in the right container (see the `home-screen` capability). The Home entry SHALL
always be present, independent of whether a widget is loaded — unlike the widget
card, which is absent when no widget is available.

#### Scenario: Home entry is shown below the pinned title

- **WHEN** the main window is shown
- **THEN** the navigation column displays a Home entry inside the scrollable region,
  below the pinned app title and above the `Tools` heading and any widget card

#### Scenario: Clicking the Home entry activates the home view

- **WHEN** the user clicks the Home entry in the navigation column
- **THEN** the right container displays the home view

#### Scenario: Home entry is present even with no widget

- **WHEN** there is no widget to load, or the widget failed to load
- **THEN** the navigation column still shows the Home entry (the Home entry does
  not depend on a widget being available)

## REMOVED Requirements

### Requirement: Window body is a header on top plus a main region

**Reason**: The top header row is removed; the two-pane navigation-plus-view region
now fills the whole padded window body directly, with no header above it. Replaced by
the modified "Main region is a persistent two-pane layout" requirement (which now
describes the two-pane layout as the entire window body).

**Migration**: No data or API migration — this is an in-app layout change. The
window content is now a single two-pane grid inset by a margin; the former
header-row / main-row split is gone.
