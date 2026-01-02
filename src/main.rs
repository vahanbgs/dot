use clap::{Parser, Subcommand, command};
use directories_next::BaseDirs;
use std::{
    collections::HashMap,
    fs::{self, File, read_to_string},
    io::{self, Write},
    path::{Path, PathBuf},
};
use tielpmet::template::Template;
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

pub fn deploy(src: &Path, dst: &Path) -> io::Result<()> {
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

            const TEMPLATE_FILE_EXTENSION: &'static str = ".tielpmet";

            if entry
                .file_name()
                .to_str()
                .unwrap()
                .ends_with(TEMPLATE_FILE_EXTENSION)
            {
                let dst_file_path = PathBuf::from(
                    dst_file_path
                        .to_str()
                        .unwrap()
                        .strip_suffix(TEMPLATE_FILE_EXTENSION)
                        .unwrap(),
                );

                let template_string = read_to_string(src_file_path).unwrap();

                let template = Template::from_str(&template_string, "(<|[", "]|>)")
                    .unwrap_or_else(|e| panic!("{}", e));

                let variable_set = template.get_variable_set();

                let mut variable_map = HashMap::<String, String>::new();

                for &variable_name in variable_set {
                    print!("{variable_name}: ");
                    io::stdout().flush().unwrap();

                    let mut input = String::new();

                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");

                    let input = input.trim();

                    variable_map.insert(variable_name.to_string(), input.to_string());
                }

                let mut output_file = File::create(&dst_file_path).unwrap();

                template.render(&mut output_file, &variable_map).unwrap();
            } else {
                fs::copy(src_file_path, dst_file_path)?;
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let _ = Cli::parse();

    let base_dirs = BaseDirs::new().expect("Could not retrieve home directory");

    let src = base_dirs.data_dir().join("dot/home/");
    let dst = base_dirs.home_dir();

    deploy(&src, dst)?;

    Ok(())
}
