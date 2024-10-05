//! `checkout` subcommand.

use crate::{git::RepositoryExt, store::StoreWithRepository};
use anyhow::Result;
use clap::Args;

/// CLI arguments for the `checkout` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct CheckoutCmd;

impl CheckoutCmd {
    /// Run the `checkout` subcommand.
    pub fn run(self, store: StoreWithRepository<'_>) -> Result<()> {
        let branches = store.display_branches()?;

        let branch = inquire::Select::new("Select a branch to checkout", branches)
            .with_formatter(&|f| f.value.branch_name.clone())
            .prompt()?;

        store
            .repository
            .checkout_branch(branch.branch_name.as_str(), None)
    }
}
