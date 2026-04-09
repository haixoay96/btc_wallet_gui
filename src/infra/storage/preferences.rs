use anyhow::{Context, Result};
use std::fs;

use crate::infra::storage::structure::Storage;
use crate::infra::storage::structure::{AppPreferences, AppTheme};
use crate::ui::i18n::AppLanguage;

// ─── Preference Load/Save ────────────────────────────────────────────────

impl Storage {
    pub fn load_language_preference(&self) -> Result<AppLanguage> {
        if !self.paths.preferences_file.exists() {
            return Ok(AppLanguage::English);
        }
        let prefs = self.load_preferences()?;
        Ok(prefs.language)
    }

    pub fn save_language_preference(&self, language: AppLanguage) -> Result<()> {
        let current = self.load_preferences()?;
        let prefs = AppPreferences {
            language,
            last_selected_wallet: current.last_selected_wallet,
            last_viewed_page: current.last_viewed_page,
            ..Default::default()
        };
        self.save_preferences(&prefs)
    }

    pub fn save_wallet_selection(&self, wallet_index: usize, page: &str) -> Result<()> {
        let prefs = AppPreferences {
            language: self.load_language_preference()?,
            last_selected_wallet: Some(wallet_index),
            last_viewed_page: Some(page.to_string()),
            ..Default::default()
        };
        self.save_preferences(&prefs)
    }

    pub fn load_wallet_selection(&self) -> Result<(Option<usize>, Option<String>)> {
        let prefs = self.load_preferences()?;
        Ok((prefs.last_selected_wallet, prefs.last_viewed_page))
    }

    pub fn load_theme(&self) -> Result<AppTheme> {
        Ok(self.load_preferences()?.theme)
    }

    pub fn save_theme(&self, theme: AppTheme) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.theme = theme;
        self.save_preferences(&prefs)
    }

    pub fn load_font_scale(&self) -> Result<f64> {
        Ok(self.load_preferences()?.font_scale)
    }

    pub fn save_font_scale(&self, scale: f64) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.font_scale = scale;
        self.save_preferences(&prefs)
    }

    pub fn load_high_contrast(&self) -> Result<bool> {
        Ok(self.load_preferences()?.high_contrast)
    }

    pub fn save_high_contrast(&self, enabled: bool) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.high_contrast = enabled;
        self.save_preferences(&prefs)
    }

    pub fn load_onboarding_completed(&self) -> Result<bool> {
        Ok(self.load_preferences()?.onboarding_completed)
    }

    pub fn save_onboarding_completed(&self, completed: bool) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.onboarding_completed = completed;
        self.save_preferences(&prefs)
    }

    pub fn load_esplora_endpoint(&self) -> Result<String> {
        let prefs = self.load_preferences()?;
        if prefs.esplora_endpoint.is_empty() {
            Ok("https://blockstream.info/api".to_string())
        } else {
            Ok(prefs.esplora_endpoint)
        }
    }

    pub fn save_esplora_endpoint(&self, endpoint: String) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.esplora_endpoint = endpoint;
        self.save_preferences(&prefs)
    }

    pub fn load_timeout_secs(&self) -> Result<u64> {
        let prefs = self.load_preferences()?;
        Ok(if prefs.timeout_secs == 0 {
            15
        } else {
            prefs.timeout_secs
        })
    }

    pub fn save_timeout_secs(&self, secs: u64) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.timeout_secs = secs;
        self.save_preferences(&prefs)
    }

    pub fn load_enable_debug(&self) -> Result<bool> {
        Ok(self.load_preferences()?.enable_debug)
    }

    pub fn save_enable_debug(&self, enabled: bool) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.enable_debug = enabled;
        self.save_preferences(&prefs)
    }

    pub fn load_auto_refresh(&self) -> Result<bool> {
        Ok(self.load_preferences()?.auto_refresh)
    }

    pub fn save_auto_refresh(&self, enabled: bool) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.auto_refresh = enabled;
        self.save_preferences(&prefs)
    }

    pub fn load_show_satoshis(&self) -> Result<bool> {
        Ok(self.load_preferences()?.show_satoshis)
    }

    pub fn save_show_satoshis(&self, enabled: bool) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.show_satoshis = enabled;
        self.save_preferences(&prefs)
    }

    pub fn load_compact_mode(&self) -> Result<bool> {
        Ok(self.load_preferences()?.compact_mode)
    }

    pub fn save_compact_mode(&self, enabled: bool) -> Result<()> {
        let mut prefs = self.load_preferences()?;
        prefs.compact_mode = enabled;
        self.save_preferences(&prefs)
    }

    pub fn reset_preferences(&self) -> Result<()> {
        self.save_preferences(&AppPreferences::default())
    }

    // ─── Private helpers ─────────────────────────────────────────────

    fn save_preferences(&self, prefs: &AppPreferences) -> Result<()> {
        let encoded =
            serde_json::to_vec_pretty(prefs).context("Không serialize được app preferences")?;
        let parent: &std::path::Path = self
            .paths
            .preferences_file
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));
        fs::create_dir_all(parent)
            .with_context(|| format!("Không tạo được thư mục dữ liệu: {}", parent.display()))?;
        let tmp_path = self.paths.preferences_file.with_extension("json.tmp");
        fs::write(&tmp_path, encoded)
            .with_context(|| format!("Không ghi được file tạm: {}", tmp_path.display()))?;
        fs::rename(&tmp_path, &self.paths.preferences_file).with_context(|| {
            format!(
                "Không đổi tên file: {}",
                self.paths.preferences_file.display()
            )
        })?;
        Ok(())
    }

    fn load_preferences(&self) -> Result<AppPreferences> {
        if !self.paths.preferences_file.exists() {
            return Ok(AppPreferences::default());
        }
        let encoded = fs::read_to_string(&self.paths.preferences_file).with_context(|| {
            format!(
                "Không đọc được preferences: {}",
                self.paths.preferences_file.display()
            )
        })?;
        let prefs: AppPreferences =
            serde_json::from_str(&encoded).context("Không parse được app preferences")?;
        Ok(prefs)
    }
}
