//! Utilities for interacting with `git` repositories for the `st` application.

use git2::Repository;
use std::env;

/// Returns the repository for the current working directory, and [None] if
/// the current working directory is not within a git repository or an error
/// occurs.
pub fn active_repository() -> Option<Repository> {
    Repository::discover(env::current_dir().ok()?).ok()
}
