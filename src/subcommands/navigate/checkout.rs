//! `checkout` subcommand.

use crate::{git::RepositoryExt, store::StoreWithRepository};
use anyhow::Result;
use clap::Args;

/// CLI arguments for the `checkout` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct CheckoutArgs;

impl CheckoutArgs {
    /// Run the `checkout` subcommand.
    pub fn run(self, store: StoreWithRepository<'_>) -> Result<()> {
        let StoreWithRepository { stacks, repository } = store;

        let mut buf = String::new();
        stacks.write_log_short(&mut buf, repository.current_branch()?.as_str())?;

        let branch = inquire::Select::new(
            "Select a branch to checkout",
            buf.lines().filter(|l| !l.is_empty()).collect(),
        )
        .with_formatter(&|_| "test".to_string())
        .prompt()?;

        dbg!(branch);

        Ok(())
    }
}
