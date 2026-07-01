use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;

pub fn resolve_path(s: &str) -> Result<std::path::PathBuf> {
    let path = std::path::absolute(expanduser::expanduser(s)?)?;
    Ok(path)
}

pub fn make_executable<P>(path: P) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    let metadata = std::fs::metadata(&path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    std::fs::set_permissions(&path, permissions)
}
