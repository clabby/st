//! `log` subcommand.

use crate::{ctx::StContext, git::RepositoryExt};
use anyhow::Result;
use clap::Args;

/// CLI arguments for the `log` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct LogCmd;

impl LogCmd {
    /// Run the `log` subcommand.
    pub fn run(self, ctx: StContext<'_>) -> Result<()> {
        let StContext { repository, .. } = ctx;

        let current_branch = repository.current_branch()?;
        let current_branch_name = current_branch
            .name()?
            .ok_or(anyhow::anyhow!("Name of current branch not found"))?;

        let mut buf = String::new();
        ctx.write_tree(&mut buf, Some(current_branch_name))?;
        print!("{}", buf);

        Ok(())
    }
}
