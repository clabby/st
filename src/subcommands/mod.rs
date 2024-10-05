//! The subcommands for the `st` application.

use anyhow::Result;
use clap::Subcommand;
use git2::Repository;

mod log;
mod navigate;
mod remote;
mod stack;

pub type SubcommandDispatcher<T> = fn(Repository, T) -> Result<()>;

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum Subcommands {
    /// Initialize the repository for use with `st`.
    Init,
}
