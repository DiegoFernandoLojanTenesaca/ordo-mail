use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::{CREDENTIALS_FILE, KEYRING_USER, LABEL_LENGTH, LOCALE_LENGTH, SETTINGS_FILE};
use crate::error::{ErrorCode, Result};

#[derive(Clone)]
pub struct Storage {
    dir: PathBuf,
    service: String,
}

impl Storage {
    pub fn new(dir: impl Into<PathBuf>, service: impl Into<String>) -> Self {
        Self { dir: dir.into(), service: service.into() }
    }

    pub fn credentials(&self) -> PathBuf {
        self.dir.join(CREDENTIALS_FILE)
    }

    pub fn settings(&self) -> PathBuf {
        self.dir.join(SETTINGS_FILE)
    }

    pub fn write(&self, file: &Path, content: &str) -> Result<()> {
        fs::create_dir_all(&self.dir)?;
        fs::write(file, content)?;
        Ok(())
    }

    fn token_entry(&self) -> Result<keyring::Entry> {
        Ok(keyring::Entry::new(&self.service, KEYRING_USER)?)
    }

    pub fn load_token(&self) -> Result<Option<String>> {
        match self.token_entry()?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save_token(&self, token: &str) -> Result<()> {
        Ok(self.token_entry()?.set_password(token)?)
    }

    pub fn clear_token(&self) -> Result<()> {
        match self.token_entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ClaudeModel {
    #[default]
    Sonnet,
    Haiku,
    Opus,
}

impl ClaudeModel {
    pub const ALL: [ClaudeModel; 3] = [ClaudeModel::Sonnet, ClaudeModel::Haiku, ClaudeModel::Opus];

    pub fn cli_name(self) -> &'static str {
        match self {
            ClaudeModel::Sonnet => "sonnet",
            ClaudeModel::Haiku => "haiku",
            ClaudeModel::Opus => "opus",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub const ALL: [Theme; 3] = [Theme::System, Theme::Light, Theme::Dark];
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct Catalog {
    pub models: Vec<ClaudeModel>,
    pub themes: Vec<Theme>,
    pub label_max_length: usize,
}

impl Catalog {
    pub fn get() -> Self {
        Self { models: ClaudeModel::ALL.to_vec(), themes: Theme::ALL.to_vec(), label_max_length: LABEL_LENGTH }
    }
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[serde(default)]
#[ts(export)]
pub struct Settings {
    pub model: ClaudeModel,
    pub limit: usize,
    pub cleanup_days: u32,
    pub protected: Vec<String>,
    pub theme: Theme,
    pub locale: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self { model: ClaudeModel::default(), limit: 2000, cleanup_days: 30, protected: Vec::new(), theme: Theme::default(), locale: None }
    }
}

impl Settings {
    pub fn load(storage: &Storage) -> Self {
        fs::read_to_string(storage.settings()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    pub fn save(&self, storage: &Storage) -> Result<()> {
        if self.locale.as_deref().is_some_and(|l| !valid_locale(l)) {
            return Err(ErrorCode::InvalidLocale.into());
        }
        storage.write(&storage.settings(), &serde_json::to_string_pretty(self)?)
    }

    pub fn is_protected(&self, name: &str) -> bool {
        self.protected.iter().any(|p| p.eq_ignore_ascii_case(name))
    }

    pub fn rename(&mut self, old: &str, new: &str) -> bool {
        let was = self.is_protected(old);
        if was {
            self.protect(old, false);
            self.protect(new, true);
        }
        was
    }

    pub fn protect(&mut self, name: &str, protected: bool) {
        self.protected.retain(|p| !p.eq_ignore_ascii_case(name));
        if protected {
            self.protected.push(name.to_string());
        }
    }
}

pub fn valid_locale(locale: &str) -> bool {
    !locale.is_empty() && locale.len() <= LOCALE_LENGTH && locale.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protection_ignores_case() {
        let mut s = Settings::default();
        s.protect("Banco", true);
        assert!(s.is_protected("banco"));
        assert!(s.rename("banco", "Bank") && s.is_protected("Bank") && !s.is_protected("Banco"));
        assert!(!s.rename("Other", "X"));
        s.protect("BANK", false);
        assert!(!s.is_protected("Bank"));
    }

    #[test]
    fn locale_is_a_bcp47_tag() {
        assert!(valid_locale("es") && valid_locale("pt-BR"));
        assert!(!valid_locale("") && !valid_locale("es; drop") && !valid_locale("x".repeat(20).as_str()));
    }
}
