//! `checkout` subcommand.

use crate::{ctx::StContext, git::RepositoryExt};
use anyhow::Result;
use clap::Args;

/// CLI arguments for the `checkout` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct CheckoutCmd {
    /// Name of the tracked branch to check out.
    #[clap(index = 1)]
    branch_name: Option<String>,
}

impl CheckoutCmd {
    /// Run the `checkout` subcommand.
    pub fn run(self, ctx: StContext<'_>) -> Result<()> {
        let branches = ctx.display_branches()?;

        // Prompt the user for the name of the branch to checkout, or use the provided name.
        let branch_name = match self.branch_name {
            Some(branch) => branch,
            None => {
                inquire::Select::new("Select a branch to checkout", branches)
                    .with_formatter(&|f| f.value.branch_name.clone())
                    .prompt()?
                    .branch_name
            }
        };

        // Ensure the provided branch is tracked with `st`.
        if !ctx.tree.get(&branch_name).is_some() {
            anyhow::bail!(
                "Branch `{}` is not tracked with `st`. Track it first with `st track`.",
                branch_name
            );
        }

        ctx.repository
            .checkout_branch(branch_name.as_str())
            .map_err(Into::into)
    }
}
