//! `log` subcommand.

use crate::ctx::StContext;
use anyhow::Result;
use clap::Args;

/// CLI arguments for the `log` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct LogCmd;

impl LogCmd {
    /// Run the `log` subcommand.
    pub fn run(self, ctx: StContext<'_>) -> Result<()> {
        ctx.print_tree()?;
        Ok(())
    }
}
