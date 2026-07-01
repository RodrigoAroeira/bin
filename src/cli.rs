use std::path::{PathBuf, absolute};

use clap::{Parser, Subcommand};

/// Simple package manager that simply installs from a source, local or remote
#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Install a binary to given path
    Install {
        file_path: String,
        #[arg(value_parser = path_parser)]
        install_path: PathBuf,
        #[arg(long, short)]
        name: Option<String>,
        #[arg(short, long)]
        remote: bool,
        /// Copies binary instead of moving to path
        #[arg(short, long)]
        copy: bool,
    },
    /// Remove registered binary from database and delete it
    Uninstall { name: String },
    /// Rename registered binary to a new name in the same directory
    Rename { old_name: String, new_name: String },
    /// Move registered binary to a new path
    Move { name: String, new_path: PathBuf },
    /// Register binary in the database
    Adopt {
        #[arg(value_parser = path_parser)]
        path: PathBuf,
    },
    /// Run binary registered in database
    Run { name: String, args: Vec<String> },
    /// List all binaries in the database
    List,
}

fn path_parser(s: &str) -> anyhow::Result<PathBuf> {
    let path = absolute(expanduser::expanduser(s)?)?;
    Ok(path)
}
