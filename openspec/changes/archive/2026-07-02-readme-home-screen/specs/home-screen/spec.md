## ADDED Requirements

### Requirement: Home view renders the project README

The shell SHALL provide a **home view** that renders the project `README.md` as
its content. The README SHALL be bundled into the application at build time (its
text compiled into the executable), so the home view has no runtime dependency on
a README file beside the exe and always has content to show. The home view is
read-only: it presents the document and performs no configuration and mutates no
state.

#### Scenario: Home view shows the README content

- **WHEN** the home view is active in the right container
- **THEN** the right container displays the project README's text as rendered
  content (for example, the top-level `Prismatic Tools` heading and the prose
  beneath it)

#### Scenario: Home content is always available

- **WHEN** the application runs from a staged run directory that contains no
  separate README file beside the exe
- **THEN** the home view still renders the README, because the README text is
  compiled into the executable rather than read from disk at runtime

### Requirement: README is rendered as formatted markdown

The home view SHALL render the README as **formatted markdown**, not as raw
source text. It SHALL map block-level markdown constructs onto the shell's UI
elements so the document reads as a formatted page:

- **Headings** render with visual weight/size that distinguishes heading levels
  from body text.
- **Paragraphs** render as blocks of body text separated by vertical spacing.
- **Bulleted and numbered lists** render as vertically stacked items, each item
  marked (a bullet or its number) and indented from the surrounding text.
- **Fenced and indented code blocks** render in a monospaced style inside a
  visually distinct container.
- **Block quotes** render as body text visually set off from surrounding
  paragraphs.
- **Horizontal rules** render as a visual separator between sections.

Block-level formatting is what the home view renders. Inline runs within a block
(bold, italic, inline code, links) SHALL be flattened to their readable text — the
rendering layer applies no per-run inline styling — so an inline-code span or an
emphasized word reads as ordinary body text within its paragraph. This is a
deliberate limitation of the rendering layer (see the `home-screen` design), not a
defect. The rendered markup SHALL NOT expose raw markdown syntax characters as the
primary way the document reads (for example, a heading does not read as a literal
leading `#`, a list item does not read as a literal leading `-`, and an emphasized
word does not read as literal `**` / `*` markers surrounding it).

#### Scenario: Headings are visually distinguished from body text

- **WHEN** the home view renders a README heading followed by a paragraph
- **THEN** the heading is rendered with greater visual weight or size than the
  paragraph body text, rather than as identical plain text

#### Scenario: Inline runs are flattened to readable text

- **WHEN** the home view renders a paragraph containing inline-code, bold, or
  italic spans
- **THEN** those spans render as ordinary readable body text within the paragraph
  (the markdown markers are not shown raw, and no run is individually styled)

#### Scenario: Lists render as marked, stacked items

- **WHEN** the home view renders a README bulleted or numbered list
- **THEN** each list item appears on its own line as a marked, indented entry
  stacked vertically

#### Scenario: Code blocks render in a distinct monospaced container

- **WHEN** the home view renders a README fenced code block
- **THEN** the code is shown in a monospaced style within a visually distinct
  container, preserving its lines

#### Scenario: Markdown syntax is not shown raw

- **WHEN** the home view renders a heading or list written in markdown syntax
- **THEN** the rendered output reflects the construct (a heading, a list) rather
  than surfacing the literal markdown markers (`#`, `-`) as the document's text

### Requirement: Links render as plain text

The home view SHALL render a markdown link as its visible label text. Links SHALL
NOT be clickable and SHALL NOT navigate, because the rendering layer has no inline
link navigation. This applies to every link regardless of destination
(repository-relative paths such as `docs/reactor-notes.md` / `LICENSE`, and any
absolute `http(s)` URL alike).

#### Scenario: Link renders as its label text

- **WHEN** the home view renders a markdown link
- **THEN** the link's visible label is rendered as ordinary readable text, and
  clicking it does not navigate

### Requirement: Home content is selectable

The rendered README text SHALL be selectable so the user can select and copy
content from the home view (for example, a command such as `yarn win`). Selection
SHALL NOT make the content editable.

#### Scenario: User selects and copies README text

- **WHEN** the home view is active and the user selects a run of the rendered text
- **THEN** the text can be selected and copied, and the document remains read-only
  (not editable)

### Requirement: Home view is vertically scrollable

The home view SHALL be vertically scrollable within the right container. When the
rendered README is taller than the available space, the user SHALL be able to
scroll it to reach content below the fold, while the app header and the left
navigation column remain fixed in place.

#### Scenario: Long README scrolls within the right container

- **WHEN** the home view is active and the rendered README is taller than the
  right container
- **THEN** the content can be scrolled vertically to reveal the rest of the
  document, and the header and left navigation column stay fixed

#### Scenario: Short README needs no scrolling

- **WHEN** the rendered README fits within the right container
- **THEN** the content is shown without requiring scrolling and without clipping

### Requirement: Home rendering is fault-tolerant

Rendering the README SHALL NOT crash the application. A markdown construct the
mapping does not specifically handle SHALL degrade to readable text rather than
being dropped silently or aborting the process, consistent with the app's
render-errors-as-visible-text posture (a release build aborts on panic, so
rendering must not panic).

#### Scenario: Unhandled construct degrades to readable text

- **WHEN** the README contains a markdown construct the home mapping does not
  specifically style (for example, a table)
- **THEN** its text content is still rendered as readable text in place, and the
  application does not crash
