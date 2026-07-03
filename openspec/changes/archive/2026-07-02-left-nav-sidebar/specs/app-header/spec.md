## REMOVED Requirements

### Requirement: Window content is inset from the window border

**Reason**: The header capability is dissolved. The window-edge inset is no longer a
header concern — it now belongs to the two-pane body layout.

**Migration**: Relocated verbatim to the `main-content-layout` capability, which now
owns the window content and applies the same inset (the margin on the two-pane body).

### Requirement: Header row shows the app name on the left

**Reason**: The top header row is removed. There is no longer a header row to hold
the app name.

**Migration**: The app title (`Prismatic Tools`) relocates to the top of the left
navigation column — see the `main-content-layout` requirement "Navigation column
shows the app title at the top."

### Requirement: Header row provides a Configs action in the same row

**Reason**: The top header row is removed. The Configs action no longer lives in a
header row.

**Migration**: The Configs action relocates to the navigation column's bottom group
with unchanged select-not-toggle behavior — see the `main-content-layout`
requirement "Navigation column provides a Configs entry."

### Requirement: Header row provides a link to the project's GitHub page

**Reason**: The top header row is removed. The GitHub link no longer lives in a
header row.

**Migration**: The GitHub link relocates to the navigation column's bottom group,
opening the same URL — see the `main-content-layout` requirement "Navigation column
provides a link to the project's GitHub page."
