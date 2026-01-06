use anyhow::anyhow;
use clap::{Parser, Subcommand, command};
use directories_next::BaseDirs;
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use tielpmet::template::Template;
use toml::{Table, Value};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { path: PathBuf },
    Deploy,
}

pub fn load_local_table(path: &Path) -> anyhow::Result<Table> {
    Ok(fs::read_to_string(path)?.parse::<Table>()?)
}

const TEMPLATE_FILE_EXTENSION: &'static str = ".tielpmet";

pub fn add(base_dirs: &BaseDirs, file_path: &Path) -> anyhow::Result<()> {
    if !file_path.is_file() {
        Err(anyhow!("file does not exist or is not a suitable file"))?
    }

    let src_file_path = if file_path.is_absolute() {
        file_path
    } else {
        &env::current_dir()?.join(file_path)
    };

    let relative_file_path = src_file_path
        .strip_prefix(base_dirs.home_dir())
        .map_err(|_| anyhow!("only files in the home directory can be added"))?;

    let dst_file_path = base_dirs
        .data_dir()
        .join("dot/home")
        .join(relative_file_path);

    if let Some(parent) = dst_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(src_file_path, dst_file_path)?;

    Ok(())
}

pub fn deploy_template(
    src_file_path: &Path,
    dst_file_path: &Path,
    local_variable_map: &mut Table,
) -> anyhow::Result<()> {
    let dst_file_path = PathBuf::from(
        dst_file_path
            .to_str()
            .ok_or(anyhow!("could not convert &Path to &str"))?
            .strip_suffix(TEMPLATE_FILE_EXTENSION)
            .unwrap(),
    );

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

pub fn deploy(base_dirs: &BaseDirs) -> anyhow::Result<()> {
    let src_dir_path = &base_dirs.data_dir().join("dot/home/");
    let dst_dir_path = base_dirs.home_dir();

    let local_variable_map_path = &base_dirs.config_dir().join("dot/local.toml");

    let mut local_variable_map = fs::read_to_string(local_variable_map_path)
        .map_err(anyhow::Error::from)
        .and_then(|s| s.parse::<Table>().map_err(anyhow::Error::from))
        .unwrap_or_else(|_| Table::new());

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
            if let Some(parent) = dst_file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let filename: &str = entry.file_name().try_into()?;

            if filename.ends_with(TEMPLATE_FILE_EXTENSION) {
                deploy_template(src_file_path, &dst_file_path, &mut local_variable_map)?;
            } else {
                fs::copy(src_file_path, dst_file_path)?;
            }
        }
    }

    if !local_variable_map.is_empty() {
        if let Some(parent) = local_variable_map_path.parent() {
            // This creates all missing parent directories
            fs::create_dir_all(parent)?;
        }

        fs::write(
            local_variable_map_path,
            toml::to_string_pretty(&local_variable_map)?,
        )?;

        println!(
            "Wrote local configuration variables to {:?} file.",
            local_variable_map_path
        );
    }

    println!(
        "Done copying configuration files from {:?} to {:?}",
        src_dir_path, dst_dir_path
    );

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let base_dirs = BaseDirs::new().expect("Could not retrieve home directory");

    match cli.command {
        Commands::Add { path } => add(&base_dirs, &path)?,
        Commands::Deploy => deploy(&base_dirs)?,
    }

    Ok(())
}
