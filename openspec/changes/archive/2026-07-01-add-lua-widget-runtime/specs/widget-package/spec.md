## ADDED Requirements

### Requirement: Widgets are packaged as folders beside the exe

The app SHALL load widgets from a `widgets/` directory resolved relative to the running executable's directory, where each immediate subfolder is one widget whose id is its folder name.

#### Scenario: Widgets directory resolved relative to exe

- **WHEN** the app starts
- **THEN** it looks for a `widgets/` directory in the same directory as the running executable

#### Scenario: Each subfolder is one widget identified by its folder name

- **WHEN** `widgets/` contains a subfolder `counter/`
- **THEN** `counter` is treated as a single widget whose id is `counter`

#### Scenario: Missing widgets directory is non-fatal

- **WHEN** no `widgets/` directory exists beside the exe
- **THEN** the app SHALL NOT crash and SHALL render a visible message that no widget was loaded

### Requirement: A widget folder is declared by a manifest

Each widget folder SHALL contain a `widget.toml` manifest whose presence marks the folder as a widget. The manifest declares `name` (required), `version` (optional), and `entry` (optional, defaulting to `main.lua`). The widget id is the folder name; there is no id field in the manifest.

#### Scenario: Valid manifest is loaded

- **WHEN** a widget folder contains a `widget.toml` with a `name` and its entry file is present
- **THEN** the app loads the widget using the declared `entry`, or `main.lua` when `entry` is omitted

#### Scenario: Missing or invalid manifest is reported

- **WHEN** a widget folder is missing `widget.toml` or the manifest cannot be parsed or lacks a required field
- **THEN** the app SHALL NOT crash and SHALL surface a visible error identifying the widget and the problem

### Requirement: The widget's Lua entry is read from disk

The app SHALL resolve and read the widget's Lua entry file from within the widget folder before executing it.

#### Scenario: Entry file resolved and read

- **WHEN** the manifest `entry` (or the default `main.lua`) names a file present in the widget folder
- **THEN** the app reads that file's Lua source for execution

#### Scenario: Missing entry file reported

- **WHEN** the named entry file does not exist in the widget folder
- **THEN** the app SHALL surface a visible error rather than crashing
