//! User-facing actions that can be performed in the `st` application.

use crate::{ctx::StContext, git::RepositoryExt};
use git2::BranchType;
use nu_ansi_term::Color;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActionError {
    /// Cannot delete the trunk branch.
    #[error("Cannot delete the trunk branch.")]
    CannotDeleteTrunkBranch,
    /// The branch is not tracked with `st`.
    #[error("Branch `{}` is not tracked with `st`. Track it first with `st track`.", Color::Blue.paint(.0))]
    BranchNotTracked(String),
    /// Failed to delete a branch from the local `st` tree.
    #[error("Failed to delete branch `{}` from local `st` tree.", Color::Blue.paint(.0))]
    FailedToDeleteBranch(String),
    /// A [git2::Error] occurred.
    #[error("libgit2 error: {}", .0)]
    Git2Error(#[from] git2::Error),
    /// An [inquire::Error] occurred.
    #[error("inquire error: {}", .0)]
    InquireError(#[from] inquire::InquireError),
    /// An [anyhow::Error] occurred.
    #[error("anyhow error: {}", .0)]
    AnyhowError(#[from] anyhow::Error),
}

pub type ActionResult<T> = Result<T, ActionError>;

/// An [Action] is a re-usable, dispatchable operation that is performed by the user in the `st` application.
pub enum Action<'a> {
    /// Deletes a tracked branch from the local repository and untracks it with `st`.
    DeleteBranch {
        branch_name: &'a str,
        must_delete_from_tree: bool,
    },
}

impl<'a> Action<'a> {
    /// Dispatches the action.
    pub async fn dispatch(self, ctx: &mut StContext<'_>) -> ActionResult<()> {
        match self {
            Action::DeleteBranch {
                branch_name,
                must_delete_from_tree,
            } => {
                // Ensure the user does not:
                // 1. Attempt to delete the trunk branch.
                // 2. Attempt to delete an untracked branch.
                if branch_name == ctx.tree.trunk_name {
                    return Err(ActionError::CannotDeleteTrunkBranch);
                } else if !ctx.tree.get(&branch_name).is_some() {
                    return Err(ActionError::BranchNotTracked(branch_name.to_string()));
                }

                // Ask for confirmation to prevent accidental deletion of local refs.
                let confirm = inquire::Confirm::new(
                    format!(
                        "Are you sure you want to delete branch `{}`?",
                        Color::Blue.paint(branch_name)
                    )
                    .as_str(),
                )
                .with_default(false)
                .prompt()?;

                // Exit early if the user doesn't confirm.
                if !confirm {
                    if must_delete_from_tree {
                        ctx.tree.delete(&branch_name).ok_or_else(|| {
                            ActionError::FailedToDeleteBranch(branch_name.to_string())
                        })?;
                    }
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
                ctx.tree
                    .delete(&branch_name)
                    .ok_or_else(|| ActionError::FailedToDeleteBranch(branch_name.to_string()))?;

                Ok(())
            }
        }
    }
}
