//! `track` subcommand.

use crate::{
    git::RepositoryExt,
    stack::{LocalMetadata, StackedBranch, StackedBranchInner},
    store::StoreWithRepository,
};
use anyhow::{anyhow, Result};
use clap::Args;
use git2::BranchType;
use nu_ansi_term::Color::Blue;

/// CLI arguments for the `track` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct TrackCmd;

impl TrackCmd {
    /// Run the `track` subcommand.
    pub fn run(self, store: StoreWithRepository<'_>) -> Result<()> {
        // Check if the current branch is already being tracked.
        if store.current_stack_node().is_some() {
            return Err(anyhow::anyhow!(
                "Already tracking branch within a stack. Use `st checkout` to switch branches."
            ));
        }

        // Fetch the current branch and its name.
        let current_branch = store.repository.current_branch()?;
        let current_branch_name = current_branch
            .name()?
            .ok_or(anyhow::anyhow!("Name of current branch not found"))?;

        // Prompt the user to select the parent branch.
        let branches = store.display_branches(Some(current_branch_name))?;
        let prompt = format!("Select the parent of `{}`", Blue.paint(current_branch_name));
        let parent_branch_name = inquire::Select::new(prompt.as_str(), branches)
            .with_formatter(&|f| f.value.branch_name.clone())
            .prompt()?;

        // Modify the store in-memory to reflect the new stack.
        let child_local_meta = LocalMetadata {
            branch_name: current_branch_name.to_string(),
            parent_oid_cache: store
                .repository
                .find_branch(&parent_branch_name.branch_name, BranchType::Local)?
                .get()
                .target()
                .ok_or(anyhow!("Could not find target for parent branch ref"))?
                .to_string(),
        };
        let new_child = StackedBranch::new(StackedBranchInner::new(child_local_meta, None));
        store
            .stack
            .find_child(parent_branch_name.branch_name.as_str())
            .ok_or(anyhow!("Could not find stack node for parent branch"))?
            .insert_child(new_child);

        // Rebase the current branch onto the parent branch.
        store
            .repository
            .rebase_branch_onto(current_branch_name, parent_branch_name.branch_name.as_str())?;

        // Checkout the branch to complete the tracking operation.
        store
            .repository
            .checkout_branch(current_branch_name, None)?;

        // Update the store on disk.
        store.write()?;

        Ok(())
    }
}
