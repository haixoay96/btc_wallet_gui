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
    pub preferences_file: PathBuf,
}

impl StoragePaths {
    pub fn resolve() -> Result<Self> {
        let data_dir = data_directory_path()?;
        Ok(Self {
            data_dir: data_dir.clone(),
            encrypted_state_file: data_dir.join(ENCRYPTED_DATA_FILE),
            preferences_file: data_dir.join(PREFERENCES_FILE),
        })
    }
}

pub fn data_directory_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
        .context("Không thể xác định thư mục dữ liệu")?;
    Ok(dirs.data_local_dir().to_path_buf())
}
