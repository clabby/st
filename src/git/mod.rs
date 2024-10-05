//! Utilities for interacting with `git` repositories for the `st` application.

use anyhow::{anyhow, Result};
use git2::{build::CheckoutBuilder, BranchType, Repository};
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
    /// Returns the name of the current branch.
    ///
    /// ## Returns
    /// - `Result<String>` - The name of the current branch, or an error.
    fn current_branch(&self) -> Result<String>;

    /// Checks out a branch with the given `branch_name`.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to checkout.
    /// - `opts` - The checkout options to use.
    ///
    /// ## Returns
    /// - `Result<()>` - The result of the operation.
    fn checkout_branch(
        &self,
        branch_name: &str,
        opts: Option<&mut CheckoutBuilder<'_>>,
    ) -> Result<()>;
}

impl RepositoryExt for Repository {
    fn current_branch(&self) -> Result<String> {
        let head = self.head()?;
        let branch = self.find_branch(
            head.name()
                .ok_or(anyhow!("HEAD ref does not have a name"))?
                .trim_start_matches("refs/heads/"),
            BranchType::Local,
        )?;
        let branch_name = branch
            .name()?
            .ok_or(anyhow!("Name of current branch not found"))?;

        Ok(branch_name.to_string())
    }

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
