//! `log` subcommand.

use crate::{ctx::StContext, errors::StResult};
use clap::Args;

/// CLI arguments for the `log` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct LogCmd;

impl LogCmd {
    /// Run the `log` subcommand.
    pub fn run(self, ctx: StContext<'_>) -> StResult<()> {
        ctx.print_tree()?;
        Ok(())
    }
}
