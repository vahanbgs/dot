use anyhow::anyhow;
use clap::{CommandFactory, Parser};
use clap_complete::{self};
use directories_next::BaseDirs;
use dot::{
    cli::{Cli, Commands},
    config,
    deploy::{self, TEMPLATE_FILE_EXTENSION, create_parent_directory},
    nuke,
};
use git_url_parse::GitUrl;
use opensesame::Editor;
use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{self, Path},
    process::Command,
};

fn add(
    base_dirs: &BaseDirs,
    options: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> anyhow::Result<()> {
    let repository_path = base_dirs.data_dir().join("dot");

    Command::new("git")
        .arg("add")
        .args(options)
        .current_dir(repository_path)
        .status()?;

    Ok(())
}

fn cd(base_dirs: &BaseDirs) -> anyhow::Result<()> {
    let repository_path = base_dirs.data_dir().join("dot");

    Command::new(env::var("SHELL")?)
        .current_dir(repository_path)
        .status()?;

    Ok(())
}

fn commit(
    base_dirs: &BaseDirs,
    options: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> anyhow::Result<()> {
    let repository_path = base_dirs.data_dir().join("dot");

    Command::new("git")
        .arg("commit")
        .args(options)
        .current_dir(repository_path)
        .status()?;

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

    let file_path = base_dirs.data_dir().join("dot/home").join(relative_path);

    let source_file_path = [nuke::FILE_EXTENSION, TEMPLATE_FILE_EXTENSION]
        .into_iter()
        .map(|extension| file_path.with_added_extension(extension))
        .find(|candidate| candidate.exists())
        .unwrap_or(file_path);

    Editor::open(source_file_path)?;

    if should_deploy {
        deploy::deploy(base_dirs, verbose)?;
    }

    Ok(())
}

fn init(
    base_dirs: &BaseDirs,
    repository: &GitUrl,
    should_deploy: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    let src_path = repository;
    let dst_path = base_dirs.data_dir().join("dot");

    create_parent_directory(&dst_path)?;

    let src = src_path.to_string();
    let dst = dst_path.to_str().expect("the unexpected");

    if dst_path.exists() {
        return Err(anyhow!("{} already exists", dst));
    }

    if verbose {
        println!("git clone {} {}", src, dst);
    }

    Command::new("git")
        .arg("clone")
        .arg(src)
        .arg(dst)
        .status()?;

    if should_deploy {
        deploy::deploy(base_dirs, verbose)?;
    }

    Ok(())
}

fn pull(base_dirs: &BaseDirs, should_deploy: bool, verbose: bool) -> anyhow::Result<()> {
    let repository_path = base_dirs.data_dir().join("dot");

    Command::new("git")
        .arg("pull")
        .arg("--rebase")
        .current_dir(repository_path)
        .status()?;

    if should_deploy {
        deploy::deploy(base_dirs, verbose)?;
    }

    Ok(())
}

fn push(
    base_dirs: &BaseDirs,
    options: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> anyhow::Result<()> {
    let repository_path = base_dirs.data_dir().join("dot");

    Command::new("git")
        .arg("push")
        .args(options)
        .current_dir(repository_path)
        .status()?;

    Ok(())
}

fn status(base_dirs: &BaseDirs) -> anyhow::Result<()> {
    let repository_path = base_dirs.data_dir().join("dot");

    Command::new("git")
        .arg("status")
        .current_dir(repository_path)
        .status()?;

    Ok(())
}

fn track(base_dirs: &BaseDirs, file_path: &Path, template: bool) -> anyhow::Result<()> {
    if !file_path.is_file() {
        Err(anyhow!("file does not exist or is not a suitable file"))?
    }

    let src_file_path = path::absolute(file_path)?;

    let relative_file_path = src_file_path
        .strip_prefix(base_dirs.home_dir())
        .map_err(|_| anyhow!("only files in the home directory can be tracked"))?;

    let dst_file_path = base_dirs.data_dir().join("dot/home").join(if template {
        relative_file_path.with_added_extension(TEMPLATE_FILE_EXTENSION)
    } else {
        relative_file_path.to_path_buf()
    });

    create_parent_directory(&dst_file_path)?;

    fs::copy(src_file_path, dst_file_path)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let base_dirs = BaseDirs::new().expect("Could not retrieve home directory");

    let config = config::load(&base_dirs)?;

    match cli.command {
        Commands::Add { options } => add(&base_dirs, options)?,
        Commands::Cd => cd(&base_dirs)?,
        Commands::Commit { options } => commit(&base_dirs, options)?,
        Commands::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "dot", &mut io::stdout())
        }
        Commands::Deploy => deploy::deploy(&base_dirs, cli.verbose)?,
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
        Commands::Init {
            repository,
            deploy,
            no_deploy,
        } => init(
            &base_dirs,
            &repository,
            !no_deploy && (deploy || config.auto_deploy),
            cli.verbose,
        )?,
        Commands::Pull { deploy, no_deploy } => pull(
            &base_dirs,
            !no_deploy && (deploy || config.auto_deploy),
            cli.verbose,
        )?,
        Commands::Push { options } => push(&base_dirs, options)?,
        Commands::Status => status(&base_dirs)?,
        Commands::Track { path, template } => track(&base_dirs, &path, template)?,
    }

    Ok(())
}
