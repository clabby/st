//! The in-memory context of the `st` application.

use crate::{
    constants::{GIT_DIR, ST_CTX_FILE_NAME},
    tree::StackTree,
};
use anyhow::{anyhow, Result};
use git2::Repository;
use std::path::PathBuf;

mod fmt;
mod stack_management;

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
        let store_with_repo = Self {
            repository,
            tree: stack,
        };

        Ok(Some(store_with_repo))
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
