//! `track` subcommand.

use crate::{
    ctx::StContext,
    git::RepositoryExt,
    stack::{LocalMetadata, STree, STreeInner},
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
    pub fn run(self, ctx: StContext<'_>) -> Result<()> {
        // Check if the current branch is already being tracked.
        if ctx.current_stack_node().is_some() {
            return Err(anyhow::anyhow!(
                "Already tracking branch within a stack. Use `st checkout` to switch branches."
            ));
        }

        // Fetch the current branch and its name.
        let current_branch = ctx.repository.current_branch()?;
        let current_branch_name = current_branch
            .name()?
            .ok_or(anyhow::anyhow!("Name of current branch not found"))?;

        // Prompt the user to select the parent branch.
        let branches = ctx.display_branches(Some(current_branch_name))?;
        let prompt = format!("Select the parent of `{}`", Blue.paint(current_branch_name));
        let parent_branch_name = inquire::Select::new(prompt.as_str(), branches)
            .with_formatter(&|f| f.value.branch_name.clone())
            .prompt()?;

        // Modify the store in-memory to reflect the new stack.
        let child_local_meta = LocalMetadata {
            branch_name: current_branch_name.to_string(),
            parent_oid_cache: ctx
                .repository
                .find_branch(&parent_branch_name.branch_name, BranchType::Local)?
                .get()
                .target()
                .map(|p| p.to_string()),
        };
        let new_child = STree::new(STreeInner::new(child_local_meta, None));
        ctx.tree
            .find_branch(parent_branch_name.branch_name.as_str())
            .ok_or(anyhow!("Could not find stack node for parent branch"))?
            .insert_child(new_child);

        // Rebase the current branch onto the parent branch.
        ctx.repository
            .rebase_branch_onto(current_branch_name, parent_branch_name.branch_name.as_str())?;

        // Checkout the branch to complete the tracking operation.
        ctx.repository.checkout_branch(current_branch_name, None)?;

        Ok(())
    }
}
