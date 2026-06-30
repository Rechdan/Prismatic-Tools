## ADDED Requirements

### Requirement: Window content is inset from the window border

The shell SHALL render the window content (header row plus hosted tool) inside a
container that is padded away from the window's outer edges, so no content sits flush
against the window border.

#### Scenario: Content does not touch the window edges
- **WHEN** the main window is shown
- **THEN** there is visible padding between the window border and both the header row and
  the hosted tool's content on all sides

### Requirement: Header row shows the app name on the left

The shell SHALL render a persistent header row at the top of the window content, above the
hosted tool, with the app name (`Prismatic Tools`) as the left-most element of that row.
The header SHALL be present regardless of which tool is hosted below it. The header SHALL
lay the name and the action buttons out space-between, so the name sits at the far left edge
of the row and the action-button group sits at the far right edge.

#### Scenario: App name is the leading element of the header
- **WHEN** the main window is shown
- **THEN** the header row displays the text `Prismatic Tools` positioned to the left of the
  header's action buttons

#### Scenario: Buttons are pinned to the far right
- **WHEN** the main window is shown
- **THEN** the action-button group (Configs + GitHub) is aligned to the right edge of the
  header row, separated from the app name by the row's free space

#### Scenario: App name lives in the header, not the tool
- **WHEN** the hosted tool renders below the header
- **THEN** the tool's own content does not duplicate the `Prismatic Tools` app-name heading

### Requirement: Header row provides a Configs action in the same row

The shell SHALL render a **Configs** button within the same header row as the app name. For
this change the button's action is a placeholder that performs no navigation and changes no
persisted state.

#### Scenario: Configs button is present alongside the app name
- **WHEN** the main window is shown
- **THEN** a **Configs** button is rendered in the header row, in the same row as the app
  name

#### Scenario: Configs button is a placeholder
- **WHEN** the user clicks the **Configs** button
- **THEN** no configuration surface opens and no settings are read or written (placeholder
  behavior)

### Requirement: Header row provides a link to the project's GitHub page

The shell SHALL render a **GitHub** button within the same header row that, when clicked,
opens the project's GitHub page (`https://github.com/Rechdan/Prismatic-Tools`) in the
user's default browser.

#### Scenario: GitHub button opens the repository page
- **WHEN** the user clicks the **GitHub** button in the header row
- **THEN** the default browser navigates to `https://github.com/Rechdan/Prismatic-Tools`
