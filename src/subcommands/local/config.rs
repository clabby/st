//! `config` subcommand.

use crate::{config::prompt_for_configuration, ctx::StContext, errors::StResult};

#[derive(Debug, Clone, Eq, PartialEq, clap::Args)]
pub struct ConfigCmd;

impl ConfigCmd {
    /// Run the `config` subcommand to force or allow configuration editing.
    pub fn run(self, mut ctx: StContext<'_>) -> StResult<()> {
        let ser = toml::to_string_pretty(&ctx.cfg)?;
        let cfg = prompt_for_configuration(Some(&ser))?;
        ctx.cfg = cfg;

        Ok(())
    }
}
