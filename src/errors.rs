//! Errors for the `st` application.

use crate::{config::StConfigError, git::GitCommandError};
use nu_ansi_term::Color;
use thiserror::Error;

/// Errors for the `st` application.
#[derive(Error, Debug)]
pub enum StError {
    // ---- [ `st` application errors (local) ] ----
    /// The branch is not tracked with `st`.
    #[error(
        "Branch `{}` is not tracked with `{}`. Track it first with `{}`.",
        Color::Blue.paint(.0),
        Color::Blue.paint("st"),
        Color::Blue.paint("st track")
    )]
    BranchNotTracked(String),
    /// The branch is already tracked with `st`.
    #[error("Branch `{}` is already tracked with `{}`.", Color::Blue.paint(.0), Color::Blue.paint("st"))]
    BranchAlreadyTracked(String),
    /// Cannot delete the trunk branch.
    #[error("Cannot delete the trunk branch.")]
    CannotDeleteTrunkBranch,
    /// A branch needs to be restacked.
    #[error(
        "Branch `{}` needs to be restacked before continuing. Restack with `{}` before continuing.",
        Color::Green.paint(.0),
        Color::Blue.paint("st restack")
    )]
    NeedsRestack(String),
    /// The working tree is dirty.
    #[error("Working tree is dirty. Please commit or stash changes before continuing.")]
    WorkingTreeDirty,
    /// The parent's [git2::Oid] cache is missing.
    #[error("Parent's [git2::Oid] cache is missing.")]
    MissingParentOidCache,
    /// A generic decoding error occurred.
    #[error("Decoding error: {}", .0)]
    DecodingError(String),

    // ---- [ `st` application errors (remote) ] ----
    /// A remote pull request could not be found.
    #[error("Remote pull request not found.")]
    PullRequestNotFound,
    /// A pull request is already open.
    #[error("Pull request is already open for branch {} with parent `{}`.", Color::Blue.paint(.0), Color::Blue.paint(.1))]
    PullRequestAlreadyOpen(String, String),

    // ---- [ Git Errors ] ----
    /// `st` mused be run within a git repository.
    #[error("`{}` must be used within a git repository.", Color::Blue.paint("st"))]
    NotAGitRepository,
    /// The git repository root could not be found.
    #[error("Git repository root could not be found.")]
    GitRepositoryRootNotFound,
    /// Remote not found.
    #[error("Remote `{}` not found.", Color::Blue.paint(.0))]
    RemoteNotFound(String),
    /// The branch was not found in the local git tree.
    #[error("Branch was not found in local git tree.")]
    BranchUnavailable,

    // ---- [ Child Errors ] ----
    /// An [StConfigError] occurred.
    #[error(transparent)]
    StConfigError(#[from] StConfigError),
    /// A [git2::Error] occurred.
    #[error("🐙 libgit2 error: {}", .0)]
    Git2Error(#[from] git2::Error),
    /// A `git` command error occurred.
    #[error(transparent)]
    GitCommandError(#[from] GitCommandError),
    /// An [octocrab::Error] occurred.
    #[error("🐙 octocrab error: {:?}", .0)]
    OctocrabError(#[from] octocrab::Error),
    /// An [inquire::InquireError] occurred.
    #[error("🔍 inquire error: {}", .0)]
    InquireError(#[from] inquire::InquireError),
    /// An [std::io::Error] occurred.
    #[error("🦀 IO error: {}", .0)]
    IoError(#[from] std::io::Error),
    /// A [std::fmt::Write] error occurred.
    #[error("🖋️ write error: {}", .0)]
    WriteError(#[from] std::fmt::Error),
    /// A [toml::ser::Error] occurred.
    #[error("🍅 toml decoding error: {}", .0)]
    TomlSerializationError(#[from] toml::ser::Error),
    /// A [toml::de::Error] occurred.
    #[error("🍅 toml decoding error: {}", .0)]
    TomlDecodingError(#[from] toml::de::Error),
}

/// A short-hand [Result] type alias for the [StError].
pub type StResult<T> = Result<T, StError>;
