//! Utilities for interacting with `git` repositories for the `st` application.

use anyhow::{anyhow, ensure, Result};
use git2::{build::CheckoutBuilder, Branch, BranchType, Config, Repository, Signature};
use std::{
    env,
    process::{Command, Stdio},
};

/// Returns the repository for the current working directory, and [None] if
/// the current working directory is not within a git repository or an error
/// occurs.
pub fn active_repository() -> Option<Repository> {
    Repository::discover(env::current_dir().ok()?).ok()
}

/// Returns the [Signature] for the committer.
pub fn committer_signature<'a>() -> Result<Signature<'a>> {
    let config = Config::open_default()?;
    let name = config.get_string("user.name")?;
    let email = config.get_string("user.email")?;

    Signature::now(name.as_str(), email.as_str()).map_err(Into::into)
}

/// Extension trait for the [Repository] type to expose helper functions related to
/// repository management.
pub trait RepositoryExt {
    /// Returns the name of the current branch.
    ///
    /// ## Returns
    /// - `Result<String>` - The name of the current branch, or an error.
    fn current_branch(&self) -> Result<Branch>;

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

    /// Rebases a branch onto another branch.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to rebase.
    /// - `onto` - The name of the branch to rebase onto.
    ///
    /// ## Returns
    /// - `Result<()>` - The result of the operation.
    fn rebase_branch_onto(&self, branch_name: &str, onto: &str) -> Result<()>;
}

impl RepositoryExt for Repository {
    fn current_branch(&self) -> Result<Branch> {
        let head = self.head()?;
        let branch = self.find_branch(
            head.name()
                .ok_or(anyhow!("HEAD ref does not have a name"))?
                .trim_start_matches("refs/heads/"),
            BranchType::Local,
        )?;
        Ok(branch)
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

    fn rebase_branch_onto(&self, branch_name: &str, onto_name: &str) -> Result<()> {
        // Check out the branch to rebase.
        self.checkout_branch(branch_name, None)?;

        // Run the interactive rebase with `git`. `git2` library does not support interactive
        // rebasing.
        let status = Command::new("git")
            .args(&["rebase", onto_name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        ensure!(
            status.success(),
            "Rebase failed. Resolve conflicts first and try again."
        );

        Ok(())
    }
}
