use clap::{Parser, Subcommand, command};
use directories_next::BaseDirs;
use std::{fs, io, path::Path};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Deploy,
}

pub fn copy(src: &Path, dst: &Path) -> io::Result<()> {
    for entry in WalkDir::new(src).follow_links(false) {
        let entry = entry?;
        let src_file_path = entry.path();

        let relative_file_path = match src_file_path.strip_prefix(src) {
            Ok(p) if !p.as_os_str().is_empty() => p,
            _ => continue,
        };

        let file_type = entry.file_type();
        let dst_file_path = dst.join(relative_file_path);

        if file_type.is_dir() {
            fs::create_dir_all(dst_file_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = dst_file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(src_file_path, dst_file_path)?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let _ = Cli::parse();

    let base_dirs = BaseDirs::new().expect("Could not retrieve home directory");

    let src = base_dirs.data_dir().join("dot/home/");
    let dst = base_dirs.home_dir();

    copy(&src, dst)?;

    Ok(())
}
