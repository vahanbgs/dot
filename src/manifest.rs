use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use directories_next::BaseDirs;
use nuke_eval::bind::{self, ErrorKind};
use nuke_transpile::Target;
use serde::Deserialize;

/// The document at the root of the tracked tree, beside `home/` rather than
/// inside it, so the deployment never walks over it.
pub const FILE_NAME: &str = "dot.nuke";

/// What the tracked tree says about itself, as against `Config`, which says how
/// this machine behaves. This one is versioned with the tree it describes.
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// What a Nuke document renders to where its name does not say, keyed by
    /// the deployed path relative to the home directory. `.gitconfig` and
    /// Ghostty's `config` are named rather than extended, so nothing but this
    /// table can name their target.
    #[serde(default)]
    pub targets: HashMap<PathBuf, Target>,
}

/// Reads the manifest out of the tracked tree.
///
/// A tree that does not say anything about itself is a tree with nothing to
/// say, so a missing file is an empty manifest. Every other fault is reported:
/// a document that cannot be read, one that is wrong, and one that is merely
/// not this manifest are three different things, and answering an empty
/// manifest for any of them would deploy the wrong tree in silence.
pub fn load(base_dirs: &BaseDirs) -> anyhow::Result<Manifest> {
    read(&base_dirs.data_dir().join("dot").join(FILE_NAME))
}

/// Reads the manifest at a path, which is what `load` does once it knows one.
pub fn read(path: &Path) -> anyhow::Result<Manifest> {
    match bind::from_path(path) {
        Ok(manifest) => Ok(manifest),
        Err(error) if is_absent(&error) => Ok(Manifest::default()),
        Err(error) => Err(anyhow!("{error}")),
    }
}

fn is_absent(error: &bind::Error) -> bool {
    matches!(error.kind(), ErrorKind::Unreadable(io::ErrorKind::NotFound))
}
