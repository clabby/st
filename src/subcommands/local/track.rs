//! `track` subcommand.

use crate::{
    ctx::StContext,
    errors::{StError, StResult},
    git::RepositoryExt,
};
use clap::Args;
use nu_ansi_term::Color;

/// CLI arguments for the `track` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct TrackCmd;

impl TrackCmd {
    /// Run the `track` subcommand.
    pub fn run(self, mut ctx: StContext<'_>) -> StResult<()> {
        // Gather metadata about the current branch.
        let current_branch = ctx.repository.current_branch()?;
        let current_branch_name = ctx.repository.current_branch_name()?;

        // Ensure the current branch is not already tracked.
        if ctx.tree.get(&current_branch_name).is_some() {
            return Err(StError::BranchAlreadyTracked(current_branch_name));
        }

        // Prompt the user for the parent branch of the current branch.
        let display_branches = ctx.display_branches()?;
        let prompt = format!(
            "Select the parent of `{}`",
            Color::Blue.paint(&current_branch_name)
        );
        let parent_branch_name = inquire::Select::new(prompt.as_str(), display_branches)
            .with_formatter(&|f| f.value.branch_name.clone())
            .prompt()?;

        // Insert the current branch into the stack tree.
        ctx.tree.insert(
            &parent_branch_name.branch_name,
            &current_branch.get().peel_to_commit()?.id().to_string(),
            &current_branch_name,
        )?;

        // Attempt to restack the current stack with the new addition.
        ctx.restack()?;

        println!(
            "Tracked branch `{}` on top of `{}`",
            Color::Green.paint(&current_branch_name),
            Color::Yellow.paint(&parent_branch_name.branch_name)
        );
        Ok(())
    }
}
