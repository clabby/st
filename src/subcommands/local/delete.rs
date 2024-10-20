//! `delete` subcommand.

use crate::{ctx::StContext, git::RepositoryExt};
use anyhow::{anyhow, bail, Result};
use clap::Args;
use git2::BranchType;
use nu_ansi_term::Color;

/// CLI arguments for the `delete` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct DeleteCmd {
    /// Name of the new branch to delete.
    #[clap(index = 1)]
    branch_name: Option<String>,
}

impl DeleteCmd {
    /// Run the `delete` subcommand.
    pub fn run(self, mut ctx: StContext<'_>) -> Result<()> {
        // Gather the display branches.
        let display_branches = ctx.display_branches()?;

        // Prompt the user for the name of the branch to delete, or use the provided name.
        let branch_name = match self.branch_name {
            Some(name) => name,
            None => {
                inquire::Select::new("Select a branch to delete", display_branches)
                    .with_formatter(&|f| f.value.branch_name.clone())
                    .prompt()?
                    .branch_name
            }
        };

        // Ensure the user does not:
        // 1. Attempt to delete the trunk branch.
        // 2. Attempt to delete an untracked branch.
        if branch_name == ctx.tree.trunk_name {
            bail!("Cannot delete the trunk branch.");
        } else if !ctx.tree.get(&branch_name).is_some() {
            bail!(
                "Branch `{}` is not tracked with `st`. Track it first with `st track`.",
                branch_name
            );
        }

        // Ask for confirmation to prevent accidental deletion of local refs.
        let confirm = inquire::Confirm::new(
            format!(
                "Are you sure you want to delete branch `{}`?",
                Color::Blue.paint(&branch_name)
            )
            .as_str(),
        )
        .with_default(false)
        .prompt()?;

        // Exit early if the user doesn't confirm.
        if !confirm {
            return Ok(());
        }

        // Check out the trunk branch prior to deletion.
        ctx.repository
            .checkout_branch(ctx.tree.trunk_name.as_str())?;

        // Delete the selected branch.
        ctx.repository
            .find_branch(&branch_name, BranchType::Local)?
            .delete()?;

        // Delete the selected branch from the stack tree.
        ctx.tree.delete(&branch_name).ok_or(anyhow!(
            "Failed to delete branch `{}` from local `st` tree.",
            branch_name
        ))?;

        println!(
            "Successfully deleted branch `{}`.",
            Color::Blue.paint(&branch_name)
        );
        Ok(())
    }
}
