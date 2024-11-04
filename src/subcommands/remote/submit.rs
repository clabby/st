//! `submit` subcommand.

use crate::{
    ctx::StContext,
    errors::{StError, StResult},
    git::RepositoryExt,
    tree::RemoteMetadata,
};
use clap::Args;
use git2::BranchType;
use nu_ansi_term::Color;
use octocrab::{issues::IssueHandler, models::CommentId, pulls::PullRequestHandler, Octocrab};
use std::fmt::Display;

/// CLI arguments for the `submit` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct SubmitCmd {
    /// Force the submission of the stack, analogous to `git push --force`.
    #[clap(long, short)]
    force: bool,
}

impl SubmitCmd {
    /// Run the `submit` subcommand.
    pub async fn run(self, mut ctx: StContext<'_>) -> StResult<()> {
        // Establish the GitHub API client.
        let gh_client = Octocrab::builder()
            .personal_token(ctx.cfg.github_token.clone())
            .build()?;
        let (owner, repo) = ctx.owner_and_repository()?;
        let mut pulls = gh_client.pulls(&owner, &repo);
        let mut issues = gh_client.issues(&owner, &repo);

        // Resolve the active stack.
        let stack = ctx.discover_stack()?;

        // Perform pre-flight checks.
        println!("🔍 Checking for closed pull requests...");
        self.pre_flight(&mut ctx, &stack, &mut pulls).await?;

        // Submit the stack.
        println!(
            "\n🐙 Submitting changes to remote `{}`...",
            Color::Blue.paint("origin")
        );
        self.submit_stack(&mut ctx, &mut pulls, &mut issues, &owner, &repo)
            .await?;

        // Update the stack navigation comments on the PRs.
        println!("\n📝 Updating stack navigation comments...");
        self.update_pr_comments(&mut ctx, gh_client.issues(owner, repo), &stack)
            .await?;

        println!("\n🧙💫 All pull requests up to date.");
        Ok(())
    }

    /// Performs pre-flight checks before submitting the stack.
    async fn pre_flight(
        &self,
        ctx: &mut StContext<'_>,
        stack: &[String],
        pulls: &mut PullRequestHandler<'_>,
    ) -> StResult<()> {
        // Return early if the stack is not restacked or the current working tree is dirty.
        ctx.check_cleanliness(stack)?;

        // Check if any PRs have been closed, and offer to delete them before starting the submission process.
        let num_closed = ctx
            .delete_closed_branches(
                stack.iter().skip(1).cloned().collect::<Vec<_>>().as_slice(),
                pulls,
            )
            .await?;

        if num_closed > 0 {
            println!(
                "Deleted {} closed pull request{}. Run `{}` to re-stack the branches.",
                Color::Red.paint(num_closed.to_string()),
                (num_closed != 1).then_some("s").unwrap_or_default(),
                Color::Blue.paint("st restack")
            );
        }

        Ok(())
    }

    /// Submits the stack of branches to GitHub.
    async fn submit_stack(
        &self,
        ctx: &mut StContext<'_>,
        pulls: &mut PullRequestHandler<'_>,
        issues: &mut IssueHandler<'_>,
        owner: &str,
        repo: &str,
    ) -> StResult<()> {
        let stack = ctx.discover_stack()?;

        // Iterate over the stack and submit PRs.
        for (i, branch) in stack.iter().enumerate().skip(1) {
            let parent = &stack[i - 1];

            let tracked_branch = ctx
                .tree
                .get_mut(branch)
                .ok_or_else(|| StError::BranchNotTracked(branch.to_string()))?;

            if let Some(remote_meta) = tracked_branch.remote.as_ref() {
                // If the PR has already been submitted.

                // Grab remote metadata for the pull request.
                let remote_pr = pulls.get(remote_meta.pr_number).await?;

                // Check if the PR base needs to be updated
                if &remote_pr.base.ref_field != parent {
                    // Update the PR base.
                    pulls
                        .update(remote_meta.pr_number)
                        .base(parent)
                        .send()
                        .await?;
                    println!(
                        "-> Updated base branch for pull request for branch `{}` to `{}`.",
                        Color::Green.paint(branch),
                        Color::Yellow.paint(parent)
                    );
                }

                // Check if the local branch is ahead of the remote.
                let remote_synced = remote_pr.head.sha
                    == ctx
                        .repository
                        .find_branch(branch, BranchType::Local)?
                        .get()
                        .target()
                        .ok_or(StError::BranchUnavailable)?
                        .to_string();
                if remote_synced {
                    println!(
                        "Branch `{}` is up-to-date with the remote. Skipping push.",
                        Color::Green.paint(branch)
                    );
                    continue;
                }

                // Push the branch to the remote.
                ctx.repository.push_branch(branch, "origin", self.force)?;

                // Print success message.
                println!("Updated branch `{}` on remote.", Color::Green.paint(branch));
            } else {
                // If the PR has not been submitted yet.

                // Push the branch to the remote.
                ctx.repository.push_branch(branch, "origin", self.force)?;

                // Prompt the user for PR metadata.
                let metadata = Self::prompt_pr_metadata(branch, parent, issues).await?;

                // Submit PR.
                let pr_info = pulls
                    .create(metadata.title, branch, parent)
                    .body(metadata.body)
                    .draft(metadata.is_draft)
                    .send()
                    .await?;

                // Update labels and assignees, if declared.
                if metadata.labels.is_some() || metadata.assignees.is_some() {
                    let mut metadata_update = issues.update(pr_info.number);
                    if let Some(ref labels) = metadata.labels {
                        metadata_update = metadata_update.labels(labels);
                    }
                    if let Some(ref assignees) = metadata.assignees {
                        metadata_update = metadata_update.assignees(assignees);
                    }
                    metadata_update.send().await?;
                }

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

        Ok(())
    }

    /// Updates the comments on a PR with the current stack information.
    async fn update_pr_comments(
        &self,
        ctx: &mut StContext<'_>,
        issue_handler: IssueHandler<'_>,
        stack: &[String],
    ) -> StResult<()> {
        for branch in stack.iter().skip(1) {
            let tracked_branch = ctx
                .tree
                .get_mut(branch)
                .ok_or_else(|| StError::BranchNotTracked(branch.to_string()))?;

            // Skip branches that are not submitted as PRs.
            let Some(remote_meta) = tracked_branch.remote else {
                continue;
            };

            // If the PR has been submitted, update the comment.
            // If the PR is new, create a new comment.
            let rendered_comment = Self::render_pr_comment(ctx, branch, stack)?;
            match remote_meta.comment_id {
                Some(id) => {
                    // Update the existing comment.
                    issue_handler
                        .update_comment(CommentId(id), rendered_comment)
                        .await?;
                }
                None => {
                    // Create a new comment.
                    let comment_info = issue_handler
                        .create_comment(remote_meta.pr_number, rendered_comment)
                        .await?;

                    // Get a new mutable reference to the branch and update the comment ID.
                    ctx.tree
                        .get_mut(branch)
                        .expect("Must exist")
                        .remote
                        .as_mut()
                        .expect("Must exist")
                        .comment_id = Some(comment_info.id.0);
                }
            }
        }
        Ok(())
    }

    /// Prompts the user for metadata about the PR during the initial submission process.
    async fn prompt_pr_metadata(
        branch_name: &str,
        parent_name: &str,
        issues: &mut IssueHandler<'_>,
    ) -> StResult<PRCreationMetadata> {
        let title = inquire::Text::new(
            format!(
                "Title of pull request (`{}` -> `{}`):",
                Color::Green.paint(branch_name),
                Color::Yellow.paint(parent_name)
            )
            .as_str(),
        )
        .prompt()?;
        let body = inquire::Editor::new("Pull request description")
            .with_file_extension(".md")
            .prompt()?;
        let is_draft = inquire::Confirm::new("Is this PR a draft? (default: yes)")
            .with_default(true)
            .prompt()?;

        let set_labels =
            inquire::Confirm::new("Would you like to set labels for the pull request?")
                .with_default(false)
                .prompt()?;
        let labels = if set_labels {
            let labels = issues.list_labels_for_repo().send().await?.take_items();
            let display_labels = labels
                .iter()
                .map(|label| {
                    let color_hex = u32::from_str_radix(&label.color, 16).map_err(|_| {
                        StError::DecodingError("Failed to decode label color".to_string())
                    })?;
                    let (r, g, b) = (
                        (color_hex >> 16 & 0xff) as u8,
                        (color_hex >> 8 & 0xff) as u8,
                        (color_hex & 0xff) as u8,
                    );
                    let color = Color::Rgb(r, g, b);
                    Ok(SelectLabel {
                        name: label.name.clone(),
                        formatted: color.paint(label.name.as_str()).to_string(),
                    })
                })
                .collect::<StResult<Vec<_>>>()?;

            let selected_labels =
                inquire::MultiSelect::new("Select labels for the pull request:", display_labels)
                    .prompt()?;

            Some(
                selected_labels
                    .into_iter()
                    .map(|label| label.name)
                    .collect(),
            )
        } else {
            None
        };

        let set_assignee = inquire::Confirm::new("Would you like to assign the pull request?")
            .with_default(false)
            .prompt()?;
        let assignees = set_assignee
            .then(|| {
                let answer = inquire::Text::new("Assignees (comma-separated):")
                    .prompt()?
                    .replace(' ', "")
                    .split(',')
                    .map(ToString::to_string)
                    .collect();
                Ok::<_, StError>(answer)
            })
            .transpose()?;

        Ok(PRCreationMetadata {
            title,
            body,
            is_draft,
            labels,
            assignees,
        })
    }

    /// Renders the PR comment body for the current stack.
    fn render_pr_comment(
        ctx: &StContext<'_>,
        current_branch: &str,
        stack: &[String],
    ) -> StResult<String> {
        let mut comment = String::new();
        comment.push_str("## 📚 $\\text{Stack Overview}$\n\n");
        comment.push_str("Pulls submitted in this stack:\n");

        // Display all branches in the stack.
        for branch in stack.iter().skip(1).rev() {
            let tracked_branch = ctx
                .tree
                .get(branch)
                .ok_or_else(|| StError::BranchNotTracked(branch.to_string()))?;
            if let Some(remote) = tracked_branch.remote {
                comment.push_str(&format!(
                    "* #{}{}\n",
                    remote.pr_number,
                    (branch == current_branch)
                        .then_some(" 👈")
                        .unwrap_or_default()
                ));
            }
        }
        comment.push_str(format!("* `{}`\n", ctx.tree.trunk_name).as_str());

        comment.push_str(
            "\n_This comment was automatically generated by [`st`](https://github.com/clabby/st)._",
        );
        Ok(comment)
    }
}

/// Metadata about pull request creation.
#[derive(Debug)]
struct PRCreationMetadata {
    /// Title of the pull request.
    title: String,
    /// Body of the pull request.
    body: String,
    /// Whether or not the pull request is a draft.
    is_draft: bool,
    /// Labels to apply to the pull request.
    labels: Option<Vec<String>>,
    /// Assignees for the pull request.
    assignees: Option<Vec<String>>,
}

/// A colored label for display in the terminal.
#[derive(Debug)]
struct SelectLabel {
    /// The raw name of the label.
    name: String,
    /// The formatted name of the label.
    formatted: String,
}

impl Display for SelectLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.formatted)
    }
}
