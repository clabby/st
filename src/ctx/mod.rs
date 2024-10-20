//! The in-memory context of the `st` application.

use crate::{
    constants::{GIT_DIR, ST_CTX_FILE_NAME},
    git::RepositoryExt,
    stack::{DisplayBranch, StackTree, TrackedBranch},
};
use anyhow::{anyhow, Result};
use git2::{BranchType, Repository};
use nu_ansi_term::Color::{Cyan, Green};
use std::{collections::VecDeque, fmt::Write, path::PathBuf};

/// Returns the path to the persistent application context for the given [Repository].
///
/// ## Takes
/// - `repository` - The repository to get the context path for.
///
/// ## Returns
/// - `Some(PathBuf)` - The path to the serialized context.
/// - `None` - If the repository does not have a workdir.
pub fn ctx_path<'a>(repository: &Repository) -> Option<PathBuf> {
    repository
        .workdir()
        .map(|p| p.join(GIT_DIR).join(ST_CTX_FILE_NAME))
}

/// The in-memory context of the `st` application.
pub struct StContext<'a> {
    /// The repository associated with the store.
    pub repository: &'a Repository,
    /// The tree of branches tracked by `st`.
    pub tree: StackTree,
}

impl<'a> StContext<'a> {
    /// Creates a fresh [StContext] with the given [Repository] and trunk branch name.
    pub fn fresh(repository: &'a Repository, trunk: String) -> Self {
        Self {
            repository,
            tree: StackTree::new(trunk),
        }
    }

    /// Loads the root [StackNode] for the given [Repository], and assembles a [StoreWithRepository].
    pub fn try_load(repository: &'a Repository) -> Result<Option<Self>> {
        let store_path = ctx_path(&repository).ok_or(anyhow!("Store path not found"))?;

        // If the store doesn't exist, return None.
        if !store_path.exists() {
            return Ok(None);
        }

        let stack: StackTree = toml::from_str(&std::fs::read_to_string(store_path)?)?;
        let mut store_with_repo = Self {
            repository,
            tree: stack,
        };
        store_with_repo.prune()?;

        Ok(Some(store_with_repo))
    }

    /// Returns the checked out [TrackedBranch].
    pub fn current_branch(&self) -> Option<&TrackedBranch> {
        let current_branch_name = self.current_branch_name()?;
        self.tree.get(current_branch_name.as_str())
    }

    /// Returns the checked out [TrackedBranch]'s name.
    pub fn current_branch_name(&self) -> Option<String> {
        let current_branch = self.repository.current_branch().ok()?;
        let current_branch_name = current_branch.name().ok()??;
        Some(current_branch_name.to_string())
    }

    /// Attempts to resolve the current stack in full. In the worst case, there is a fork upstack,
    /// in which case this function will terminate at the fork, as it can not determine which
    /// path to take.
    pub fn resolve_active_stack(&self) -> Result<Vec<String>> {
        let mut stack = VecDeque::new();
        let current = self.current_branch();

        // Resolve downstack
        let mut current_name = current.map(|c| &c.local.branch_name);
        while let Some(name) = current_name {
            stack.push_front(name.clone());
            current_name = self.tree.get(name.as_str()).and_then(|c| c.parent.as_ref())
        }

        // Resolve upstack
        let mut current_children = current.map(|c| &c.children);
        while let Some(children) = current_children {
            // If there are multiple children, we have a fork, and we can not continue resolving the stack.
            if children.len() != 1 {
                break;
            }

            let child_name = children.iter().next().expect("Cannot be empty");
            stack.push_back(child_name.clone());

            current_children = self.tree.get(child_name.as_str()).map(|c| &c.children);
        }

        Ok(stack.into())
    }

    /// Restacks the active stack.
    pub fn restack_current(&mut self) -> Result<()> {
        let mut stack = self.resolve_active_stack()?;

        for current in stack.iter_mut() {
            let current = self
                .tree
                .get_mut(current.as_str())
                .ok_or(anyhow!("Branch not found."))?;

            let current_name = current.local.branch_name.as_str();
            let parent_name = current
                .parent
                .clone()
                .ok_or(anyhow!("Attempted to restack trunk."))?;

            let parent_ref = self
                .repository
                .find_branch(parent_name.as_str(), BranchType::Local)?;
            let parent_ref_str = parent_ref
                .get()
                .target()
                .ok_or(anyhow!("Parent ref target not found"))?
                .to_string();

            if current
                .local
                .parent_oid_cache
                .as_ref()
                .map(|pid| pid == &parent_ref_str)
                .unwrap_or_default()
            {
                println!(
                    "Branch `{}` does not need to be restacked.",
                    Green.paint(current_name),
                );
                continue;
            }

            // Attempt to rebase the current branch onto the parent branch.
            self.repository
                .rebase_branch_onto(current_name, parent_name.as_str())?;

            println!(
                "Restacked `{}` on `{}`.",
                Green.paint(current_name),
                Cyan.paint(parent_name)
            );

            // Update the parent oid cache.
            current.local.parent_oid_cache = Some(parent_ref_str);
        }

        Ok(())
    }

    /// Parses the GitHub organization and repository from the current repository's remote URL.
    pub fn org_and_repository(&self) -> Result<(String, String)> {
        let remote = self.repository.find_remote("origin")?;
        let url = remote.url().ok_or(anyhow!("Remote URL not found."))?;

        let (org, repo) = if url.starts_with("git@") {
            // Handle SSH URL: git@github.com:org/repo.git
            let parts = url.split(':').collect::<Vec<_>>();
            let repo_parts = parts
                .get(1)
                .ok_or(anyhow!("Invalid SSH URL format."))?
                .split('/')
                .collect::<Vec<_>>();
            let org = repo_parts
                .get(0)
                .ok_or(anyhow!("Organization not found."))?;
            let repo = repo_parts.get(1).ok_or(anyhow!("Repository not found."))?;
            (org.to_string(), repo.trim_end_matches(".git").to_string())
        } else if url.starts_with("https://") {
            // Handle HTTPS URL: https://github.com/org/repo.git
            let parts = url.split('/').collect::<Vec<_>>();
            let org = parts
                .get(parts.len() - 2)
                .ok_or(anyhow!("Organization not found."))?;
            let repo = parts
                .get(parts.len() - 1)
                .ok_or(anyhow!("Repository not found."))?;
            (org.to_string(), repo.trim_end_matches(".git").to_string())
        } else {
            return Err(anyhow!("Unsupported remote URL format."));
        };

        Ok((org, repo))
    }

    /// Returns a vector of branch names within the [StackedBranch].
    pub fn branches(&self) -> Result<Vec<String>> {
        let mut branches = Vec::default();
        self.tree
            .trunk()
            .fill_branches(&self.tree.branches, &mut branches)?;
        Ok(branches)
    }

    /// Returns a vector of [DisplayBranch]es for the stack node and its children.
    ///
    /// ## Takes
    /// - `checked_out` - The name of the branch that is currently checked out.
    ///                   If [None], the current branch is not highlighted.
    ///
    /// ## Returns
    /// - `Ok(Vec<DisplayBranch>)` - The branches of the stack node and its children,
    ///                              in the order they are logged.
    /// - `Err(_)` - If an error occurs while gathering the [DisplayBranch]es.
    pub fn display_branches(&self, checked_out: Option<&str>) -> Result<Vec<DisplayBranch>> {
        // Collect the branch names.
        let branches = self.branches()?;

        // Write the log of the stacks.
        let mut buf = String::new();
        self.write_tree(&mut buf, checked_out)?;

        // Zip the pretty-printed tree with the branch names to assemble the DisplayBranches.
        let items = buf
            .lines()
            .filter(|l| !l.is_empty())
            .zip(branches.iter())
            .map(|(line, branch_name)| DisplayBranch {
                line: line.to_string(),
                branch_name: branch_name.to_string(),
            })
            .collect::<Vec<_>>();

        Ok(items)
    }

    /// Writes a pretty-printed representation of the [StackedBranch] tree to the passed [Write]r.
    ///
    /// ## Takes
    /// - `w` - The writer to write the log to.
    /// - `checked_out` - The name of the branch that is currently checked out.
    ///
    /// ## Returns
    /// - `Ok(_)` - Tree successfully written.
    /// - `Err(_)` - If an error occurs while writing the Tree.
    pub fn write_tree<W: Write>(&self, w: &mut W, checked_out: Option<&str>) -> Result<()> {
        // Find all nodes that need to be restacked.
        let needs_restack = self
            .resolve_active_stack()?
            .iter()
            .filter_map(|b| {
                let b = self.tree.get(b)?;

                let parent_branch_name = b.parent.as_ref()?;
                let parent_branch = self
                    .repository
                    .find_branch(parent_branch_name, BranchType::Local);
                let parent_ref = parent_branch.ok()?.get().target()?.to_string();

                (b.local
                    .parent_oid_cache
                    .as_ref()
                    .map(|pid| pid != &parent_ref)
                    .unwrap_or_default())
                .then(|| b.local.branch_name.clone())
            })
            .collect::<Vec<_>>();

        self.tree.trunk().write_tree_recursive(
            w,
            &self.tree.branches,
            checked_out.unwrap_or_default(),
            needs_restack.as_slice(),
            0,
            true,
            Default::default(),
            Default::default(),
        )
    }

    /// Deletes branches within the context that no longer exist in the repository.
    fn prune(&mut self) -> Result<()> {
        let branches = self.branches()?;

        for branch in branches {
            if self
                .repository
                .find_branch(branch.as_str(), BranchType::Local)
                .is_err()
            {
                self.tree.delete(branch.as_str());
            }
        }

        Ok(())
    }
}

impl<'a> Drop for StContext<'a> {
    fn drop(&mut self) {
        // Persist the store on drop.
        let store_path = ctx_path(&self.repository).expect("Failed to get context path.");
        let store = toml::to_string_pretty(&self.tree).expect("Failed to serialize context.");
        std::fs::write(store_path, store).expect("Failed to persist context to disk.");
    }
}
