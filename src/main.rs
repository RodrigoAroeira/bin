use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

mod cli;
mod db;
mod util;

use cli::Command;
use tempdir::TempDir;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    let mut database = db::Database::load().context("Load database")?;
    match args.command {
        Command::Install {
            mut file_path,
            install_path,
            name,
            copy,
            remote,
        } => {
            let _tmp = if remote {
                let (path, dir) = download(&file_path)?;
                file_path = path;
                Some(dir)
            } else {
                None
            };

            let file_path = util::resolve_path(&file_path)?;

            let name = match name {
                Some(name) => name,
                None => Path::new(&file_path)
                    .file_name()
                    .context("Unable to name from given file path")?
                    .display()
                    .to_string(),
            };

            let new_path = install_path.join(&name);

            install(&file_path, &new_path, copy).context("Install")?;
            database.add_entry(name, &new_path);
            if !in_path(&install_path)? {
                eprintln!("{} was not found in path", install_path.display());
            }
            println!(
                "Successfully installed {} to {}",
                file_path.display(),
                new_path.display()
            )
        }
        Command::Uninstall { name } => {
            let path = database.get_entry(&name)?;
            uninstall(path.as_path())?;
            database.remove_entry(&name)?;
            println!("Uninstalled `{name}`");
        }
        Command::List => {
            if database.bins().is_empty() {
                anyhow::bail!("There are no registered binaries in the database");
            }
            for (k, v) in database.bins() {
                println!("{k}:");
                println!("\t-{}", v.display());
            }
        }
        Command::Rename { old_name, new_name } => {
            let old_path = database.get_entry(&old_name)?;
            let new_path = old_path
                .parent()
                .context("Unable to determine parent directory")?
                .join(&new_name);
            std::fs::rename(old_path, &new_path)?;
            database.remove_entry(&old_name)?;
            database.add_entry(new_name, &new_path);
            println!("Renamed `{old_name}` to `{}`", new_path.display());
        }
        Command::Move { name, new_path } => {
            let old_path = database.get_entry(&name)?;
            let new_path = util::resolve_path(&new_path.to_string_lossy())?;
            let new_path = new_path.join(&name);
            install(old_path, &new_path, false).context("Move")?;
            database.remove_entry(&name)?;
            database.add_entry(name.clone(), &new_path);
            println!("Moved `{name}` to {}", new_path.display());
        }
        Command::Adopt { path } => {
            if !path.exists() {
                anyhow::bail!("{} does not exist", path.display())
            }
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .context("Unable to get filename from given path")?
                .to_string();
            database.add_entry(name.clone(), &path);
            println!("Adopted `{name}` from {}", path.display());
        }
        Command::Run { name, args } => {
            let program = database.get_entry(&name)?;
            std::process::Command::new(program)
                .args(args)
                .spawn()
                .and_then(|mut c| c.wait())?;
        }
    }
    database.save()?;
    Ok(())
}

fn install<P>(file_path: P, new_path: P, copy: bool) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    std::fs::copy(&file_path, &new_path)?;
    util::make_executable(&new_path)?;
    if !copy && let Err(e) = std::fs::remove_file(&file_path) {
        eprintln!("Warning: could not remove source file: {e}");
    }
    Ok(())
}

fn uninstall<P>(path: P) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    std::fs::remove_file(path)
}

fn in_path<P>(path: P) -> Result<bool>
where
    P: AsRef<Path>,
{
    let normalized: PathBuf = path.as_ref().components().collect();
    let env_path = match std::env::var("PATH") {
        Ok(env_path) => env_path,
        Err(e) => {
            eprintln!("{e}");
            return Ok(false);
        }
    };
    for p in env_path.split(':').map(expanduser::expanduser) {
        let candidate: PathBuf = p?.components().collect();
        if candidate == normalized {
            return Ok(true);
        }
    }
    Ok(false)
}

fn download(url: &str) -> Result<(String, TempDir)> {
    let tmp = TempDir::new("mgr")?;
    let status = std::process::Command::new("wget")
        .args(["-q", "-P", &tmp.path().display().to_string(), url])
        .spawn()
        .context("Spawn wget")?
        .wait()?;

    if !status.success() {
        anyhow::bail!("Unable to download from {url}")
    }

    let entries: Vec<_> = std::fs::read_dir(tmp.path())?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .collect();

    match &entries[..] {
        [ent] => Ok((ent.path().display().to_string(), tmp)),
        [] => anyhow::bail!("No files were downloaded"),
        _ => anyhow::bail!("Expected 1 file, downloaded {}", entries.len()),
    }
}
