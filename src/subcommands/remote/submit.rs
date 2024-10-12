//! `submit` subcommand.

use crate::{ctx::StContext, git::RepositoryExt};
use anyhow::{anyhow, Result};
use clap::Args;
use nu_ansi_term::Color::Blue;
use octocrab::Octocrab;
use std::{env, fmt::Display};

/// CLI arguments for the `submit` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct SubmitCmd;

impl SubmitCmd {
    /// Run the `submit` subcommand.
    pub async fn run(self, ctx: StContext<'_>) -> Result<()> {
        // let token = env::var("GITHUB_TOKEN")
        //     .map_err(|_| anyhow!("GITHUB_TOKEN environment variable must be set"))?
        //     .to_string();
        //
        // let gh_client = Octocrab::builder().personal_token(token.clone()).build()?;
        // let (org, repo) = ctx.org_and_repository()?;
        //
        // let stack = ctx.resolve_active_stack()?;
        // for node in stack {
        //     let node_borrow = node.borrow_mut();
        //
        //     let branch_name = &node_borrow.local.branch_name;
        //     let parent = &node_borrow
        //         .parent
        //         .clone()
        //         .map(|p| p.upgrade())
        //         .flatten()
        //         .ok_or(anyhow!("Parent not found"))?;
        //     let parent_name = &parent.borrow().local.branch_name;
        //
        //     // Push the branch to the remote.
        //     ctx.repository.push_branch(branch_name, "origin")?;
        //
        //     // Exit early if a PR has already been submitted.
        //     if let Some(remote) = node_borrow.remote {
        //         println!(
        //             "Pull request already submitted for branch `{}` (ID: {}) - Pushed latest changes.",
        //             Blue.paint(branch_name),
        //             Blue.paint(format!("{}", remote.pr_number))
        //         );
        //         return Ok(());
        //     }
        //
        //     let title = inquire::Text::new(
        //         format!(
        //             "Title of pull request ({} -> {}):",
        //             branch_name, parent_name
        //         )
        //         .as_str(),
        //     )
        //     .prompt()?;
        //     let description = inquire::Editor::new("Pull request description").prompt()?;
        //     let submit_kind = inquire::Select::new(
        //         "Pull request kind",
        //         vec![SubmitKind::Draft, SubmitKind::Ready],
        //     )
        //     .prompt()?;
        //
        //     let pulls = gh_client.pulls(org.clone(), repo.clone());
        //
        //     // Submit PR.
        //     let pr_info = pulls
        //         .create(title, branch_name.clone(), parent_name.clone())
        //         .body(description)
        //         .draft(matches!(submit_kind, SubmitKind::Draft))
        //         .send()
        //         .await?;
        //
        //     let pr_link = format!(
        //         "https://github.com/{}/{}/pull/{}",
        //         org, repo, pr_info.number
        //     );
        //     println!(
        //         "Successfully submitted pull request for {} -> {} @ `{}`",
        //         Blue.paint(branch_name),
        //         Blue.paint(parent_name),
        //         Blue.paint(pr_link)
        //     );
        //
        //     {
        //         let mut node_mut = node.borrow_mut();
        //         node_mut.remote = node_mut.remote.map(|mut r| {
        //             r.pr_number = pr_info.number;
        //             // TODO: Comment ID
        //             r
        //         });
        //     }
        // }
        //
        // Ok(())
        Ok(())
    }
}

/// The kind of pull request to submit.
enum SubmitKind {
    Draft,
    Ready,
}

impl Display for SubmitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmitKind::Draft => write!(f, "Draft"),
            SubmitKind::Ready => write!(f, "Ready"),
        }
    }
}
