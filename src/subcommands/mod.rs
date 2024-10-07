//! The subcommands for the `st` application.

use crate::ctx::StContext;
use clap::Subcommand;
use stack::{CheckoutCmd, CreateCmd, DeleteCmd, LogCmd, RestackCmd, TrackCmd};

mod remote;
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
    /// Submit the current PR stack to GitHub.
    #[clap(visible_aliases = ["s", "ss"])]
    Submit(remote::SubmitCmd),
}

impl Subcommands {
    /// Run the subcommand with the given store.
    pub async fn run(self, store: StContext<'_>) -> anyhow::Result<()> {
        match self {
            Self::Checkout(args) => args.run(store),
            Self::Restack(args) => args.run(store),
            Self::Create(args) => args.run(store),
            Self::Delete(args) => args.run(store),
            Self::Log(args) => args.run(store),
            Self::Track(args) => args.run(store),
            Self::Submit(args) => args.run(store).await,
        }
    }
}
