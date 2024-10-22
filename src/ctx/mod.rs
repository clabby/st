//! The in-memory context of the `st` application.

use crate::{
    config::StConfig,
    constants::{GIT_DIR, ST_CTX_FILE_NAME},
    errors::{StError, StResult},
    tree::StackTree,
};
use git2::{BranchType, Repository};
use std::path::PathBuf;

mod actions;
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
pub fn ctx_path(repository: &Repository) -> Option<PathBuf> {
    repository
        .workdir()
        .map(|p| p.join(GIT_DIR).join(ST_CTX_FILE_NAME))
}

/// The in-memory context of the `st` application.
pub struct StContext<'a> {
    /// The global configuration for `st`.
    pub cfg: StConfig,
    /// The repository associated with the store.
    pub repository: &'a Repository,
    /// The tree of branches tracked by `st`.
    pub tree: StackTree,
}

impl<'a> StContext<'a> {
    /// Creates a fresh [StContext] with the given [Repository] and trunk branch name.
    pub fn fresh(cfg: StConfig, repository: &'a Repository, trunk: String) -> Self {
        Self {
            cfg,
            repository,
            tree: StackTree::new(trunk),
        }
    }

    /// Loads the [StackTree] for the given [Repository], and assembles a [StContext].
    pub fn try_load(cfg: StConfig, repository: &'a Repository) -> StResult<Option<Self>> {
        let store_path = ctx_path(repository).ok_or(StError::GitRepositoryRootNotFound)?;

        // If the store doesn't exist, return None.
        if !store_path.exists() {
            return Ok(None);
        }

        let stack: StackTree = toml::from_str(&std::fs::read_to_string(store_path)?)?;
        let mut store_with_repo = Self {
            cfg,
            repository,
            tree: stack,
        };
        store_with_repo.prune()?;

        Ok(Some(store_with_repo))
    }

    /// Parses the GitHub owner and repository from the current repository's remote URL.
    pub fn owner_and_repository(&self) -> StResult<(String, String)> {
        let remote = self.repository.find_remote("origin")?;
        let url = remote
            .url()
            .ok_or(StError::RemoteNotFound("origin".to_string()))?;

        let (org, repo) = if url.starts_with("git@") {
            // Handle SSH URL: git@github.com:org/repo.git
            let parts = url.split(':').collect::<Vec<_>>();
            let repo_parts = parts
                .get(1)
                .ok_or(StError::DecodingError(
                    "Invalid SSH URL format.".to_string(),
                ))?
                .split('/')
                .collect::<Vec<_>>();
            let org = repo_parts.first().ok_or(StError::DecodingError(
                "Organization not found.".to_string(),
            ))?;
            let repo = repo_parts.get(1).ok_or(StError::DecodingError(
                "Repository not found while decoding remote URL.".to_string(),
            ))?;
            (org.to_string(), repo.trim_end_matches(".git").to_string())
        } else if url.starts_with("https://") {
            // Handle HTTPS URL: https://github.com/org/repo.git
            let parts = url.split('/').collect::<Vec<_>>();
            let org = parts.get(parts.len() - 2).ok_or(StError::DecodingError(
                "Organization not found.".to_string(),
            ))?;
            let repo = parts.last().ok_or(StError::DecodingError(
                "Repository not found while decoding remote URL.".to_string(),
            ))?;
            (org.to_string(), repo.trim_end_matches(".git").to_string())
        } else {
            return Err(StError::DecodingError(
                "Unsupported remote URL format.".to_string(),
            ));
        };

        Ok((org, repo))
    }

    /// Prunes branches in the context that no longer exist in the git repository.
    fn prune(&mut self) -> StResult<()> {
        let branches = self.tree.branches()?;
        branches.iter().try_for_each(|b| {
            if self.repository.find_branch(b, BranchType::Local).is_err() {
                self.tree.delete(b)?;
            }
            Ok::<_, StError>(())
        })
    }
}

impl Drop for StContext<'_> {
    fn drop(&mut self) {
        // Persist the store on drop.
        let store_path = ctx_path(self.repository).expect("Failed to get context path.");
        let store = toml::to_string_pretty(&self.tree).expect("Failed to serialize context.");
        std::fs::write(store_path, store).expect("Failed to persist context to disk.");
    }
}
