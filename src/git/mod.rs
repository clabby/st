//! Utilities for interacting with `git` repositories for the `st` application.

use anyhow::Result;
use git2::{build::CheckoutBuilder, Repository};
use std::env;

/// Returns the repository for the current working directory, and [None] if
/// the current working directory is not within a git repository or an error
/// occurs.
pub fn active_repository() -> Option<Repository> {
    Repository::discover(env::current_dir().ok()?).ok()
}

/// Extension trait for the [Repository] type to expose helper functions related to
/// repository management.
pub trait RepositoryExt {
    /// Checks out a branch with the given `branch_name`.
    fn checkout_branch(
        &self,
        branch_name: &str,
        opts: Option<&mut CheckoutBuilder<'_>>,
    ) -> Result<()>;
}

impl RepositoryExt for Repository {
    fn checkout_branch(
        &self,
        branch_name: &str,
        opts: Option<&mut CheckoutBuilder<'_>>,
    ) -> Result<()> {
        self.set_head(format!("refs/heads/{}", branch_name).as_str())?;
        self.checkout_head(opts)?;

        Ok(())
    }
}
