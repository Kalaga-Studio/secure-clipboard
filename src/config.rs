use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub enabled: bool,
    pub clipboard_retry_count: u32,
    pub clipboard_retry_delay_ms: u64,
    pub hotkey: HotkeyConfig,
    pub redaction: RedactionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
    pub copy_before_redact: bool,
    pub copy_settle_delay_ms: u64,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionConfig {
    #[serde(default = "default_true")]
    pub redact_email: bool,
    #[serde(default = "default_true")]
    pub redact_phone: bool,
    #[serde(default = "default_true")]
    pub redact_address: bool,
    #[serde(default = "default_true")]
    pub redact_urls_with_tokens: bool,
    #[serde(default = "default_true")]
    pub redact_account_numbers: bool,
    #[serde(default = "default_true")]
    pub redact_ssn: bool,
    #[serde(default = "default_true")]
    pub redact_labeled_fields: bool,
    #[serde(default = "default_true")]
    pub redact_dates: bool,
    #[serde(default = "default_true")]
    pub redact_names_in_salutations: bool,
    #[serde(default = "default_true")]
    pub redact_ip_addresses: bool,
    #[serde(default = "default_true")]
    pub redact_iban: bool,
    #[serde(default = "default_true")]
    pub redact_passport: bool,
    #[serde(default)]
    pub custom_names: Vec<String>,
    #[serde(default)]
    pub allowlist_tokens: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            clipboard_retry_count: 6,
            clipboard_retry_delay_ms: 40,
            hotkey: HotkeyConfig {
                modifiers: vec!["ctrl".into(), "shift".into()],
                key: "C".into(),
                copy_before_redact: false,
                copy_settle_delay_ms: 90,
            },
            redaction: RedactionConfig {
                redact_email: true,
                redact_phone: true,
                redact_address: true,
                redact_urls_with_tokens: true,
                redact_account_numbers: true,
                redact_ssn: true,
                redact_labeled_fields: true,
                redact_dates: true,
                redact_names_in_salutations: true,
                redact_ip_addresses: true,
                redact_iban: true,
                redact_passport: true,
                custom_names: vec![],
                allowlist_tokens: vec![],
            },
        }
    }
}

impl AppConfig {
    pub fn load_or_create_default() -> Result<Self> {
        let config_path = config_path()?;
        if !config_path.exists() {
            let parent = config_path
                .parent()
                .ok_or_else(|| anyhow!("invalid config path"))?;
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create config dir: {}", parent.display()))?;
            let default = AppConfig::default();
            fs::write(&config_path, toml::to_string_pretty(&default)?).with_context(|| {
                format!("failed to write default config: {}", config_path.display())
            })?;
            return Ok(default);
        }

        let raw = fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read config: {}", config_path.display()))?;
        let cfg = toml::from_str::<AppConfig>(&raw)
            .with_context(|| format!("failed to parse config: {}", config_path.display()))?;
        Ok(cfg)
    }
}

pub fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "secure-clipboard", "secure-clipboard")
        .ok_or_else(|| anyhow!("failed to resolve app data directory"))?;
    Ok(dirs.config_dir().join("config.toml"))
}
