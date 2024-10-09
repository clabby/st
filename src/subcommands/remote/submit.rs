//! `submit` subcommand.

use crate::ctx::StContext;
use anyhow::{anyhow, Result};
use clap::Args;
use git2::{Cred, PushOptions, RemoteCallbacks};
use octocrab::Octocrab;
use std::{env, fmt::Display};

/// CLI arguments for the `submit` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct SubmitCmd;

impl SubmitCmd {
    /// Run the `submit` subcommand.
    pub async fn run(self, ctx: StContext<'_>) -> Result<()> {
        let token = env::var("GITHUB_TOKEN")
            .map_err(|_| anyhow!("GITHUB_TOKEN environment variable must be set"))?
            .to_string();

        let gh_client = Octocrab::builder().personal_token(token.clone()).build()?;
        let (org, repo) = ctx.org_and_repository()?;

        let mut remote = ctx.repository.find_remote("origin")?;
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_, _, _| Cred::userpass_plaintext("clabby", &token));
        let mut push_opts = PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        let stack = ctx.resolve_active_stack()?;
        for node in stack {
            let branch_name = &node.borrow().local.branch_name;
            let parent = &node
                .borrow()
                .parent
                .clone()
                .map(|p| p.upgrade())
                .flatten()
                .ok_or(anyhow!("Parent not found"))?;
            let parent_name = &parent.borrow().local.branch_name;

            // Push the branch to the remote.
            remote.push(
                &[&format!("refs/heads/{}", branch_name)],
                Some(&mut push_opts),
            )?;

            let title = inquire::Text::new("Title of pull request:").prompt()?;
            let description = inquire::Editor::new("Pull request description").prompt()?;
            let submit_kind = inquire::Select::new(
                "Pull request kind",
                vec![SubmitKind::Draft, SubmitKind::Ready],
            )
            .prompt()?;

            // Submit PR.
            let pr_info = gh_client
                .pulls(org.clone(), repo.clone())
                .create(title, branch_name, parent_name)
                .body(description)
                .draft(matches!(submit_kind, SubmitKind::Draft))
                .send()
                .await?;

            let mut node_mut = node.borrow_mut();
            node_mut.remote = node_mut.remote.map(|mut r| {
                r.pr_number = pr_info.number;
                // TODO: Comment ID
                r
            });
        }

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
