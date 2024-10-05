//! `log` subcommand.

use crate::{git::RepositoryExt, store::StoreWithRepository};
use anyhow::Result;
use clap::Args;

/// CLI arguments for the `log` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct LogArgs;

impl LogArgs {
    /// Run the `log` subcommand.
    pub fn run(self, store: StoreWithRepository<'_>) -> Result<()> {
        let StoreWithRepository { stacks, repository } = store;

        let mut buf = String::new();
        stacks.write_log_short(&mut buf, Some(repository.current_branch()?.as_str()))?;

        print!("{}", buf);
        Ok(())
    }
}
