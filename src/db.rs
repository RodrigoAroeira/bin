use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const DATABASE_NAME: &str = "mgr-db.json";

fn get_database_path() -> Result<PathBuf> {
    let dir = dirs::data_local_dir().context("Unable to find data_local_dir")?;
    Ok(dir.join(DATABASE_NAME))
}

#[derive(Serialize, Default, Deserialize)]
pub struct Database {
    bins: HashMap<String, PathBuf>,
    dirty: bool,
}

impl Database {
    pub fn bins(&self) -> &HashMap<String, PathBuf> {
        &self.bins
    }

    pub fn load() -> Result<Self> {
        let path = get_database_path()?;
        if !path.exists() {
            return Ok(Default::default());
        }
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);

        let db = serde_json::from_reader(reader)?;
        Ok(db)
    }

    // NOTE: This overwrites the entry
    pub fn add_entry<P: AsRef<Path>>(&mut self, name: String, path: P) {
        self.bins.insert(name, path.as_ref().to_path_buf());
        self.dirty = true;
    }

    pub fn remove_entry(&mut self, name: &str) -> Result<()> {
        if !self.bins.contains_key(name) {
            anyhow::bail!("`{name}` was never installed as binary")
        }
        self.bins.remove(name);
        self.dirty = true;
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }
        let path = get_database_path()?;
        let tmp_path = path.with_extension("tmp");

        {
            let mut file = std::fs::File::create(&tmp_path)?;
            write!(file, "{}", serde_json::to_string(self)?)?;
        }
        std::fs::rename(tmp_path, path)?;
        self.dirty = false;
        Ok(())
    }
}
