//! `delete` subcommand.

use crate::{git::RepositoryExt, store::StoreWithRepository};
use anyhow::{anyhow, Result};
use clap::Args;
use git2::BranchType;
use nu_ansi_term::Color::Blue;

/// CLI arguments for the `delete` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct DeleteArgs;

impl DeleteArgs {
    /// Run the `delete` subcommand.
    pub fn run(self, mut store: StoreWithRepository<'_>) -> Result<()> {
        // Gather the branches to display to the user.
        let branches = store.display_branches()?;

        let branch = inquire::Select::new("Select a branch to delete", branches)
            .with_formatter(&|f| f.value.branch_name.clone())
            .prompt()?;

        // Ensure the user doesn't attempt to delete the trunk branch.
        if branch.branch_name == store.stacks.branch {
            return Err(anyhow::anyhow!("Cannot delete the trunk branch."));
        }

        // Ask for confirmation.
        let confirm = inquire::Confirm::new(
            format!("Delete branch `{}`?", Blue.paint(&branch.branch_name)).as_str(),
        )
        .with_default(false)
        .prompt()?;

        // Exit early if the user doesn't confirm.
        if !confirm {
            return Ok(());
        }

        // Delete the branch from the store in-memory.
        store
            .stacks
            .delete_stack_node(&branch.branch_name)
            .ok_or(anyhow!("Branch not found in local `st` store."))?;

        // Checkout the trunk branch prior to deletion.
        store
            .repository
            .checkout_branch(store.stacks.branch.as_str(), None)?;

        // Delete the selected branch.
        store
            .repository
            .find_branch(branch.branch_name.as_str(), BranchType::Local)?
            .delete()?;

        // Update the store on disk.
        store.write()?;

        // Inform the user of success.
        println!(
            "Successfully deleted branch `{}`.",
            Blue.paint(branch.branch_name)
        );
        Ok(())
    }
}
