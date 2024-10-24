//! `create` subcommand.

use crate::{ctx::StContext, errors::StResult, git::RepositoryExt};
use clap::Args;
use nu_ansi_term::Color;

/// CLI arguments for the `create` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct CreateCmd {
    /// Name of the new branch to create.
    #[clap(index = 1)]
    branch_name: Option<String>,
}
impl CreateCmd {
    /// Run the `create` subcommand.
    pub fn run(self, mut ctx: StContext<'_>) -> StResult<()> {
        // Gather metadata about the current branch.
        let current_branch = ctx.repository.current_branch()?;
        let current_branch_head = current_branch.get().peel_to_commit()?;
        let current_branch_name = ctx.repository.current_branch_name()?;

        // Prompt the user for the name of their new branch, or use the provided name.
        let new_branch_name = match self.branch_name {
            Some(name) => name,
            None => inquire::Text::new("Name of new branch:").prompt()?,
        };

        // Check if the working tree is clean.
        if !ctx.repository.is_working_tree_clean()? {
            return Err(crate::errors::StError::WorkingTreeDirty);
        }

        // Attempt to create the new branch.
        ctx.repository
            .branch(&new_branch_name, &current_branch_head, false)?;
        ctx.repository.checkout_branch(&new_branch_name)?;

        // Insert the new branch into the stack tree.
        ctx.tree.insert(
            &current_branch_name,
            &current_branch_head.id().to_string(),
            &new_branch_name,
        )?;

        println!(
            "Successfully created and tracked new branch `{}` on top of `{}`",
            Color::Blue.paint(&new_branch_name),
            Color::Blue.paint(&current_branch_name)
        );
        Ok(())
    }
}
