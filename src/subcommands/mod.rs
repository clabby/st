//! The subcommands for the `st` application.

use crate::store::StoreWithRepository;
use clap::Subcommand;
use log::LogArgs;
use navigate::CheckoutArgs;
use stack::CreateArgs;

mod log;
mod navigate;
mod stack;

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum Subcommands {
    /// Create a new branch within the current stack. If the stack does not exist, the branch will be created as a new stack on top of the trunk branch.
    #[clap(alias = "c")]
    Create(CreateArgs),
    /// Checkout a branch that is tracked with `st`.
    #[clap(alias = "co")]
    Checkout(CheckoutArgs),
    /// Print a tree of all local stacks.
    #[clap(aliases = ["l", "ls"])]
    Log(LogArgs),
}

impl Subcommands {
    /// Run the subcommand with the given store.
    pub async fn run(self, store: StoreWithRepository<'_>) -> anyhow::Result<()> {
        match self {
            Self::Create(args) => args.run(store),
            Self::Checkout(args) => args.run(store),
            Self::Log(args) => args.run(store),
        }
    }
}
