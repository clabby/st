//! `restack` subcommand.

use crate::store::StoreWithRepository;
use anyhow::Result;
use clap::Args;

/// CLI arguments for the `restack` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct RestackCmd;

impl RestackCmd {
    /// Run the `restack` subcommand.
    pub fn run(self, store: StoreWithRepository<'_>) -> Result<()> {
        store.restack_current()
    }
}
