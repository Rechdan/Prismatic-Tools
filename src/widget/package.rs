//! `widget-package`: find a widget folder beside the exe, parse its
//! `widget.toml`, and read its Lua entry source. The widget id is the folder
//! name — there is no id field in the manifest.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::WidgetError;

const WIDGETS_DIR: &str = "widgets";
const MANIFEST_FILE: &str = "widget.toml";
const DEFAULT_ENTRY: &str = "main.lua";

/// Parsed `widget.toml`. `name` is required; `version`/`entry` are optional
/// (`entry` defaults to `main.lua`). `name`/`version` are metadata for a future
/// picker/config surface and are unused today.
pub struct Manifest {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub version: Option<String>,
    pub entry: String,
}

/// A widget resolved from disk: id, folder, manifest, and its Lua entry source.
pub struct WidgetSource {
    pub id: String,
    pub dir: PathBuf,
    #[allow(dead_code)]
    pub manifest: Manifest,
    pub source: String,
}

/// The `widgets/` directory beside the running executable.
fn widgets_root() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    Some(exe.parent()?.join(WIDGETS_DIR))
}

/// Discover and read the first widget folder (sorted by name) beside the exe.
pub fn discover_first() -> Result<WidgetSource, WidgetError> {
    let root = widgets_root().ok_or_else(no_widgets_dir)?;
    if !root.is_dir() {
        return Err(no_widgets_dir());
    }
    let mut dirs: Vec<PathBuf> = fs::read_dir(&root)
        .map_err(|e| WidgetError::NoWidgets(format!("Cannot read widgets folder: {e}")))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    let dir = dirs
        .into_iter()
        .next()
        .ok_or_else(|| WidgetError::NoWidgets("No widgets installed.".into()))?;
    load_from_dir(&dir)
}

fn no_widgets_dir() -> WidgetError {
    WidgetError::NoWidgets("No widgets folder found.".into())
}

/// Load one widget folder: id = folder name, parse manifest, read entry source.
fn load_from_dir(dir: &Path) -> Result<WidgetSource, WidgetError> {
    let id = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();

    let manifest_text =
        fs::read_to_string(dir.join(MANIFEST_FILE)).map_err(|e| WidgetError::Manifest {
            id: id.clone(),
            problem: format!("cannot read {MANIFEST_FILE} ({e})"),
        })?;
    let manifest = parse_manifest(&id, &manifest_text)?;

    let source =
        fs::read_to_string(dir.join(&manifest.entry)).map_err(|e| WidgetError::Entry {
            id: id.clone(),
            problem: format!("cannot read entry '{}' ({e})", manifest.entry),
        })?;

    Ok(WidgetSource {
        id,
        dir: dir.to_path_buf(),
        manifest,
        source,
    })
}

/// Parse `widget.toml` via `toml::Table` (no serde derive for three fields).
fn parse_manifest(id: &str, text: &str) -> Result<Manifest, WidgetError> {
    let table: toml::Table = text.parse().map_err(|e| WidgetError::Manifest {
        id: id.to_string(),
        problem: format!("invalid {MANIFEST_FILE} ({e})"),
    })?;
    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WidgetError::Manifest {
            id: id.to_string(),
            problem: format!("{MANIFEST_FILE} missing required 'name'"),
        })?
        .to_string();
    let version = table
        .get("version")
        .and_then(|v| v.as_str())
        .map(String::from);
    let entry = table
        .get("entry")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_ENTRY)
        .to_string();
    Ok(Manifest {
        name,
        version,
        entry,
    })
}
