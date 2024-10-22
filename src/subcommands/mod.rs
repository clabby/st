//! The subcommands for the `st` application.

use crate::{ctx::StContext, errors::StResult};
use clap::Subcommand;

mod local;
use local::{CheckoutCmd, CreateCmd, DeleteCmd, LogCmd, RestackCmd, TrackCmd, UntrackCmd};

mod remote;
use remote::{SubmitCmd, SyncCmd};

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
    /// Untrack the passed branch.
    #[clap(visible_alias = "ut")]
    Untrack(UntrackCmd),
    /// Submit the current PR stack to GitHub.
    #[clap(visible_aliases = ["s", "ss"])]
    Submit(SubmitCmd),
    /// Sync the remote branches with the local branches.
    #[clap(visible_aliases = ["rs", "sy"])]
    Sync(SyncCmd),
}

impl Subcommands {
    /// Run the subcommand with the given store.
    pub async fn run(self, ctx: StContext<'_>) -> StResult<()> {
        match self {
            // Remote
            Self::Sync(args) => args.run(ctx).await,
            Self::Submit(args) => args.run(ctx).await,
            // Local
            Self::Create(args) => args.run(ctx),
            Self::Delete(args) => args.run(ctx),
            Self::Log(args) => args.run(ctx),
            Self::Checkout(args) => args.run(ctx),
            Self::Restack(args) => args.run(ctx),
            Self::Track(args) => args.run(ctx),
            Self::Untrack(args) => args.run(ctx),
        }
    }
}
