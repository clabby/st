//! The subcommands for the `st` application.

use crate::store::StoreWithRepository;
use clap::Subcommand;
use stack::CreateArgs;

mod log;
mod navigate;
mod remote;
mod stack;

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum Subcommands {
    /// Create a new branch within the current stack. If the stack does not exist, the branch will be created as a new stack on top of the trunk branch.
    Create(CreateArgs),
}

impl Subcommands {
    /// Run the subcommand with the given store.
    pub async fn run(self, store: StoreWithRepository<'_>) -> anyhow::Result<()> {
        match self {
            Self::Create(args) => args.run(store).await,
        }
    }
}
