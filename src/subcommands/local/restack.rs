//! `restack` subcommand.

use crate::{ctx::StContext, errors::StResult};
use clap::Args;

/// CLI arguments for the `restack` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct RestackCmd;

impl RestackCmd {
    /// Run the `restack` subcommand.
    pub fn run(self, mut ctx: StContext<'_>) -> StResult<()> {
        ctx.restack()
    }
}
