//! `checkout` subcommand.

use crate::{ctx::StContext, git::RepositoryExt};
use anyhow::Result;
use clap::Args;

/// CLI arguments for the `checkout` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct CheckoutCmd;

impl CheckoutCmd {
    /// Run the `checkout` subcommand.
    pub fn run(self, ctx: StContext<'_>) -> Result<()> {
        let current_branch = ctx.repository.current_branch()?;
        let current_branch_name = current_branch.name()?;
        let branches = ctx.display_branches(current_branch_name)?;

        let branch = inquire::Select::new("Select a branch to checkout", branches)
            .with_formatter(&|f| f.value.branch_name.clone())
            .prompt()?;

        ctx.repository
            .checkout_branch(branch.branch_name.as_str(), None)
    }
}
