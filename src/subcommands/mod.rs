//! The subcommands for the `st` application.

use crate::store::StoreWithRepository;
use clap::Subcommand;
use stack::{CheckoutCmd, CreateCmd, DeleteCmd, LogCmd, RestackCmd, TrackCmd};

mod stack;

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum Subcommands {
    /// Checkout a branch that is tracked with `st`.
    #[clap(visible_alias = "co")]
    Checkout(CheckoutCmd),
    /// Restack the the current stack.
    #[clap(visible_aliases = ["r", "sr"])]
    Restack(RestackCmd),
    /// Create and track a new branch within the current stack.
    #[clap(visible_alias = "c")]
    Create(CreateCmd),
    /// Delete a branch that is tracked with `st`.
    #[clap(visible_aliases = ["d", "del"])]
    Delete(DeleteCmd),
    /// Print a tree of all tracked stacks.
    #[clap(visible_aliases = ["l", "ls"])]
    Log(LogCmd),
    /// Track the current branch on top of a tracked stack node.
    #[clap(visible_alias = "tr")]
    Track(TrackCmd),
}

impl Subcommands {
    /// Run the subcommand with the given store.
    pub async fn run(self, store: StoreWithRepository<'_>) -> anyhow::Result<()> {
        match self {
            Self::Checkout(args) => args.run(store),
            Self::Restack(args) => args.run(store),
            Self::Create(args) => args.run(store),
            Self::Delete(args) => args.run(store),
            Self::Log(args) => args.run(store),
            Self::Track(args) => args.run(store),
        }
    }
}
