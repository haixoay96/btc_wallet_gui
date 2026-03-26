use std::{
    env,
    path::PathBuf,
};

use anyhow::{Context, Result};
use directories::ProjectDirs;

const APP_QUALIFIER: &str = "com";
const APP_ORGANIZATION: &str = "duclinh";
const APP_NAME: &str = "btc_wallet_gui";

const ENCRYPTED_DATA_FILE: &str = "wallet_data.enc";
const LEGACY_DATA_FILE: &str = "wallet_data.json";

#[derive(Debug)]
pub struct StoragePaths {
    pub data_dir: PathBuf,
    pub encrypted_state_file: PathBuf,
    pub legacy_candidates: Vec<PathBuf>,
}

impl StoragePaths {
    pub fn resolve() -> Result<Self> {
        let data_dir =
            if let Some(project_dirs) = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME) {
                project_dirs.data_local_dir().to_path_buf()
            } else {
                env::current_dir()
                    .context("Không lấy được current directory")?
                    .join(APP_NAME)
            };

        let encrypted_state_file = data_dir.join(ENCRYPTED_DATA_FILE);
        let mut legacy_candidates = vec![data_dir.join(LEGACY_DATA_FILE)];

        if let Ok(cwd) = env::current_dir() {
            let cwd_legacy = cwd.join(LEGACY_DATA_FILE);
            if cwd_legacy != legacy_candidates[0] {
                legacy_candidates.push(cwd_legacy);
            }
        }

        Ok(Self {
            data_dir,
            encrypted_state_file,
            legacy_candidates,
        })
    }

    pub fn first_existing_legacy_path(&self) -> Option<&PathBuf> {
        self.legacy_candidates.iter().find(|path| path.exists())
    }
}

#[allow(dead_code)]
pub fn data_directory_path() -> Result<PathBuf> {
    let paths = StoragePaths::resolve()?;
    Ok(paths.data_dir)
}