use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;

const APP_QUALIFIER: &str = "vn";
const APP_ORGANIZATION: &str = "bitboy";
const APP_NAME: &str = "btc_wallet_gui";

const ENCRYPTED_DATA_FILE: &str = "app_data.enc";
const PREFERENCES_FILE: &str = "app_preferences.json";

#[derive(Debug)]
pub struct StoragePaths {
    pub data_dir: PathBuf,
    pub encrypted_state_file: PathBuf,
    pub preferences_file: PathBuf
}

impl StoragePaths {
    pub fn resolve() -> Result<Self> {
        let data_dir = if let Some(project_dirs) =
            ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
        {
            project_dirs.data_local_dir().to_path_buf()
        } else {
            env::current_dir()
                .context("Không lấy được current directory")?
                .join(APP_NAME)
        };

        let encrypted_state_file = data_dir.join(ENCRYPTED_DATA_FILE);
        let preferences_file = data_dir.join(PREFERENCES_FILE);
        Ok(Self {
            data_dir,
            encrypted_state_file,
            preferences_file
        })
    }
}

#[allow(dead_code)]
pub fn data_directory_path() -> Result<PathBuf> {
    let paths = StoragePaths::resolve()?;
    Ok(paths.data_dir)
}
