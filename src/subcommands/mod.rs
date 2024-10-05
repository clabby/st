//! The subcommands for the `st` application.

use crate::store::StoreWithRepository;
use clap::Subcommand;
use navigate::CheckoutArgs;
use stack::{CreateArgs, DeleteArgs, LogArgs, TrackArgs};

mod navigate;
mod stack;

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum Subcommands {
    /// Checkout a branch that is tracked with `st`.
    #[clap(alias = "co")]
    Checkout(CheckoutArgs),
    /// Create a new branch within the current stack.
    #[clap(alias = "c")]
    Create(CreateArgs),
    /// Delete a branch that is tracked with `st`.
    #[clap(aliases = ["d", "del"])]
    Delete(DeleteArgs),
    /// Print a tree of all local stacks.
    #[clap(aliases = ["l", "ls"])]
    Log(LogArgs),
    /// Track the current branch on top of a stack node.
    #[clap(alias = "tr")]
    Track(TrackArgs),
}

impl Subcommands {
    /// Run the subcommand with the given store.
    pub async fn run(self, store: StoreWithRepository<'_>) -> anyhow::Result<()> {
        match self {
            Self::Checkout(args) => args.run(store),
            Self::Create(args) => args.run(store),
            Self::Delete(args) => args.run(store),
            Self::Log(args) => args.run(store),
            Self::Track(args) => args.run(store),
        }
    }
}
