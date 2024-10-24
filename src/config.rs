//! Contains the global configuration for `st`.

use crate::{constants::ST_CFG_FILE_NAME, errors::StResult};
use nu_ansi_term::Color;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};
use thiserror::Error;

pub(crate) const DEFAULT_CONFIG_PRETTY: &str = r#"# GitHub personal access token. Used for pushing branches to GitHub remotes as well as querying
# information about the active repository.
#
# Must have the following scopes:
# - repo:status
# - repo:public_repo
#
# If you're planning to use st with private repositories, you'll need to add the full `repo` scope.
github_token = """#;

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StConfig {
    /// GitHub personal access token.
    pub github_token: String,
}

impl StConfig {
    /// Loads the configuration from disk.
    pub fn try_load() -> Result<Option<Self>, StConfigError> {
        let config_path = PathBuf::from(env!("HOME")).join(ST_CFG_FILE_NAME);
        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => Ok(Some(config)),
                Err(e) => Err(StConfigError::FailedToLoad(io::Error::new(
                    io::ErrorKind::InvalidData,
                    e,
                ))),
            },
            Err(_) => Ok(None),
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), StConfigError> {
        if self.github_token.is_empty() {
            return Err(StConfigError::MissingField("github_token".to_string()));
        }
        Ok(())
    }
}

impl Drop for StConfig {
    fn drop(&mut self) {
        let config_path = PathBuf::from(env!("HOME")).join(ST_CFG_FILE_NAME);
        fs::write(&config_path, toml::to_string(self).unwrap()).unwrap();
    }
}

/// Error type for global [StConfig] operations.
#[derive(Error, Debug)]
pub enum StConfigError {
    /// Failed to load the configuration file.
    #[error("Failed to load the configuration file: {}", .0)]
    FailedToLoad(io::Error),
    /// Missing a reqired field.
    #[error("Missing required field: {}", .0)]
    MissingField(String),
}

/// Prompts the user to set up the global configuration for `st`.
///
/// ## Returns
/// - `Result<StConfig>` - The newly created global `st` config.
pub fn prompt_for_configuration(existing_config: Option<&str>) -> StResult<StConfig> {
    let setup_text = format!(
        "{} configuration found for `{}`. Set up the environment.",
        existing_config.map(|_| "Existing").unwrap_or("No"),
        Color::Blue.paint("st")
    );

    // Use the provided predefined text or fall back to the default.
    let default_text = existing_config.unwrap_or(DEFAULT_CONFIG_PRETTY);

    // Print the default config.
    let ser_cfg = inquire::Editor::new(&setup_text)
        .with_file_extension(".toml")
        .with_predefined_text(default_text)
        .prompt()?;

    let config: StConfig = toml::from_str(&ser_cfg)?;
    config.validate()?;

    Ok(config)
}

#[cfg(test)]
mod test {
    use super::{StConfig, DEFAULT_CONFIG_PRETTY};

    #[test]
    fn pretty_default_config_is_valid() {
        let de = toml::from_str::<StConfig>(DEFAULT_CONFIG_PRETTY);
        assert!(de.is_ok());
    }
}
