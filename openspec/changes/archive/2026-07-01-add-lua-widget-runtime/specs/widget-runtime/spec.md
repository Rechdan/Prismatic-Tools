## ADDED Requirements

### Requirement: A widget is a module exposing state and render

A widget's Lua entry SHALL return a module table with a `render` function (required) and a `state` table (optional, defaulting to an empty table). `state` SHALL contain only plain data (nil, boolean, number, string, or nested tables of those). The app SHALL execute the chunk once, hold the returned `state` table so it can be inspected/serialized by future features, and call `render(state)` with that same table on each render.

#### Scenario: Module contract is honored

- **WHEN** a widget's entry returns `{ state = { ... }, render = function(state) ... end }`
- **THEN** the app executes the chunk once and thereafter calls `render(state)` passing the held `state` table

#### Scenario: State table is optional

- **WHEN** a widget's module omits `state`
- **THEN** the app SHALL treat its state as an empty table and still call `render`

#### Scenario: Missing render is reported

- **WHEN** a widget's module does not provide a `render` function
- **THEN** the app SHALL surface a visible error rather than crashing

### Requirement: Widget UI is declared in Lua and rendered with native controls

A widget's `render(state)` SHALL return a UI-description table built from the runtime's UI vocabulary, and the app SHALL translate it into native Reactor controls on each render. The runtime SHALL provide a vertical stack, a horizontal stack (each accepting a `spacing` prop), a text node, and a button node. In a node table, hash keys are props and array entries are children.

#### Scenario: Render table becomes native UI

- **WHEN** `render(state)` returns a table of `vstack`/`hstack`/`text`/`button` nodes
- **THEN** the app displays the corresponding native Reactor controls in the tool surface

#### Scenario: Unrecognized node is reported inline, not fatal

- **WHEN** `render(state)` returns a node whose kind the runtime does not recognize
- **THEN** the app SHALL render a visible inline error in that node's place and SHALL render the rest of the tree, without crashing

### Requirement: Widget controls dispatch events to Lua and state changes re-render

A button carrying an on-click handler SHALL invoke the widget's Lua callback when activated, and any resulting change to the widget's `state` SHALL cause the UI to re-render from `render(state)`.

#### Scenario: Button invokes its Lua callback

- **WHEN** the user activates a button whose node carries a Lua on-click function
- **THEN** the runtime calls that Lua function

#### Scenario: State change updates the display

- **WHEN** a Lua callback mutates the widget's `state`
- **THEN** `render(state)` is re-invoked and the displayed UI reflects the new state

### Requirement: Widget Lua runs in a restricted sandbox

The app SHALL run each widget in a Lua VM that opens only safe libraries — `base`, `table`, `string`, `math`, `coroutine`, `utf8`, and a hardened `package` — and SHALL NOT expose `os`, `io`, or `debug`. Filesystem-reading base functions (`dofile`, `loadfile`) SHALL be removed, `require` SHALL resolve only within the widget's own folder, and native/C module loading SHALL be disabled.

#### Scenario: Dangerous libraries are absent

- **WHEN** a widget's Lua references `os`, `io`, or `debug`
- **THEN** those globals are unavailable (nil), so the widget cannot run processes or read/write arbitrary files

#### Scenario: require is scoped to the widget folder

- **WHEN** a widget calls `require` for a sibling module inside its own folder
- **THEN** the module resolves; **AND WHEN** it requires a path outside its folder or a native module, the require fails

### Requirement: Widget execution errors are contained

Errors raised while loading a widget, calling its `render(state)`, or dispatching one of its callbacks SHALL be caught and surfaced as visible messages. They SHALL NOT panic or terminate the app.

#### Scenario: Lua error is surfaced

- **WHEN** a widget's Lua raises an error during load, render, or a callback
- **THEN** the app SHALL display the error text in the tool surface and continue running
