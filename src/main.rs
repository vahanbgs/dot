mod config;

use anyhow::anyhow;
use clap::{Parser, Subcommand, command};
use config::Config;
use directories_next::BaseDirs;
use opensesame::Editor;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Write},
    path::{self, Path, PathBuf},
};
use tielpmet::template::Template;
use toml::{Table, Value};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        path: PathBuf,

        #[arg(long)]
        template: bool,
    },
    Deploy,
    Edit {
        path: PathBuf,

        #[arg(long, conflicts_with = "no_deploy")]
        deploy: bool,

        #[arg(long, conflicts_with = "deploy")]
        no_deploy: bool,
    },
}

/// Creates the parent directory of a given path if there is one.
fn create_parent_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

fn load_config(base_dirs: &BaseDirs) -> anyhow::Result<Config> {
    let path = base_dirs.config_dir().join("dot/config.toml");

    let config_content = match fs::read_to_string(path) {
        Ok(s) => s,
        _ => return Ok(Config::default()),
    };

    Ok(toml::from_str(&config_content)?)
}

const TEMPLATE_FILE_EXTENSION: &'static str = "tielpmet";

fn add(base_dirs: &BaseDirs, file_path: &Path, template: bool) -> anyhow::Result<()> {
    if !file_path.is_file() {
        Err(anyhow!("file does not exist or is not a suitable file"))?
    }

    let src_file_path = path::absolute(file_path)?;

    let relative_file_path = src_file_path
        .strip_prefix(base_dirs.home_dir())
        .map_err(|_| anyhow!("only files in the home directory can be added"))?;

    let dst_file_path = base_dirs.data_dir().join("dot/home").join(if template {
        relative_file_path.with_added_extension(TEMPLATE_FILE_EXTENSION)
    } else {
        relative_file_path.to_path_buf()
    });

    create_parent_directory(&dst_file_path)?;

    fs::copy(src_file_path, dst_file_path)?;

    Ok(())
}

fn deploy_template(
    src_file_path: &Path,
    dst_file_path: &Path,
    local_variable_map: &mut Table,
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

            local_variable_map.insert(variable_name.to_string(), Value::String(input.to_string()));
        }
    }

    let mut output_file = File::create(&dst_file_path)?;

    template.render(&mut output_file, local_variable_map)?;

    Ok(())
}

fn deploy(base_dirs: &BaseDirs, verbose: bool) -> anyhow::Result<()> {
    let src_dir_path = &base_dirs.data_dir().join("dot/home/");
    let dst_dir_path = base_dirs.home_dir();

    let local_variable_map_path = &base_dirs.config_dir().join("dot/local.toml");

    let previous_local_variable_map = fs::read_to_string(local_variable_map_path)
        .map_err(anyhow::Error::from)
        .and_then(|s| s.parse::<Table>().map_err(anyhow::Error::from))
        .unwrap_or_else(|_| Table::new());

    let mut local_variable_map = previous_local_variable_map.clone();

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
            create_parent_directory(&dst_file_path)?;

            if src_file_path.extension() == Some(OsStr::new(TEMPLATE_FILE_EXTENSION)) {
                deploy_template(src_file_path, &dst_file_path, &mut local_variable_map)?;
            } else {
                fs::copy(src_file_path, dst_file_path)?;
            }
        }
    }

    if local_variable_map != previous_local_variable_map {
        create_parent_directory(&local_variable_map_path)?;

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

fn edit(
    base_dirs: &BaseDirs,
    path: &Path,
    should_deploy: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    let relative_path = path::absolute(path)?
        .strip_prefix(base_dirs.home_dir())
        .map_err(|_| anyhow!("only files in the home directory can be edited"))?
        .to_path_buf();

    Editor::open(base_dirs.data_dir().join("dot/home").join(relative_path))?;

    if should_deploy {
        deploy(base_dirs, verbose)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let base_dirs = BaseDirs::new().expect("Could not retrieve home directory");

    let config = load_config(&base_dirs)?;

    match cli.command {
        Commands::Add { path, template } => add(&base_dirs, &path, template)?,
        Commands::Deploy => deploy(&base_dirs, cli.verbose)?,
        Commands::Edit {
            path,
            deploy,
            no_deploy,
        } => edit(
            &base_dirs,
            &path,
            !no_deploy && (deploy || config.auto_deploy),
            cli.verbose,
        )?,
    }

    Ok(())
}
