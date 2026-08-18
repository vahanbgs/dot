use std::{collections::HashMap, fs, path::PathBuf};

use dot::deploy;
use tempfile::TempDir;

/// A source tree and the home directory it deploys over.
struct Trees {
    _root: TempDir,
    src: PathBuf,
    dst: PathBuf,
}

impl Trees {
    fn new() -> Self {
        let root = TempDir::new().expect("a temporary directory");
        let src = root.path().join("repository/home");
        let dst = root.path().join("home");

        fs::create_dir_all(&src).expect("a source tree");
        fs::create_dir_all(&dst).expect("a home directory");

        Self {
            _root: root,
            src,
            dst,
        }
    }

    fn track(&self, relative: &str, contents: &str) -> &Self {
        let path = self.src.join(relative);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("a parent directory");
        }

        fs::write(path, contents).expect("a tracked file");

        self
    }

    fn deployed(&self, relative: &str) -> PathBuf {
        self.dst.join(relative)
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.deployed(relative)).expect("a deployed file")
    }

    fn deploy(&self) -> anyhow::Result<()> {
        self.deploy_with(&mut HashMap::new())
    }

    fn deploy_with(&self, local_variable_map: &mut HashMap<String, String>) -> anyhow::Result<()> {
        deploy::tree(&self.src, &self.dst, local_variable_map)
    }
}

#[test]
fn a_file_is_copied_as_it_stands() {
    let trees = Trees::new();

    trees.track(".config/fish/config.fish", "set -x EDITOR hx\n");

    trees.deploy().expect("a deployment");

    assert_eq!(trees.read(".config/fish/config.fish"), "set -x EDITOR hx\n");
}

#[test]
fn a_tree_is_mirrored_rather_than_flattened() {
    let trees = Trees::new();

    trees
        .track(".config/a/one", "1\n")
        .track(".config/b/two", "2\n");

    trees.deploy().expect("a deployment");

    assert_eq!(trees.read(".config/a/one"), "1\n");
    assert_eq!(trees.read(".config/b/two"), "2\n");
}

#[test]
fn a_template_renders_and_loses_its_extension() {
    let trees = Trees::new();

    trees.track(
        ".config/jj/config.toml.tielpmet",
        "[user]\nname = \"(<|[user_name]|>)\"\n",
    );

    let mut local_variable_map = HashMap::from([("user_name".to_owned(), "vahanbgs".to_owned())]);

    trees
        .deploy_with(&mut local_variable_map)
        .expect("a deployment");

    assert_eq!(
        trees.read(".config/jj/config.toml"),
        "[user]\nname = \"vahanbgs\"\n"
    );
    assert!(!trees.deployed(".config/jj/config.toml.tielpmet").exists());
}
