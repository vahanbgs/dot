use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::anyhow;
use nuke_transpile::Target;

/// The extension marking a file in the tree as a Nuke document.
pub const FILE_EXTENSION: &str = "nuke";

/// Whether a path in the tree names a Nuke document.
pub fn names_a_document(path: &Path) -> bool {
    path.extension() == Some(OsStr::new(FILE_EXTENSION))
}

/// Where a Nuke document deploys to, which is its own path minus the `.nuke`.
pub fn destination<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().with_extension("")
}

/// The target a document renders to, looked up first in the overrides and then
/// in the extension the `.nuke` was covering.
///
/// `None` makes the document a module: something other documents import rather
/// than something deployed. A name that says nothing is the host's to answer,
/// and unlike `nuke render` there is nobody here to ask for a `--format`.
pub fn target(
    destination: &Path,
    overrides: &HashMap<PathBuf, String>,
) -> anyhow::Result<Option<Target>> {
    if let Some(name) = overrides.get(destination) {
        return Target::from_str(name)
            .map(Some)
            .map_err(|unknown| anyhow!("{}: {unknown}", destination.display()));
    }

    Ok(destination
        .extension()
        .and_then(OsStr::to_str)
        .and_then(Target::from_extension))
}

/// Renders a Nuke document to a target's text, answering it beside the files it
/// was built from, the document itself first and each import after it.
pub fn render(path: &Path, target: Target) -> anyhow::Result<(String, Vec<PathBuf>)> {
    let source =
        fs::read_to_string(path).map_err(|error| anyhow!("{}: {error}", path.display()))?;

    let reduction = nuke_eval::eval_at_with_files(&source, path)
        .map_err(|error| anyhow!("{}:{}: {error}", path.display(), error.location(&source)))?;

    let text = target
        .render(&reduction.value)
        .map_err(|refusal| anyhow!("{}: {refusal}", path.display()))?;

    Ok((text, reduction.files))
}
