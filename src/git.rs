//! Utilities for interacting with `git` repositories for the `st` application.

use anyhow::{anyhow, ensure, Result};
use git2::{
    build::CheckoutBuilder, Branch, BranchType, Config, RebaseOptions, Repository, Signature,
};
use std::env;

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
    fn rebase_branch_onto(
        &self,
        branch_name: &str,
        onto: &str,
        opts: Option<&mut RebaseOptions>,
    ) -> Result<()>;
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

    fn rebase_branch_onto(
        &self,
        branch_name: &str,
        onto: &str,
        opts: Option<&mut RebaseOptions>,
    ) -> Result<()> {
        let branch = self.find_branch(branch_name, BranchType::Local)?;
        let onto = self.find_branch(onto, BranchType::Local)?;

        let annotated_current = self.find_annotated_commit(branch.get().peel_to_commit()?.id())?;
        let annotated_parent = self.find_annotated_commit(onto.get().peel_to_commit()?.id())?;

        let mut rebase = self.rebase(
            Some(&annotated_current),
            None,
            Some(&annotated_parent),
            opts,
        )?;

        let sig = committer_signature()?;
        while let Some(op) = rebase.next() {
            let index = self.index()?;

            ensure!(
                !index.has_conflicts(),
                "Conflicts detected. Resolve them first."
            );

            // Commit the operation only if necessary (e.g., in case of modifications)
            let kind = op?
                .kind()
                .ok_or(anyhow!("Rebase operation kind not found"))?;
            if matches!(kind, git2::RebaseOperationType::Pick) {
                rebase.commit(None, &sig, None)?;
            }
        }

        rebase.finish(Some(&sig))?;

        // Update the branch reference to point to the rebased head.
        let rebased_head = self.head()?.peel_to_commit()?;
        branch
            .into_reference()
            .set_target(rebased_head.id(), "rebase")?;

        Ok(())
    }
}
