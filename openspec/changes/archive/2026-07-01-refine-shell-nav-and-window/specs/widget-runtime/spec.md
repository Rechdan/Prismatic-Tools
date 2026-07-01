## MODIFIED Requirements

### Requirement: A widget is a module exposing state and render

A widget's Lua entry SHALL return a module table with a `render` function (required), a `state` table (optional, defaulting to an empty table), and a `nav` function (optional). `state` SHALL contain only plain data (nil, boolean, number, string, or nested tables of those). The app SHALL execute the chunk once, hold the returned `state` table so it can be inspected/serialized by future features, and call `render(state)` with that same table on each render.

When the module provides a `nav` function, the app SHALL call `nav(state)` with the **same** held `state` table that `render` receives, to produce a **display-only preview** of the widget for the navigation column (see the `main-content-layout` capability). Because both functions read the one shared `state` table, a mutation made from the main view SHALL be reflected in the preview on the next render. `nav` is display-only: its returned tree SHALL be mapped without wiring any interactive callbacks, and a `button` node appearing in a `nav` tree SHALL render as an inline error rather than an interactive control. When the module omits `nav`, the app SHALL fall back to the shell's default navigation entry (the widget name) and SHALL NOT require a `nav` function.

#### Scenario: Module contract is honored

- **WHEN** a widget's entry returns `{ state = { ... }, render = function(state) ... end }`
- **THEN** the app executes the chunk once and thereafter calls `render(state)` passing the held `state` table

#### Scenario: State table is optional

- **WHEN** a widget's module omits `state`
- **THEN** the app SHALL treat its state as an empty table and still call `render`

#### Scenario: Missing render is reported

- **WHEN** a widget's module does not provide a `render` function
- **THEN** the app SHALL surface a visible error rather than crashing

#### Scenario: nav is optional

- **WHEN** a widget's module omits `nav`
- **THEN** the app SHALL still load and render the widget, and the shell SHALL use its default navigation entry (the widget name) for that widget

#### Scenario: nav preview shares state with the main view

- **WHEN** a widget provides both `render` and `nav`, and a callback in the main view mutates `state`
- **THEN** the next render calls both `render(state)` and `nav(state)` with the same held table, so the navigation preview reflects the change

#### Scenario: A button in a nav tree is an inline error

- **WHEN** a widget's `nav(state)` returns a tree containing a `button` node
- **THEN** the app renders an inline error in that node's place (the nav preview is display-only) and does not wire an interactive callback

### Requirement: Widget UI is declared in Lua and rendered with native controls

A widget's `render(state)` SHALL return a UI-description table built from the runtime's UI vocabulary, and the app SHALL translate it into native Reactor controls on each render. The runtime SHALL provide a vertical stack, a horizontal stack (each accepting a `spacing` prop), a text node, a button node, and a **border** node. The `border` node wraps a single child and accepts visual props — `corner_radius`, `border_thickness`, and `padding` (numbers, in DIPs) and `background` and `border_color` (colors expressed as `{r, g, b}` or `{r, g, b, a}` arrays) — and SHALL ignore unrecognized props (forward-compatible). In a node table, hash keys are props and array entries are children. The `border` node is available in both `render` and `nav` trees.

#### Scenario: Render table becomes native UI

- **WHEN** `render(state)` returns a table of `vstack`/`hstack`/`text`/`button`/`border` nodes
- **THEN** the app displays the corresponding native Reactor controls in the tool surface

#### Scenario: Border node applies its visual props

- **WHEN** a `render` or `nav` tree contains a `border` node with `corner_radius`, `padding`, and a `background` color
- **THEN** the app renders a native border with those visual properties wrapping the node's child

#### Scenario: Unknown border props are ignored

- **WHEN** a `border` node carries a prop the runtime does not recognize
- **THEN** the app renders the border from the recognized props and ignores the unknown one, without erroring

#### Scenario: Unrecognized node is reported inline, not fatal

- **WHEN** `render(state)` returns a node whose kind the runtime does not recognize
- **THEN** the app SHALL render a visible inline error in that node's place and SHALL render the rest of the tree, without crashing
