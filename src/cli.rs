use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use git_url_parse::GitUrl;

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        #[arg(allow_hyphen_values = true)]
        options: Vec<String>,
    },
    Cd,
    Commit {
        #[arg(allow_hyphen_values = true)]
        options: Vec<String>,
    },
    Completions {
        shell: Shell,
    },
    Deploy,
    Edit {
        path: PathBuf,

        #[arg(long, conflicts_with = "no_deploy")]
        deploy: bool,

        #[arg(long, conflicts_with = "deploy")]
        no_deploy: bool,
    },
    Init {
        #[arg(value_parser = GitUrl::parse)]
        repository: GitUrl,

        #[arg(long, conflicts_with = "no_deploy")]
        deploy: bool,

        #[arg(long, conflicts_with = "deploy")]
        no_deploy: bool,
    },
    Pull {
        #[arg(long, conflicts_with = "no_deploy")]
        deploy: bool,

        #[arg(long, conflicts_with = "deploy")]
        no_deploy: bool,
    },
    Push {
        #[arg(allow_hyphen_values = true)]
        options: Vec<String>,
    },
    Status,
    Track {
        path: PathBuf,

        #[arg(long)]
        template: bool,
    },
}
