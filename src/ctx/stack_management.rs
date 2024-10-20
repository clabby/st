//! Stack management functionality for [StContext].

use super::StContext;
use crate::git::RepositoryExt;
use anyhow::{anyhow, Result};
use git2::BranchType;
use nu_ansi_term::Color;
use std::collections::VecDeque;

impl<'a> StContext<'a> {
    /// Discovers the current stack, relative to the checked out branch, including the trunk branch.
    ///
    /// The returned stack is ordered from the trunk branch to the tip of the stack.
    pub fn discover_stack(&self) -> Result<Vec<String>> {
        let mut stack = VecDeque::new();

        // Get the current branch name.
        let current_branch = self.repository.current_branch_name()?;
        let current_tracked_branch = self.tree.get(&current_branch).ok_or(anyhow!(
            "Branch {} is not tracked with `st`.",
            current_branch
        ))?;

        // Resolve upstack.
        let mut upstack = current_tracked_branch.parent.as_ref();
        while let Some(parent) = upstack {
            stack.push_front(parent.clone());
            upstack = self
                .tree
                .get(parent)
                .ok_or(anyhow!("Parent branch not found"))?
                .parent
                .as_ref();
        }

        // Push the curent branch onto the stack.
        stack.push_back(current_branch);

        // Attempt to resolve downstack. If there are multiple children, then the stack is ambiguous,
        // and we end resolution at the fork.
        let mut downstack = Some(&current_tracked_branch.children);
        while let Some(children) = downstack {
            // End resolution if there are multiple or no children.
            if children.len() != 1 {
                break;
            }

            // Push the child onto the stack.
            let child_branch = children.iter().next().expect("Single child must exist");
            stack.push_back(child_branch.clone());

            // Continue resolution if the child has children of its own.
            downstack = self.tree.get(child_branch).map(|b| &b.children);
        }

        Ok(stack.into())
    }

    /// Returns whether or not a given branch needs to be restacked onto its parent.
    pub fn needs_restack(&self, branch_name: &str) -> Result<bool> {
        let branch = self
            .tree
            .get(branch_name)
            .ok_or(anyhow!("Branch {} is not tracked with `st`.", branch_name))?;

        // If the branch does not have a parent, then it is trunk and never needs to be restacked.
        let Some(ref parent_name) = branch.parent else {
            return Ok(false);
        };

        let parent_oid = self
            .repository
            .find_branch(parent_name.as_str(), BranchType::Local)?
            .get()
            .target()
            .ok_or(anyhow!(
                "Parent branch {} does not have a commit.",
                parent_name
            ))?;
        let parent_oid_cache = branch.parent_oid_cache.as_ref().ok_or(anyhow!(
            "Parent branch {} does not have a cached commit.",
            parent_name
        ))?;

        // If the parent oid cache is invalid, or the parent needs to be restacked, then the branch
        // needs to be restacked.
        Ok(&parent_oid.to_string() != parent_oid_cache || self.needs_restack(parent_name)?)
    }

    /// Performs a restack of the active stack.
    pub fn restack(&mut self) -> Result<()> {
        // Discover the current stack.
        let stack = self.discover_stack()?;

        // Rebase each branch onto its parent.
        for (i, branch) in stack.iter().enumerate().skip(1) {
            // Skip branches that do not need to be restacked.
            if !self.needs_restack(branch)? {
                println!(
                    "Branch `{}` does not need to be restacked onto `{}`.",
                    Color::Green.paint(branch),
                    Color::Yellow.paint(&stack[i - 1])
                );
                continue;
            }

            // Rebase the branch onto its parent.
            self.repository.rebase_branch_onto(branch, &stack[i - 1])?;

            // Update the parent oid cache.
            let parent_oid = self
                .repository
                .find_branch(&stack[i - 1], BranchType::Local)?
                .get()
                .target()
                .ok_or(anyhow!(
                    "Parent branch {} does not have a commit.",
                    &stack[i - 1]
                ))?;
            self.tree
                .get_mut(branch)
                .ok_or(anyhow!("Branch {} is not tracked with `st`.", branch))?
                .parent_oid_cache = Some(parent_oid.to_string());

            println!(
                "Restacked branch `{}` onto `{}`.",
                Color::Green.paint(branch),
                Color::Yellow.paint(&stack[i - 1])
            );
        }

        Ok(())
    }
}
