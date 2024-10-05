//! The data store for `st` configuration and stack state.

use anyhow::{anyhow, Result};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The data store for `st` configuration, with its associated [Repository]
pub struct StoreWithRepository<'a> {
    /// The repository associated with the store.
    pub repository: &'a Repository,
    /// The store for the repository.
    pub store: Store,
}

impl<'a> StoreWithRepository<'a> {
    /// Loads the [Store] for the given [Repository].
    pub fn try_load(repository: &'a Repository) -> Result<Self> {
        let store_path = store_path(&repository).ok_or(anyhow!("Store path not found"))?;
        let ser_store = std::fs::read_to_string(store_path)?;
        let store: Store = toml::from_str(&ser_store)?;

        Ok(Self { repository, store })
    }

    /// Writes the [Store] to the given [Repository].
    pub fn write(&self) -> Result<()> {
        let store_path = store_path(&self.repository).ok_or(anyhow!("Store path not found."))?;
        let store = toml::to_string_pretty(&self.store)?;
        std::fs::write(store_path, store)?;
        Ok(())
    }
}

/// The data store for `st` stack state for a given repository.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Store {
    /// The trunk branch for the repository.
    pub trunk: String,
    /// The stacks within the store.
    pub stacks: Vec<Stack>,
}

/// A stack of branches.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Stack {
    /// The branches within the stack.
    pub branches: Vec<String>,
}

/// Returns the path to the [Store] for the given [Repository].
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
        .map(|p| p.join(".git").join(".st_store.toml"))
}
