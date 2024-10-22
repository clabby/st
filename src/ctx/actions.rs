//! Actions that can be dispatched by the user.

use super::StContext;
use crate::{
    errors::{StError, StResult},
    git::RepositoryExt,
};
use git2::BranchType;
use nu_ansi_term::Color;
use octocrab::{models::IssueState, pulls::PullRequestHandler};

impl<'a> StContext<'a> {
    /// Restacks the branch onto the parent branch passed.
    ///
    /// Returns `true` if the branch was restacked, `false` otherwise.
    pub fn restack_branch(&mut self, branch_name: &str, parent_name: &str) -> StResult<()> {
        // Skip branches that do not need to be restacked.
        if !self.needs_restack(branch_name)? {
            println!(
                "Branch `{}` does not need to be restacked onto `{}`.",
                Color::Green.paint(branch_name),
                Color::Yellow.paint(parent_name)
            );
            return Ok(());
        }

        // Rebase the branch onto its parent.
        self.repository
            .rebase_branch_onto(branch_name, parent_name)?;

        // Update the parent oid cache.
        let parent_oid = self
            .repository
            .find_branch(parent_name, BranchType::Local)?
            .get()
            .target()
            .ok_or(StError::MissingParentOidCache)?;
        self.tree
            .get_mut(branch_name)
            .ok_or_else(|| StError::BranchNotTracked(branch_name.to_string()))?
            .parent_oid_cache = Some(parent_oid.to_string());

        println!(
            "Restacked branch `{}` onto `{}`.",
            Color::Green.paint(branch_name),
            Color::Yellow.paint(parent_name)
        );
        Ok(())
    }

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
    ) -> StResult<usize> {
        let mut num_closed = 0;
        for branch in branches.iter() {
            let tracked_branch = self
                .tree
                .get(branch)
                .ok_or_else(|| StError::BranchNotTracked(branch.clone()))?;

            if let Some(remote_meta) = tracked_branch.remote.as_ref() {
                let remote_pr = pulls.get(remote_meta.pr_number).await?;
                let pr_state = remote_pr.state.ok_or(StError::PullRequestNotFound)?;

                if matches!(pr_state, IssueState::Closed) || remote_pr.merged_at.is_some() {
                    let confirm = inquire::Confirm::new(
                        format!(
                            "Pull request for branch `{}` is {}. Would you like to delete the local branch?",
                            Color::Green.paint(branch),
                            Color::Purple.bold().paint("closed")
                        )
                        .as_str(),
                    )
                    .with_default(false)
                    .prompt()?;

                    if confirm {
                        self.delete_branch(branch, true)?;
                        num_closed += 1;
                    }
                }
            }
        }
        Ok(num_closed)
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
