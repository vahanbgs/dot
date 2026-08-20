use std::{fs, path::PathBuf};

use dot::manifest::{self, Manifest};
use nuke_transpile::Target;
use tempfile::TempDir;

/// A manifest written into a temporary tree, read back the way `load` reads it.
struct Tree {
    _root: TempDir,
    path: PathBuf,
}

impl Tree {
    fn holding(source: &str) -> Self {
        let root = TempDir::new().expect("a temporary directory");
        let path = root.path().join(manifest::FILE_NAME);

        fs::write(&path, source).expect("a manifest");

        Self { _root: root, path }
    }

    fn empty() -> Self {
        let root = TempDir::new().expect("a temporary directory");
        let path = root.path().join(manifest::FILE_NAME);

        Self { _root: root, path }
    }

    fn read(&self) -> anyhow::Result<Manifest> {
        manifest::read(&self.path)
    }
}

#[test]
fn a_tree_that_says_nothing_about_itself_has_nothing_to_say() {
    let tree = Tree::empty();

    assert_eq!(
        tree.read().expect("an absent manifest"),
        Manifest::default()
    );
}

#[test]
fn a_table_names_the_target_of_a_file_that_is_named_rather_than_extended() {
    let tree = Tree::holding(
        "{\n\ttargets = {\n\t\t\".config/ghostty/config\" => Ghostty\n\t\t\".config/git/config\" => Gitconfig\n\t}\n}\n",
    );

    let manifest = tree.read().expect("a manifest");

    assert_eq!(
        manifest
            .targets
            .get(&PathBuf::from(".config/ghostty/config")),
        Some(&Target::Ghostty)
    );
    assert_eq!(
        manifest.targets.get(&PathBuf::from(".config/git/config")),
        Some(&Target::Gitconfig)
    );
}

#[test]
fn a_manifest_needs_no_braces_because_the_file_is_the_table() {
    let braced =
        Tree::holding("{\n\ttargets = {\n\t\t\".config/git/config\" => Gitconfig\n\t}\n}\n");
    let braceless = Tree::holding("targets = {\n\t\".config/git/config\" => Gitconfig\n}\n");

    assert_eq!(
        braceless.read().expect("a braceless manifest"),
        braced.read().expect("a braced manifest")
    );
}

#[test]
fn a_manifest_reads_the_module_it_imports() {
    let tree = Tree::holding("named := @import \"./named.nuke\"\n{targets = named}\n");

    fs::write(
        tree.path.with_file_name("named.nuke"),
        "{\".config/ghostty/config\" => Ghostty}\n",
    )
    .expect("a module");

    assert_eq!(
        tree.read().expect("a manifest").targets,
        [(PathBuf::from(".config/ghostty/config"), Target::Ghostty)].into()
    );
}

#[test]
fn a_manifest_that_is_wrong_names_the_line_that_is_wrong() {
    let tree = Tree::holding("{targets = }\n");

    let reported = tree.read().expect_err("a broken manifest").to_string();

    assert!(reported.contains(manifest::FILE_NAME), "{reported}");
    assert!(reported.contains("1:"), "{reported}");
}

#[test]
fn a_manifest_that_is_merely_not_this_one_is_a_different_fault() {
    for source in [
        "{targets = {\".config/ghostty/config\" => Ghosty}}\n",
        "{targets = {\".config/ghostty/config\" => \"ghostty\"}}\n",
        "{target = {}}\n",
    ] {
        let tree = Tree::holding(source);

        let reported = tree.read().expect_err("a manifest of another shape");

        assert!(
            !reported.to_string().contains("1:1"),
            "a binding fault carries no position: {reported}"
        );
    }
}
