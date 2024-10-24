//! `status` subcommand.

use crate::constants::ST_CFG_FILE_NAME;
use crate::{
    cli::Cli, // Import the Cli struct to access the function
    ctx::StContext,
    errors::{StError, StResult},
};
use clap::Args;
use cli_table::{Cell, Style, Table};
use octocrab::{models::IssueState, Octocrab};
use std::path::PathBuf;

/// CLI arguments for the `config` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct ConfigCmd;

impl ConfigCmd {
    /// Run the `config` subcommand.
    pub fn run(self, _ctx: StContext<'_>) -> StResult<()> {
        let config = Cli::load_cfg_or_initialize()?;
        let config_path = PathBuf::from(env!("HOME")).join(ST_CFG_FILE_NAME);

        if config.github_token.is_empty() {
            return Err(StError::ConfigNotInitialized(
                config_path.display().to_string(),
            ));
        } else {
            println!(
                "Configuration successfully initialized at: {:?}",
                config_path
            );
        }

        Ok(())
    }
}
