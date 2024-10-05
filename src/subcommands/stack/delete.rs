//! `delete` subcommand.

use crate::{git::RepositoryExt, store::StoreWithRepository};
use anyhow::{anyhow, bail, Result};
use clap::Args;
use git2::BranchType;
use nu_ansi_term::Color::Blue;

/// CLI arguments for the `delete` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct DeleteCmd {
    /// Name of the tracked branch to delete.
    #[clap(index = 1)]
    branch_name: Option<String>,
}

impl DeleteCmd {
    /// Run the `delete` subcommand.
    pub fn run(self, mut store: StoreWithRepository<'_>) -> Result<()> {
        // Gather the branches to display to the user.
        let branches = store.display_branches()?;

        let branch_name = match self.branch_name {
            Some(name) => name,
            None => {
                inquire::Select::new("Select a branch to delete", branches)
                    .with_formatter(&|f| f.value.branch_name.clone())
                    .prompt()?
                    .branch_name
            }
        };

        // Ensure the user doesn't attempt to delete the trunk branch, and that the branch
        // is tracked by `st`.
        if branch_name == store.stacks.branch {
            bail!("Cannot delete the trunk branch.");
        } else if store.stacks.find_stack_node(&branch_name).is_none() {
            bail!("Branch not found in local `st` store. Is it tracked?");
        }

        // Ask for confirmation.
        let confirm = inquire::Confirm::new(
            format!("Delete branch `{}`?", Blue.paint(&branch_name)).as_str(),
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
            .delete_stack_node(&branch_name)
            .ok_or(anyhow!("Branch not found in local `st` store."))?;

        // Checkout the trunk branch prior to deletion.
        store
            .repository
            .checkout_branch(store.stacks.branch.as_str(), None)?;

        // Delete the selected branch.
        store
            .repository
            .find_branch(branch_name.as_str(), BranchType::Local)?
            .delete()?;

        // Update the store on disk.
        store.write()?;

        // Inform the user of success.
        println!("Successfully deleted branch `{}`.", Blue.paint(branch_name));
        Ok(())
    }
}
