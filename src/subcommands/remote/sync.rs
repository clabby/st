//! `sync` subcommand.

use crate::{
    ctx::StContext,
    errors::{StError, StResult},
    git::RepositoryExt,
};
use clap::Args;
use nu_ansi_term::Color;
use octocrab::{pulls::PullRequestHandler, Octocrab};

/// CLI arguments for the `sync` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct SyncCmd;

impl SyncCmd {
    /// Run the `sync` subcommand.
    pub async fn run(self, mut ctx: StContext<'_>) -> StResult<()> {
        // Establish the GitHub API client.
        let gh_client = Octocrab::builder()
            .personal_token(ctx.cfg.github_token.clone())
            .build()?;
        let (owner, repo) = ctx.owner_and_repository()?;
        let mut pulls = gh_client.pulls(&owner, &repo);

        // Perform pre-flight checks.
        self.pre_flight(&mut ctx, &mut pulls).await?;

        // Resolve all branches in the stack tree after the deletions have been applied.
        let branches = ctx.tree.branches()?;
        let branches_without_trunk = branches.iter().skip(1).cloned().collect::<Vec<_>>();

        // Pull all of the latest changes from GitHub.
        println!(
            "\n🐙 Pulling latest changes from remote `{}`...",
            Color::Blue.paint("origin")
        );
        self.pull_changes(&mut ctx, branches.as_slice()).await?;

        // Attempt to restack the current stack with the new changes.
        println!("\n🧙 Restacking branches...");
        self.try_restack_branches(ctx, branches_without_trunk.as_slice())
            .await?;

        println!("\n🔄 Sync completed");
        Ok(())
    }

    /// Performs pre-flight checks before syncing the stack.
    ///
    /// Steps:
    /// 1. Check if the working tree is clean.
    /// 2. Check if any PRs have been closed, and offer to delete branches before pulling latest.
    async fn pre_flight(
        &self,
        ctx: &mut StContext<'_>,
        pulls: &mut PullRequestHandler<'_>,
    ) -> StResult<()> {
        // Resolve the active stack.
        let branches = ctx.tree.branches()?;
        let branches_without_trunk = branches.iter().skip(1).cloned().collect::<Vec<_>>();

        // Return early if the stack is not restacked or the current working tree is dirty.
        if !ctx.repository.is_working_tree_clean()? {
            return Err(StError::WorkingTreeDirty);
        }

        // Check if any PRs have been closed, and offer to delete them before pulling latest
        // changes from GitHub.
        ctx.delete_closed_branches(branches_without_trunk.as_slice(), pulls)
            .await?;

        Ok(())
    }

    /// Pulls the latest changes from GitHub for the provided branches.
    async fn pull_changes(&self, ctx: &mut StContext<'_>, branches: &[String]) -> StResult<()> {
        for branch in branches {
            // If the branch hasn't been pushed to a remote, skip it.
            let tracked_branch = ctx
                .tree
                .get(branch)
                .ok_or_else(|| StError::BranchNotTracked(branch.clone()))?;
            if tracked_branch.remote.is_none() && branch != &ctx.tree.trunk_name {
                continue;
            }

            if let Err(e) = ctx.repository.pull_branch(branch, "origin") {
                eprintln!("{}\n\n", e);

                let message = format!(
                    "Failed to pull branch `{}`. Choose how to proceed:",
                    Color::Green.paint(branch)
                );
                let option = inquire::Select::new(
                    message.as_str(),
                    vec!["Continue", "Overwrite local with remote version"],
                )
                .prompt()?;

                if option.contains("Overwrite") {
                    ctx.repository
                        .set_target_to_upstream_ref(branch, "origin")?;
                    println!(
                        "Successfully overwrote local branch `{}` with remote version.",
                        Color::Green.paint(branch)
                    );
                }
            }
        }
        Ok(())
    }

    /// Restacks the provided branches.
    async fn try_restack_branches(
        &self,
        mut ctx: StContext<'_>,
        branches: &[String],
    ) -> StResult<()> {
        for branch in branches {
            if ctx.needs_restack(branch)? {
                let tracked_branch = ctx
                    .tree
                    .get(branch)
                    .ok_or_else(|| StError::BranchNotTracked(branch.clone()))?;
                let parent_name = tracked_branch
                    .parent
                    .as_ref()
                    .expect("Parent must exist")
                    .clone();

                let mut num_conflicts = 0;
                if ctx.restack_branch(branch, &parent_name).is_err() {
                    ctx.repository.abort_rebase()?;
                    println!(
                        "Failed to restack branch `{}` onto `{}`.",
                        Color::Green.paint(branch),
                        Color::Yellow.paint(parent_name)
                    );
                    num_conflicts += 1;
                }

                if num_conflicts > 0 {
                    println!(
                        "Failed to restack {} branches. You can resolve conflicts by checking out the stack and running `{}`.",
                        Color::Red.paint(num_conflicts.to_string()),
                        Color::Blue.paint("st restack")
                    );
                }
            }
        }
        Ok(())
    }
}
