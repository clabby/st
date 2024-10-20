//! Structured, [Serialize] + [Deserialize] representation of a stack of branches.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod fmt;
pub(crate) use fmt::DisplayBranch;

/// An n-nary tree of branches, represented as a flat data structure.
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
            TrackedBranch::new(LocalMetadata::new(trunk_name.clone(), None), None),
        )]);

        Self { trunk_name, branches }
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
    pub fn insert(&mut self, parent_name: &str, local_metadata: LocalMetadata) -> Result<()> {
        // Get the parent branch.
        let parent = self
            .branches
            .get_mut(parent_name)
            .ok_or(anyhow!("Parent does not exist"))?;

        // Get the name of the child branch.
        let child_branch_name = local_metadata.branch_name.clone();

        // Register the child branch with the parent.
        parent.children.insert(child_branch_name.clone());

        // Create the child branch.
        let child = TrackedBranch::new(local_metadata, Some(parent_name.to_string()));

        self.branches.insert(child_branch_name, child);
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
        let branch = self.branches.remove(branch_name)?;

        // Remove the child from the parent's children list.
        if let Some(ref parent) = branch.parent {
            let parent_branch = self.branches.get_mut(parent)?;

            parent_branch.children.remove(branch_name);
        }

        // Re-link the children of the deleted branch to the parent.
        branch
            .children
            .iter()
            .try_for_each(|child_name| {
                let child = self
                    .branches
                    .get_mut(child_name)
                    .ok_or(anyhow!("Child does not exist"))?;
                child.parent = branch.parent.clone();
                Ok::<_, anyhow::Error>(())
            })
            .ok()?;

        Some(branch)
    }
}

/// A local branch tracked by `st`.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TrackedBranch {
    /// The index of the parent branch in the stack graph.
    ///
    /// [None] if the branch is trunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// The index of the child branches within the stack graph.
    pub children: HashSet<String>,
    /// The [LocalMetadata] for the branch.
    pub local: LocalMetadata,
    /// The [RemoteMetadata] for the branch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<RemoteMetadata>,
}

impl TrackedBranch {
    /// Creates a new [TrackedBranch] with the given local metadata and parent branch name.
    ///
    /// Upon instantiation, the branch has children or remote metadata.
    pub fn new(local: LocalMetadata, parent: Option<String>) -> Self {
        Self {
            parent,
            local,
            ..Default::default()
        }
    }
}

/// Local metadata for a branch that is tracked by `st`.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LocalMetadata {
    /// The name of the branch.
    pub(crate) branch_name: String,
    /// The cached [git2::Oid] of the parent's target ref, in [String] form.
    ///
    /// Valid iff the parent branch's target ref is a commit with an equivalent [git2::Oid].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_oid_cache: Option<String>,
}

impl LocalMetadata {
    /// Creates a new [LocalMetadata] with the given branch name and parent OID cache.
    pub fn new(branch_name: String, parent_oid_cache: Option<String>) -> Self {
        Self {
            branch_name,
            parent_oid_cache,
        }
    }
}

/// Remote metadata for a branch that is tracked by `st`.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteMetadata {
    /// The number of the pull request associated with the branch.
    pub(crate) pr_number: u64,
    /// The comment ID of the stack status comment on the pull request.
    pub(crate) comment_id: u64,
}

impl RemoteMetadata {
    /// Creates a new [RemoteMetadata] with the given PR number and comment ID.
    pub fn new(pr_number: u64, comment_id: u64) -> Self {
        Self {
            pr_number,
            comment_id,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::stack::LocalMetadata;

    use super::StackTree;

    #[test]
    fn insert_new_branch() {
        let mut tree = StackTree::new("main".to_string());

        tree.insert("main", LocalMetadata::new("feature_branch".to_string(), None)).unwrap();

        let feature_branch = tree.get("feature_branch").unwrap();
        assert_eq!(feature_branch.parent.clone().unwrap(), "main".to_string());
    }
}
