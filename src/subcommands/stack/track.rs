//! `track` subcommand.

use crate::{
    git::RepositoryExt,
    store::{StackNode, StoreWithRepository},
};
use anyhow::{anyhow, ensure, Result};
use clap::Args;
use git2::BranchType;
use nu_ansi_term::Color::Blue;

/// CLI arguments for the `track` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct TrackArgs;

impl TrackArgs {
    /// Run the `track` subcommand.
    pub fn run(self, mut store: StoreWithRepository<'_>) -> Result<()> {
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
        let branches = store.display_branches()?;
        let prompt = format!("Select the parent of `{}`", Blue.paint(current_branch_name));
        let parent_branch_name = inquire::Select::new(prompt.as_str(), branches)
            .with_formatter(&|f| f.value.branch_name.clone())
            .prompt()?;

        // Fetch the parent branch.
        let parent_branch = store
            .repository
            .find_branch(parent_branch_name.branch_name.as_str(), BranchType::Local)?;

        // Fetch the annotated commits for the current and parent branch heads.
        let annotated_current = store
            .repository
            .find_annotated_commit(current_branch.get().peel_to_commit()?.id())?;
        let annotated_parent = store
            .repository
            .find_annotated_commit(parent_branch.get().peel_to_commit()?.id())?;

        // Open a rebase operation, rebasing `current` on top of `parent`.
        let mut rebase = store.repository.rebase(
            Some(&annotated_current),
            Some(&annotated_parent),
            None,
            None,
        )?;

        // Apply all rebase operations, halting if there is a conflict.
        while let Some(op) = rebase.next() {
            let _op = op?;
            let index = store.repository.index()?;

            ensure!(
                !index.has_conflicts(),
                "Conflicts detected. Resolve them first."
            );
        }

        // Finish the rebase operation.
        rebase.finish(None)?;

        // Check out the rebased branch.
        store
            .repository
            .checkout_branch(current_branch_name, None)?;

        // Modify the store in-memory to reflect the new stack.
        store
            .stacks
            .find_stack_node(parent_branch_name.branch_name.as_str())
            .ok_or(anyhow!("Could not find stack node for parent branch"))?
            .children
            .push(StackNode::new(current_branch_name.to_string()));
        // Update the store on disk.
        store.write()?;

        Ok(())
    }
}
