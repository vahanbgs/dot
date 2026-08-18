use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::Path,
};

use anyhow::anyhow;
use directories_next::BaseDirs;
use nuke_transpile::Target;
use tempfile::NamedTempFile;
use tielpmet::template::Template;
use toml::Table;
use walkdir::WalkDir;

use crate::{config::Config, nuke};

/// The extension marking a file in the tree as a template.
pub const TEMPLATE_FILE_EXTENSION: &str = "tielpmet";

/// Creates the parent directory of a given path if there is one.
pub fn create_parent_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

/// Writes a file in one step, through a temporary file in the same directory,
/// so a rendering that fails part way through leaves the previous file
/// standing rather than a truncated one.
pub fn write_atomically(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    let directory = path
        .parent()
        .ok_or_else(|| anyhow!("{} has no parent directory", path.display()))?;

    let mut file = NamedTempFile::new_in(directory)?;

    file.write_all(contents)?;
    file.persist(path)?;

    Ok(())
}

fn deploy_document(
    src_file_path: &Path,
    dst_file_path: &Path,
    target: Target,
    verbose: bool,
) -> anyhow::Result<()> {
    let dst_file_path = nuke::destination(dst_file_path);

    let (mut text, files) = nuke::render(src_file_path, target)?;

    // A backend answers text, and where that text ends is the host's to say:
    // `nuke render` closes it with a `println!` and a file wants the same.
    if !text.ends_with('\n') {
        text.push('\n');
    }

    write_atomically(&dst_file_path, text.as_bytes())?;

    if verbose {
        println!("Rendered {:?} as {}", dst_file_path, target);

        for import in files.iter().skip(1) {
            println!("  from {:?}", import);
        }
    }

    Ok(())
}

fn deploy_template(
    src_file_path: &Path,
    dst_file_path: &Path,
    local_variable_map: &mut HashMap<String, String>,
) -> anyhow::Result<()> {
    let dst_file_path = dst_file_path.with_extension("");

    let template_string = fs::read_to_string(src_file_path)?;

    let template =
        Template::from_str(&template_string, "(<|[", "]|>)").unwrap_or_else(|e| panic!("{}", e));

    let variable_set = template.get_variable_set();

    for &variable_name in variable_set {
        if !local_variable_map.contains_key(variable_name) {
            print!("{variable_name}: ");
            io::stdout().flush()?;

            let mut input = String::new();

            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let input = input.trim();

            local_variable_map.insert(variable_name.to_string(), input.to_string());
        }
    }

    let mut rendered = Vec::new();

    template.render(&mut rendered, local_variable_map)?;

    write_atomically(&dst_file_path, &rendered)
}

/// Deploys one tree over another, rendering what has a grammar through Nuke and
/// what has none through a template, and copying everything else.
pub fn tree(
    src_dir_path: &Path,
    dst_dir_path: &Path,
    config: &Config,
    local_variable_map: &mut HashMap<String, String>,
    verbose: bool,
) -> anyhow::Result<()> {
    for entry in WalkDir::new(src_dir_path).follow_links(false) {
        let entry = entry?;
        let src_file_path = entry.path();

        let relative_file_path = match src_file_path.strip_prefix(src_dir_path) {
            Ok(p) if !p.as_os_str().is_empty() => p,
            _ => continue,
        };

        let file_type = entry.file_type();
        let dst_file_path = dst_dir_path.join(relative_file_path);

        if file_type.is_dir() {
            fs::create_dir_all(dst_file_path)?;
        } else if file_type.is_file() {
            let document = if nuke::names_a_document(src_file_path) {
                match nuke::target(&nuke::destination(relative_file_path), &config.targets)? {
                    Some(target) => Some(target),
                    None => continue,
                }
            } else {
                None
            };

            create_parent_directory(&dst_file_path)?;

            if let Some(target) = document {
                deploy_document(src_file_path, &dst_file_path, target, verbose)?;
            } else if src_file_path.extension() == Some(OsStr::new(TEMPLATE_FILE_EXTENSION)) {
                deploy_template(src_file_path, &dst_file_path, local_variable_map)?;
            } else {
                fs::copy(src_file_path, dst_file_path)?;
            }
        }
    }

    Ok(())
}

/// Deploys the tracked tree over the home directory.
pub fn deploy(base_dirs: &BaseDirs, config: &Config, verbose: bool) -> anyhow::Result<()> {
    let src_dir_path = &base_dirs.data_dir().join("dot/home/");
    let dst_dir_path = base_dirs.home_dir();

    let local_variable_map_path = &base_dirs.config_dir().join("dot/local.toml");

    let previous_local_variable_map = fs::read_to_string(local_variable_map_path)
        .map_err(anyhow::Error::from)
        .and_then(|s| s.parse::<Table>().map_err(anyhow::Error::from))
        .unwrap_or_else(|_| Table::new())
        .into_iter()
        .filter_map(|(key, value)| value.as_str().map(|value| (key, value.to_owned())))
        .collect::<HashMap<_, _>>();

    let mut local_variable_map = previous_local_variable_map.clone();

    tree(
        src_dir_path,
        dst_dir_path,
        config,
        &mut local_variable_map,
        verbose,
    )?;

    if local_variable_map != previous_local_variable_map {
        create_parent_directory(local_variable_map_path)?;

        fs::write(
            local_variable_map_path,
            toml::to_string_pretty(&local_variable_map)?,
        )?;

        println!(
            "Wrote local configuration variables to {:?} file.",
            local_variable_map_path
        );
    }

    if verbose {
        println!(
            "Done copying configuration files from {:?} to {:?}",
            src_dir_path, dst_dir_path
        );
    }

    Ok(())
}
