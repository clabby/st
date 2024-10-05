//! `restack` subcommand.

use crate::{git::RepositoryExt, store::StoreWithRepository};
use anyhow::{anyhow, Result};
use clap::Args;
use git2::BranchType;
use nu_ansi_term::Color::Blue;

/// CLI arguments for the `restack` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct RestackArgs;

impl RestackArgs {
    /// Run the `restack` subcommand.
    pub fn run(self, mut store: StoreWithRepository<'_>) -> Result<()> {
        // let stack = store.resolve_active_stack()?;

        Ok(())
    }
}
