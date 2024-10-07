//! The data store for `st` configuration and stack state.

use crate::{
    constants::ST_STORE_FILE_NAME,
    git::RepositoryExt,
    stack::{LocalMetadata, StackedBranch, StackedBranchInner},
};
use anyhow::{anyhow, Result};
use git2::Repository;
use std::path::PathBuf;

/// The data store for `st` configuration, with its associated [Repository]
pub struct StoreWithRepository<'a> {
    /// The repository associated with the store.
    pub repository: &'a Repository,
    /// The store for the repository.
    pub stack: StackedBranch,
}

impl<'a> StoreWithRepository<'a> {
    /// Creates a new [StoreWithRepository] with the given [Repository] and trunk branch name.
    pub fn new(repository: &'a Repository, trunk: String) -> Self {
        let local_meta = LocalMetadata {
            branch_name: trunk,
            ..Default::default()
        };
        let branch = StackedBranchInner::new(local_meta, None);
        Self {
            repository,
            stack: StackedBranch::new(branch),
        }
    }

    /// Loads the root [StackNode] for the given [Repository], and assembles a [StoreWithRepository].
    pub fn try_load(repository: &'a Repository) -> Result<Option<Self>> {
        let store_path = store_path(&repository).ok_or(anyhow!("Store path not found"))?;

        // If the store doesn't exist, return None.
        if !store_path.exists() {
            return Ok(None);
        }

        let store: StackedBranch = toml::from_str(&std::fs::read_to_string(store_path)?)?;
        let mut store_with_repo = Self {
            repository,
            stack: store,
        };
        store_with_repo.prune()?;
        store_with_repo.write()?;

        Ok(Some(store_with_repo))
    }

    /// Updates the [StackNode] store with the current branches and their children. If any of the branches
    /// have been deleted, they are pruned from the store.
    pub fn prune(&mut self) -> Result<()> {
        let branches = self.stack.branches();

        for branch in branches {
            if self
                .repository
                .find_branch(branch.as_str(), git2::BranchType::Local)
                .is_err()
            {
                self.stack.delete_child(branch.as_str());
            }
        }

        Ok(())
    }

    /// Persists the [StackNode] to the given [Repository] on disk.
    pub fn write(&self) -> Result<()> {
        let store_path = store_path(&self.repository).ok_or(anyhow!("Store path not found."))?;
        let store = toml::to_string_pretty(&self.stack)?;
        std::fs::write(store_path, store)?;
        Ok(())
    }

    /// Returns the current stack node, if the current branch exists within a tracked stack.
    pub fn current_stack_node(&mut self) -> Option<StackedBranch> {
        let current_branch = self.repository.current_branch().ok()?;
        let current_branch_name = current_branch.name().ok()??;
        self.stack.find_child(current_branch_name)
    }
}

/// Returns the path to the [StackNode] store for the given [Repository].
///
/// ## Takes
/// - `repository` - The repository to get the store path for.
///
/// ## Returns
/// - `Some(PathBuf)` - The path to the store.
/// - `None` - If the repository does not have a workdir.
pub fn store_path<'a>(repository: &Repository) -> Option<PathBuf> {
    repository
        .workdir()
        .map(|p| p.join(".git").join(ST_STORE_FILE_NAME))
}
