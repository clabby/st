//! Structured, [Serialize] + [Deserialize] representation of a stack of branches.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// An n-nary tree of branches, represented as a flat data structure.
///
/// By itself, [StackTree] has no context of its relationship with the local repository. For this functionality,
/// [StContext] holds onto both the [StackTree] and the [Repository]
///
/// [StContext]: crate::ctx::StContext
/// [Repository]: git2::Repository
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StackTree {
    /// The name of the trunk branch.
    pub trunk_name: String,
    /// A map of branch names to [TrackedBranch]es.
    pub branches: HashMap<String, TrackedBranch>,
}

impl StackTree {
    /// Creates a new [StackTree] with the given trunk branch name.
    pub fn new(trunk_name: String) -> Self {
        let branches = HashMap::from([(
            trunk_name.clone(),
            TrackedBranch::new(trunk_name.clone(), None, None),
        )]);

        Self {
            trunk_name,
            branches,
        }
    }

    /// Gets the trunk branch from the stack graph.
    ///
    /// ## Panics
    /// - If the trunk branch does not exist.
    pub fn trunk(&self) -> &TrackedBranch {
        self.branches.get(&self.trunk_name).unwrap()
    }

    /// Gets a branch by name from the stack graph.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to get.
    ///
    /// ## Returns
    /// - `Some(branch)` - The branch.
    /// - `None` - The branch by the name of `branch_name` was not found.
    pub fn get(&self, branch_name: &str) -> Option<&TrackedBranch> {
        self.branches.get(branch_name)
    }

    /// Gets a mutable branch by name from the stack graph.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to get.
    ///
    /// ## Returns
    /// - `Some(branch)` - The branch.
    /// - `None` - The branch by the name of `branch_name` was not found.
    pub fn get_mut(&mut self, branch_name: &str) -> Option<&mut TrackedBranch> {
        self.branches.get_mut(branch_name)
    }

    /// Adds a child branch to the passed parent branch, if it exists.
    ///
    /// ## Takes
    /// - `parent` - The name of the parent branch.
    /// - `local_metadata` - The [LocalMetadata] for the child branch.
    ///
    /// ## Returns
    /// - `Ok(()` if the child branch was successfully added.)`
    /// - `Err(_)` if the parent branch does not exist.
    pub fn insert(
        &mut self,
        parent_name: &str,
        parent_oid_cache: &str,
        branch_name: &str,
    ) -> Result<()> {
        // Get the parent branch.
        let parent = self.branches.get_mut(parent_name).ok_or(anyhow!(
            "Parent branch {} is not tracked with `st`. Track it first with `st track`.",
            parent_name
        ))?;

        // Register the child branch with the parent.
        parent.children.insert(branch_name.to_string());

        // Create the child branch.
        let child = TrackedBranch::new(
            branch_name.to_string(),
            Some(parent_name.to_string()),
            Some(parent_oid_cache.to_string()),
        );
        self.branches.insert(branch_name.to_string(), child);

        Ok(())
    }

    /// Deletes a branch from the stack graph. If the branch does not exist, returns [None].
    ///
    /// ## Takes
    /// - `branch` - The name of the branch to delete.
    ///
    /// ## Returns
    /// - `Some(branch)` - The deleted branch.
    /// - `None` - The branch by the name of `branch` was not found.
    pub fn delete(&mut self, branch_name: &str) -> Option<TrackedBranch> {
        // Remove the branch from the stack tree.
        let branch = self.branches.remove(branch_name)?;

        // Remove the child from the parent's children list.
        if let Some(ref parent) = branch.parent {
            let parent_branch = self.branches.get_mut(parent)?;
            parent_branch.children.remove(branch_name);

            // Re-link the children of the deleted branch to the parent.
            branch
                .children
                .iter()
                .try_for_each(|child_name| {
                    // Change the pointer of the child to the parent.
                    let child = self
                        .branches
                        .get_mut(child_name)
                        .ok_or(anyhow!("Child does not exist"))?;
                    child.parent = branch.parent.clone();

                    // Add the child to the parent's children list.
                    let parent = self
                        .branches
                        .get_mut(parent)
                        .ok_or(anyhow!("Parent does not exist"))?;
                    parent.children.insert(child_name.clone());
                    Ok::<_, anyhow::Error>(())
                })
                .ok()?;
        }

        Some(branch)
    }

    /// Returns a vector of branch names in the stack graph. The vector is filled recursively, meaning that children are
    /// guaranteed to be listed after their parents.
    pub fn branches(&self) -> Result<Vec<String>> {
        let mut branch_names = Vec::new();
        self.fill_branches(&self.trunk_name, &mut branch_names)?;
        Ok(branch_names)
    }

    /// Fills a vector with the trunk branch and its children. The resulting vector is filled recursively, meaning that
    /// children are guaranteed to be listed after their parents.
    fn fill_branches(&self, name: &str, branch_names: &mut Vec<String>) -> Result<()> {
        let current = self
            .branches
            .get(name)
            .ok_or(anyhow!("Branch {} is not tracked with `st`.", name))?;

        branch_names.push(current.name.clone());
        current
            .children
            .iter()
            .try_for_each(|child| self.fill_branches(child, branch_names))
    }
}

/// A local branch tracked by `st`.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TrackedBranch {
    /// The branch name.
    pub name: String,
    /// The parent branch's [git2::Oid] cache, in string form.
    ///
    /// Invalid iff the parent branch's `HEAD` commit is not equal to the [git2::Oid] cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_oid_cache: Option<String>,
    /// The index of the parent branch in the stack graph.
    ///
    /// [None] if the branch is trunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// The index of the child branches within the stack graph.
    pub children: HashSet<String>,
    /// The [RemoteMetadata] for the branch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<RemoteMetadata>,
}

impl TrackedBranch {
    /// Creates a new [TrackedBranch] with the given local metadata and parent branch name.
    ///
    /// Upon local instantiation, the branch has children or remote metadata.
    pub fn new(
        branch_name: String,
        parent: Option<String>,
        parent_oid_cache: Option<String>,
    ) -> Self {
        Self {
            name: branch_name,
            parent,
            parent_oid_cache,
            ..Default::default()
        }
    }
}

/// Remote metadata for a branch that is tracked by `st`.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteMetadata {
    /// The number of the pull request on GitHub associated with the branch.
    pub(crate) pr_number: u64,
    /// The comment ID of the stack status comment on the pull request.
    ///
    /// This is used to update the comment with the latest stack status each time the stack
    /// is submitted.
    pub(crate) comment_id: Option<u64>,
}

impl RemoteMetadata {
    /// Creates a new [RemoteMetadata] with the given PR number and comment ID.
    pub fn new(pr_number: u64) -> Self {
        Self {
            pr_number,
            comment_id: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::StackTree;

    #[test]
    fn insert_new_branch() {
        let mut tree = StackTree::new("main".to_string());

        tree.insert("main", Default::default(), "feature_branch")
            .unwrap();

        let feature_branch = tree.get("feature_branch").unwrap();
        assert_eq!(feature_branch.parent.clone().unwrap(), "main".to_string());
    }
}
