//! Lua widget subsystem. [`package`] resolves a widget folder beside the exe and
//! reads its manifest + Lua entry source; [`runtime`] runs it in a sandboxed VM
//! and maps its declarative `render(state)` output onto reactor elements.
//!
//! Failure-tolerant by construction: no load or render path panics (release
//! aborts into WinUI), so every problem surfaces as visible text via
//! [`WidgetError`].

mod package;
mod runtime;

use std::fmt;

pub use runtime::LoadedWidget;

/// Discover the first widget folder (sorted) beside the exe and load it into a
/// sandboxed VM. Any failure is a [`WidgetError`] the caller renders as text.
pub fn load_first() -> Result<LoadedWidget, WidgetError> {
    let source = package::discover_first()?;
    runtime::load(&source)
}

/// A widget failure, formatted to one user-visible line. Never panics the app.
pub enum WidgetError {
    /// No `widgets/` dir, or it holds no widget folders — a benign "nothing to
    /// show" state, rendered as a neutral notice rather than an error.
    NoWidgets(String),
    /// `widget.toml` missing, unreadable, unparseable, or missing a field.
    Manifest { id: String, problem: String },
    /// The Lua entry file is missing or unreadable.
    Entry { id: String, problem: String },
    /// The widget's Lua failed to load or errored while running.
    Lua { id: String, problem: String },
}

impl WidgetError {
    /// True for the benign empty case (no widget to show), so the surface can
    /// style it as a plain notice instead of an error.
    pub fn is_empty_notice(&self) -> bool {
        matches!(self, WidgetError::NoWidgets(_))
    }
}

impl fmt::Display for WidgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WidgetError::NoWidgets(msg) => write!(f, "{msg}"),
            WidgetError::Manifest { id, problem }
            | WidgetError::Entry { id, problem }
            | WidgetError::Lua { id, problem } => write!(f, "Widget '{id}': {problem}"),
        }
    }
}
