//! Utilities for interacting with `git` repositories for the `st` application.

use crate::constants::QUOTE_CHAR;
use git2::{
    build::CheckoutBuilder, Branch, BranchType, ErrorClass, ErrorCode, Repository, StatusOptions,
};
use nu_ansi_term::Color::Red;
use std::{env, process::Command};
use thiserror::Error;

/// Returns the repository for the current working directory, and [None] if
/// the current working directory is not within a git repository or an error
/// occurs.
pub fn active_repository() -> Option<Repository> {
    Repository::discover(env::current_dir().ok()?).ok()
}

/// Extension trait for the [Repository] type to expose helper functions related to
/// repository management.
pub trait RepositoryExt {
    /// Returns the current [Branch].
    ///
    /// ## Returns
    /// - `Result<Branch>` - The current [Branch], or an error.
    fn current_branch(&self) -> Result<Branch, git2::Error>;

    /// Returns the name of the current [Branch].
    ///
    /// ## Returns
    /// - `Result<String>` - The name of the current branch, or an error.
    fn current_branch_name(&self) -> Result<String, git2::Error>;

    /// Returns whether or not the working tree is clean.
    ///
    /// ## Returns
    /// - `Result<bool>` - True if the working tree is clean, false otherwise.
    fn is_working_tree_clean(&self) -> Result<bool, git2::Error>;

    /// Checks out a branch with the given `branch_name`.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to checkout.
    /// - `opts` - The checkout options to use.
    ///
    /// ## Returns
    /// - `Result<()>` - The result of the operation.
    fn checkout_branch(&self, branch_name: &str) -> Result<(), git2::Error>;

    /// Set the target of a branch to its upstream [git2::Reference].
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to set ref for.
    ///
    /// ## Returns
    /// - `Result<()>` - The result of the operation.
    fn set_target_to_upstream_ref(
        &self,
        branch_name: &str,
        remote_name: &str,
    ) -> Result<(), git2::Error>;

    /// Rebases a branch onto another branch.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to rebase.
    /// - `onto` - The name of the branch to rebase onto.
    ///
    /// ## Returns
    /// - `Result<()>` - The result of the operation.
    fn rebase_branch_onto(&self, branch_name: &str, onto: &str) -> Result<(), GitCommandError>;

    /// Aborts a rebase in progress.
    ///
    /// ## Returns
    /// - `Result<()>` - The result of the operation.
    fn abort_rebase(&self) -> Result<(), GitCommandError>;

    /// Pushes a branch to a registered remote.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to push.
    /// - `remote_name` - The name of the remote to push to.
    /// - `force` - Whether to force push.
    ///
    /// ## Returns
    /// - `Result<()>` - The result of the operation.
    fn push_branch(
        &self,
        branch_name: &str,
        remote_name: &str,
        force: bool,
    ) -> Result<(), GitCommandError>;

    /// Pulls a branch from a registered remote.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to push.
    /// - `remote_name` - The name of the remote to push to.
    ///
    /// ## Returns
    /// - `Result<()>` - The result of the operation.
    fn pull_branch(&self, branch_name: &str, remote_name: &str) -> Result<(), GitCommandError>;

    /// Returns whether the local branch is ahead of the remote branch.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the local branch.
    ///
    /// ## Returns
    /// - `Result<bool>` - Whether the local branch is ahead of the remote branch.
    fn ahead_and_behind_upstream(&self, branch_name: &str) -> Result<(usize, usize), git2::Error>;
}

impl RepositoryExt for Repository {
    fn current_branch(&self) -> Result<Branch, git2::Error> {
        let head = self.head()?;
        let branch = self.find_branch(
            head.name()
                .ok_or(git2::Error::new(
                    ErrorCode::GenericError,
                    ErrorClass::Object,
                    "HEAD name not found",
                ))?
                .trim_start_matches("refs/heads/"),
            BranchType::Local,
        )?;
        Ok(branch)
    }

    fn current_branch_name(&self) -> Result<String, git2::Error> {
        let branch = self.current_branch()?;
        branch
            .name()?
            .ok_or(git2::Error::new(
                ErrorCode::GenericError,
                ErrorClass::Object,
                "Branch name not found",
            ))
            .map(|n| n.to_string())
    }

    fn is_working_tree_clean(&self) -> Result<bool, git2::Error> {
        // Check if the working tree is clean
        let mut status_opts = StatusOptions::new();
        status_opts
            .include_untracked(true) // Count untracked files
            .include_ignored(false) // Don't count ignored files
            .include_unmodified(false) // Don't include unmodified files
            .exclude_submodules(false) // Include submodules
            .recurse_untracked_dirs(true); // Look in untracked directories
        let statuses = self.statuses(Some(&mut status_opts))?;
        Ok(statuses.is_empty())
    }

    fn checkout_branch(&self, branch_name: &str) -> Result<(), git2::Error> {
        self.set_head(format!("refs/heads/{}", branch_name).as_str())?;
        self.checkout_head(Some(CheckoutBuilder::new().force()))?;
        Ok(())
    }

    fn set_target_to_upstream_ref(
        &self,
        branch_name: &str,
        remote_name: &str,
    ) -> Result<(), git2::Error> {
        let mut branch = self.find_branch(branch_name, BranchType::Local)?;
        let remote_ref = format!("refs/remotes/{}/{}", remote_name, branch_name);

        let upstream_ref = self.find_reference(&remote_ref)?;
        let upstream_ref_target = upstream_ref.target().ok_or(git2::Error::new(
            ErrorCode::GenericError,
            ErrorClass::Reference,
            "Upstream ref target not found",
        ))?;

        branch
            .get_mut()
            .set_target(upstream_ref_target, "Set ref to upstream ref")?;
        self.checkout_branch(branch_name)
    }

    fn rebase_branch_onto(
        &self,
        branch_name: &str,
        onto_name: &str,
    ) -> Result<(), GitCommandError> {
        // Check out the branch to rebase.
        self.checkout_branch(branch_name)?;

        // Cheat and shell out to git to rebase the branch. This is mainly because git2 doesn't
        // create rebase-todo files, etc., and it's easier to just shell out to git.
        execute_git_command(&["rebase", onto_name], false)
    }

    fn abort_rebase(&self) -> Result<(), GitCommandError> {
        execute_git_command(&["rebase", "--abort"], false)
    }

    fn push_branch(
        &self,
        branch_name: &str,
        remote_name: &str,
        force: bool,
    ) -> Result<(), GitCommandError> {
        let mut args = vec!["push", remote_name, branch_name];
        if force {
            args.push("--force");
        }

        execute_git_command(args.as_slice(), false)
    }

    fn pull_branch(&self, branch_name: &str, remote_name: &str) -> Result<(), GitCommandError> {
        self.checkout_branch(branch_name)?;
        execute_git_command(&["pull", remote_name, branch_name], false)
    }

    fn ahead_and_behind_upstream(&self, branch_name: &str) -> Result<(usize, usize), git2::Error> {
        // Get the local branch
        let local_branch = self.find_branch(branch_name, BranchType::Local)?;

        // Get the remote tracking branch
        let upstream = local_branch.upstream()?;

        // Get the commit that local branch points to
        let local_commit = local_branch.get().peel_to_commit()?;

        // Get the commit that upstream points to
        let upstream_commit = upstream.get().peel_to_commit()?;

        // Count ahead/behind commits
        self.graph_ahead_behind(local_commit.id(), upstream_commit.id())
    }
}

/// Error type for git command execution.
#[derive(Error, Debug)]
pub enum GitCommandError {
    /// An error occurred while executing a git command.
    #[error("git error:\n{}", .0)]
    Command(String),
    /// An IO error occurred.
    #[error("IO error: {}", .0)]
    IO(#[from] std::io::Error),
    /// A git2 error occurred.
    #[error("libgit2 error: {}", .0)]
    Git2(#[from] git2::Error),
}

/// Executes a `git` command with the given arguments in a blocking child task.
///
/// ## Takes
/// - `args` - The arguments to pass to the `git` command.
/// - `interactive` - Whether the command should be interactive.
fn execute_git_command(args: &[&str], interactive: bool) -> Result<(), GitCommandError> {
    let mut cmd = Command::new("git");
    if interactive {
        let status = cmd.args(args).status()?;

        if !status.success() {
            return Err(GitCommandError::Command(format!(
                "-> Command: `git {}`",
                args.join(" ")
            )));
        }
    } else {
        let output = cmd.args(args).output()?;

        if !output.status.success() {
            let git_error = String::from_utf8_lossy(&output.stderr)
                .trim_end_matches('\n')
                .replace("\n", &format!("\n{} ", QUOTE_CHAR))
                .replace("error: ", "");

            let error_message = format!("{} Git error:\n{} {}", QUOTE_CHAR, QUOTE_CHAR, git_error);
            return Err(GitCommandError::Command(
                Red.paint(error_message).to_string(),
            ));
        }
    }

    Ok(())
}
