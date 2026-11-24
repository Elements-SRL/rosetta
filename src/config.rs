//! Persistent user settings, stored in `%APPDATA%\rosetta\config.toml`.
//!
//! The only setting is the workspace folder chosen with `rosetta init`.

use serde::{Deserialize, Serialize};
use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub workspace: PathBuf,
}

/// Location of the config file: `%APPDATA%\rosetta\config.toml` (`$HOME/.config/rosetta/...`
/// when `APPDATA` is unset).
pub fn config_path() -> io::Result<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "neither APPDATA nor HOME is set")
        })?;
    Ok(base.join("rosetta").join("config.toml"))
}

/// Workspace proposed by `rosetta init`: `<user profile>\Documents\rosetta`.
pub fn default_workspace() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("Documents")
        .join("rosetta")
}

/// Loads the config, or `None` if it has never been saved.
pub fn load(path: &Path) -> Result<Option<Config>, Box<dyn std::error::Error>> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Some(toml::from_str(&s)?)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn save(path: &Path, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, toml::to_string(config)?)?;
    Ok(())
}

#[cfg(test)]
mod config_tests {
    use super::{Config, load, save};

    #[test]
    fn missing_is_none_and_save_round_trips() {
        let dir = std::env::temp_dir().join("rosetta_config_test");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("sub").join("config.toml");
        assert!(load(&path).unwrap().is_none());

        let cfg = Config {
            workspace: r"C:\Users\me\Documents\rosetta".into(),
        };
        save(&path, &cfg).unwrap();
        assert_eq!(load(&path).unwrap(), Some(cfg));
    }
}
