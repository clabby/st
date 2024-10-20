//! `submit` subcommand.

use crate::{ctx::StContext, git::RepositoryExt, tree::RemoteMetadata};
use anyhow::{anyhow, Result};
use clap::Args;
use nu_ansi_term::Color;
use octocrab::{issues::IssueHandler, pulls::PullRequestHandler, Octocrab};
use std::env;

/// CLI arguments for the `submit` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct SubmitCmd;

impl SubmitCmd {
    /// Run the `submit` subcommand.
    pub async fn run(self, mut ctx: StContext<'_>) -> Result<()> {
        // Establish the GitHub API client.
        let token = env::var("GITHUB_TOKEN")
            .map_err(|_| anyhow!("GITHUB_TOKEN environment variable must be set"))?
            .to_string();
        let gh_client = Octocrab::builder().personal_token(token.clone()).build()?;
        let (owner, repo) = ctx.owner_and_repository()?;

        // Resolve the active stack.
        let stack = ctx.discover_stack()?;

        for (i, branch) in stack.iter().enumerate().skip(1) {
            let parent = &stack[i - 1];

            let tracked_branch = ctx
                .tree
                .get_mut(branch)
                .ok_or_else(|| anyhow!("Branch `{}` is not tracked with `st`.", branch))?;

            if let Some(_) = tracked_branch.remote.as_ref() {
                // If the PR has already been submitted.
            } else {
                // If the PR has not been submitted yet.

                // Push the branch to the remote.
                ctx.repository.push_branch(branch, "origin")?;

                // Prompt the user for PR metadata.
                let metadata = Self::prompt_pr_metadata(branch, parent)?;

                // Submit PR.
                let pulls = gh_client.pulls(&owner, &repo);
                let pr_info = pulls
                    .create(metadata.title, branch, parent)
                    .body(metadata.body)
                    .draft(metadata.is_draft)
                    .send()
                    .await?;

                // Update the tracked branch with the remote information.
                tracked_branch.remote = Some(RemoteMetadata::new(pr_info.number));

                // Print success message.
                let pr_link = format!(
                    "https://github.com/{}/{}/pull/{}",
                    owner, repo, pr_info.number
                );
                println!(
                    "Submitted new pull request for branch `{}` @ `{}`",
                    Color::Green.paint(branch),
                    Color::Blue.paint(pr_link)
                );
            }
        }

        // Update the comments on the PRs.
        Self::update_pr_comments(&mut ctx, gh_client.issues(owner, repo), &stack).await
    }

    /// Updates the comments on a PR with the current stack information.
    async fn update_pr_comments(
        ctx: &mut StContext<'_>,
        issue_handler: IssueHandler<'_>,
        stack: &[String],
    ) -> Result<()> {
        for branch in stack.iter().skip(1) {
            // Check if the pull request has a comment from `st`.
            let tracked_branch = ctx
                .tree
                .get_mut(branch)
                .ok_or_else(|| anyhow!("Branch `{}` is not tracked with `st`.", branch))?;

            let Some(remote_meta) = tracked_branch.remote.as_mut() else {
                continue;
            };

            match remote_meta.comment_id {
                Some(_) => {
                    // Update the existing comment.
                }
                None => {
                    // Create a new comment.
                    let comment_info = issue_handler
                        .create_comment(remote_meta.pr_number, "Test!")
                        .await?;
                    remote_meta.comment_id = Some(comment_info.id.0)
                }
            }
        }
        Ok(())
    }

    /// Prompts the user for metadata about the PR during the initial submission process.
    fn prompt_pr_metadata(branch_name: &str, parent_name: &str) -> Result<PRCreationMetadata> {
        let title = inquire::Text::new(
            format!(
                "Title of pull request (`{}` -> `{}`):",
                Color::Green.paint(branch_name),
                Color::Yellow.paint(parent_name)
            )
            .as_str(),
        )
        .prompt()?;
        let body = inquire::Editor::new("Pull request description").prompt()?;
        let is_draft = inquire::Confirm::new("Is this PR a draft?").prompt()?;

        Ok(PRCreationMetadata {
            title,
            body,
            is_draft,
        })
    }

    // /// Renders the PR comment body for the current stack.
    // fn render_pr_comment(current_branch: &str, stack: &[String]) -> String {
    //     let mut comment = String::new();
    //     comment.push_str("### 📚 Stack Status\n\n");
    //     for branch in stack.iter().rev() {
    //         comment.push_str(&format!("* "))
    //     }
    //     comment
    // }
}

/// Metadata about pull request creation.
struct PRCreationMetadata {
    /// Title of the pull request.
    title: String,
    /// Body of the pull request.
    body: String,
    /// Whether or not the pull request is a draft.
    is_draft: bool,
}
