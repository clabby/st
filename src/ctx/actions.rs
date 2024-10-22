//! Actions that can be dispatched by the user.

use git2::BranchType;
use nu_ansi_term::Color;
use octocrab::{models::IssueState, pulls::PullRequestHandler};

use crate::{
    errors::{StError, StResult},
    git::RepositoryExt,
};

use super::StContext;

impl<'a> StContext<'a> {
    /// Checks if the current working tree is clean and the stack is restacked.
    pub fn check_cleanliness(&self, branches: &[String]) -> StResult<()> {
        // Return early if the stack is not restacked or the current working tree is dirty.
        if let Some(branch) = branches
            .iter()
            .find(|branch| self.needs_restack(branch).unwrap_or_default())
        {
            return Err(StError::NeedsRestack(branch.to_string()));
        }

        // Check if the working tree is dirty.
        if !self.repository.is_working_tree_clean()? {
            return Err(StError::WorkingTreeDirty);
        }

        Ok(())
    }

    /// Checks if any branches passed have corresponding closed pull requests, and deletes them
    /// if the user confirms.
    pub async fn delete_closed_branches(
        &mut self,
        branches: &[String],
        pulls: &mut PullRequestHandler<'_>,
    ) -> StResult<()> {
        for branch in branches.iter().skip(1) {
            let tracked_branch = self
                .tree
                .get(branch)
                .ok_or_else(|| StError::BranchNotTracked(branch.clone()))?;

            if let Some(remote_meta) = tracked_branch.remote.as_ref() {
                let remote_pr = pulls.get(remote_meta.pr_number).await?;
                let pr_state = remote_pr.state.ok_or(StError::PullRequestNotFound)?;

                if matches!(pr_state, IssueState::Closed) {
                    let confirm = inquire::Confirm::new(
                        format!(
                            "Pull request for branch `{}` is {}. Would you like to delete the local branch?",
                            Color::Green.paint(branch),
                            Color::Red.bold().paint("closed")
                        )
                        .as_str(),
                    )
                    .with_default(false)
                    .prompt()?;

                    if confirm {
                        self.delete_branch(branch, true)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Asks the user for confirmation before deleting a branch.
    pub fn delete_branch(
        &mut self,
        branch_name: &str,
        must_delete_from_tree: bool,
    ) -> StResult<()> {
        // Ensure the user does not:
        // 1. Attempt to delete the trunk branch.
        // 2. Attempt to delete an untracked branch.
        if branch_name == self.tree.trunk_name {
            return Err(StError::CannotDeleteTrunkBranch);
        } else if !self.tree.get(&branch_name).is_some() {
            return Err(StError::BranchNotTracked(branch_name.to_string()));
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
                self.tree.delete(&branch_name)?;
            }
            return Ok(());
        }

        // Check out the trunk branch prior to deletion.
        self.repository
            .checkout_branch(self.tree.trunk_name.as_str())?;

        // Delete the selected branch.
        self.repository
            .find_branch(&branch_name, BranchType::Local)?
            .delete()?;

        // Delete the selected branch from the stack tree.
        self.tree.delete(&branch_name)?;

        Ok(())
    }
}
