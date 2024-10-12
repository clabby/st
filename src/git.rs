//! Utilities for interacting with `git` repositories for the `st` application.

use crate::constants::QUOTE_CHAR;
use anyhow::{anyhow, bail, Result};
use git2::{build::CheckoutBuilder, Branch, BranchType, Repository};
use nu_ansi_term::Color::Red;
use std::{env, process::Command};

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

    /// Pushes a branch to a registered remote.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to push.
    /// - `remote_name` - The name of the remote to push to.
    ///
    /// ## Returns
    /// - `Result<()>` - The result of the operation.
    fn push_branch(&self, branch_name: &str, remote_name: &str) -> Result<()>;
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
        self.checkout_branch(branch_name, Some(CheckoutBuilder::new().force()))?;

        execute_git_command(&["rebase", onto_name], false)
    }

    fn push_branch(&self, branch_name: &str, remote_name: &str) -> Result<()> {
        execute_git_command(&["push", remote_name, branch_name], false)
    }
}

/// Executes a `git` command with the given arguments in a blocking child task.
///
/// ## Takes
/// - `args` - The arguments to pass to the `git` command.
/// - `interactive` - Whether the command should be interactive.
fn execute_git_command(args: &[&str], interactive: bool) -> Result<()> {
    let mut cmd = Command::new("git");
    if interactive {
        let status = cmd.args(args).status()?;

        if !status.success() {
            bail!("Error executing git operation.");
        }
    } else {
        let output = cmd.args(args).output()?;

        if !output.status.success() {
            let git_error = String::from_utf8_lossy(&output.stderr)
                .trim_end_matches('\n')
                .replace("\n", &format!("\n{} ", QUOTE_CHAR))
                .replace("error: ", "");

            let error_message = format!("{} Git error:\n{} {}", QUOTE_CHAR, QUOTE_CHAR, git_error);
            bail!(
                "Error executing git operation.\n\n{}",
                Red.paint(error_message)
            );
        }
    }

    Ok(())
}
