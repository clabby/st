//! `config` subcommand.

use crate::{cli::Cli, config::StConfig, ctx::StContext, errors::StResult};
use inquire::Confirm;

#[derive(Debug, Clone, Eq, PartialEq, clap::Args)]
pub struct ConfigCmd;

impl ConfigCmd {
    /// Run the `config` subcommand to force or allow configuration editing.
    pub fn run(self, _ctx: StContext<'_>) -> StResult<()> {
        let config = Cli::load_cfg_or_initialize()?;
        if config == StConfig::default() || config.github_token.is_empty() {
            println!("Configuration is not initialized. Please configure it now.");
            Cli::prompt_for_configuration("")?;
        } else {
            let parsed_config = toml::to_string_pretty(&config).unwrap();
            println!("Current configuration:\n\n{}", parsed_config);
            if Confirm::new("Do you want to edit the configuration? (default: no)")
                .with_default(false)
                .prompt()?
            {
                Cli::prompt_for_configuration(&parsed_config)?;
                println!("Configuration updated.");
            }
        }
        Ok(())
    }
}
