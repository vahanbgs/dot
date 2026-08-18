use std::{collections::HashMap, fs, path::Path};

use dot::{config::Config, deploy};
use tempfile::TempDir;

/// A source tree and the home directory it deploys over.
struct Trees {
    _root: TempDir,
    src: std::path::PathBuf,
    dst: std::path::PathBuf,
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

    fn deployed(&self, relative: &str) -> std::path::PathBuf {
        self.dst.join(relative)
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.deployed(relative)).expect("a deployed file")
    }

    fn deploy(&self, config: &Config) -> anyhow::Result<()> {
        self.deploy_with(config, &mut HashMap::new())
    }

    fn deploy_with(
        &self,
        config: &Config,
        local_variable_map: &mut HashMap<String, String>,
    ) -> anyhow::Result<()> {
        deploy::tree(&self.src, &self.dst, config, local_variable_map, false)
    }
}

fn targets(entries: &[(&str, &str)]) -> Config {
    Config {
        targets: entries
            .iter()
            .map(|(path, target)| (Path::new(path).to_path_buf(), (*target).to_owned()))
            .collect(),
        ..Config::default()
    }
}

#[test]
fn an_extension_under_the_nuke_one_names_the_target() {
    let trees = Trees::new();

    trees.track(
        ".config/alacritty/alacritty.toml.nuke",
        "{\n\tcolors = {cursor = \"#FE8019\"}\n}\n",
    );

    trees.deploy(&Config::default()).expect("a deployment");

    assert_eq!(
        trees.read(".config/alacritty/alacritty.toml"),
        "[colors]\ncursor = \"#FE8019\"\n"
    );
}

#[test]
fn a_document_does_not_deploy_beside_what_it_renders_to() {
    let trees = Trees::new();

    trees.track(".config/app.json.nuke", "{a = 1}\n");

    trees.deploy(&Config::default()).expect("a deployment");

    assert!(trees.deployed(".config/app.json").is_file());
    assert!(!trees.deployed(".config/app.json.nuke").exists());
}

#[test]
fn a_table_names_the_target_of_a_file_that_is_named_rather_than_extended() {
    let trees = Trees::new();

    trees.track(
        ".config/ghostty/config.nuke",
        "{\n\t\"theme\" => \"gruvbox\"\n\t\"font-size\" => 12\n}\n",
    );

    let config = targets(&[(".config/ghostty/config", "ghostty")]);

    trees.deploy(&config).expect("a deployment");

    assert_eq!(
        trees.read(".config/ghostty/config"),
        "theme = gruvbox\nfont-size = 12\n"
    );
}

#[test]
fn a_document_naming_no_target_is_a_module_and_is_not_deployed() {
    let trees = Trees::new();

    trees.track(".config/palette.nuke", "{accent = \"#FE8019\"}\n");

    trees.deploy(&Config::default()).expect("a deployment");

    assert!(!trees.deployed(".config/palette").exists());
    assert!(!trees.deployed(".config/palette.nuke").exists());
}

#[test]
fn a_document_reads_a_module_beside_it() {
    let trees = Trees::new();

    trees
        .track(".config/palette.nuke", "{accent = \"#FE8019\"}\n")
        .track(
            ".config/alacritty/alacritty.toml.nuke",
            "{\n\tcolors = {cursor = @import \"../palette.nuke\".accent}\n}\n",
        );

    trees.deploy(&Config::default()).expect("a deployment");

    assert_eq!(
        trees.read(".config/alacritty/alacritty.toml"),
        "[colors]\ncursor = \"#FE8019\"\n"
    );
    assert!(!trees.deployed(".config/palette").exists());
}

#[test]
fn one_module_feeds_two_targets() {
    let trees = Trees::new();

    trees
        .track(".config/palette.nuke", "{accent = \"#FE8019\"}\n")
        .track(
            ".config/a.json.nuke",
            "{cursor = @import \"./palette.nuke\".accent}\n",
        )
        .track(
            ".config/ghostty/config.nuke",
            "{\n\t\"cursor-color\" => @import \"../palette.nuke\".accent\n}\n",
        );

    let config = targets(&[(".config/ghostty/config", "ghostty")]);

    trees.deploy(&config).expect("a deployment");

    assert!(trees.read(".config/a.json").contains("#FE8019"));
    assert_eq!(
        trees.read(".config/ghostty/config"),
        "cursor-color = #FE8019\n"
    );
}

#[test]
fn a_file_that_is_neither_is_copied_as_it_stands() {
    let trees = Trees::new();

    trees.track(".config/fish/config.fish", "set -x EDITOR hx\n");

    trees.deploy(&Config::default()).expect("a deployment");

    assert_eq!(trees.read(".config/fish/config.fish"), "set -x EDITOR hx\n");
}

#[test]
fn a_template_still_renders_beside_a_document() {
    let trees = Trees::new();

    trees
        .track(
            ".config/jj/config.toml.tielpmet",
            "[user]\nname = \"(<|[user_name]|>)\"\n",
        )
        .track(".config/app.json.nuke", "{a = 1}\n");

    let mut local_variable_map = HashMap::from([("user_name".to_owned(), "vahanbgs".to_owned())]);

    trees
        .deploy_with(&Config::default(), &mut local_variable_map)
        .expect("a deployment");

    assert_eq!(
        trees.read(".config/jj/config.toml"),
        "[user]\nname = \"vahanbgs\"\n"
    );
    assert!(trees.deployed(".config/app.json").is_file());
}

#[test]
fn a_refusal_names_the_document_and_leaves_the_previous_file_standing() {
    let trees = Trees::new();

    fs::create_dir_all(trees.dst.join(".config")).expect("a directory");
    fs::write(trees.deployed(".config/app.ini"), "[kept]\n").expect("a previous deployment");

    trees.track(".config/app.ini.nuke", "{section = {key = [1 2]}}\n");

    let error = trees
        .deploy(&Config::default())
        .expect_err("a refused deployment");

    let reported = error.to_string();

    assert!(reported.contains("app.ini.nuke"), "{reported}");
    assert!(reported.contains("list"), "{reported}");
    assert_eq!(trees.read(".config/app.ini"), "[kept]\n");
}

#[test]
fn a_fault_in_a_document_names_the_file_and_the_position() {
    let trees = Trees::new();

    trees.track(".config/app.json.nuke", "{a = }\n");

    let error = trees
        .deploy(&Config::default())
        .expect_err("a faulting deployment");

    let reported = error.to_string();

    assert!(reported.contains("app.json.nuke:1:"), "{reported}");
}

#[test]
fn a_target_the_table_cannot_name_is_reported() {
    let trees = Trees::new();

    trees.track(".config/ghostty/config.nuke", "{a = 1}\n");

    let config = targets(&[(".config/ghostty/config", "ghosty")]);

    let error = trees.deploy(&config).expect_err("an unknown target");

    assert!(error.to_string().contains("ghosty"), "{error}");
}
