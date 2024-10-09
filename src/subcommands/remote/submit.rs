//! `submit` subcommand.

use crate::ctx::StContext;
use anyhow::Result;
use clap::Args;
use octocrab::Octocrab;
use std::{env, fmt::Display};

/// CLI arguments for the `submit` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct SubmitCmd;

impl SubmitCmd {
    /// Run the `submit` subcommand.
    pub async fn run(self, ctx: StContext<'_>) -> Result<()> {
        let gh_client = Octocrab::builder()
            .personal_token(env::var("GITHUB_TOKEN")?.to_string())
            .build()?;
        let (org, repo) = ctx.org_and_repository()?;

        let title = inquire::Text::new("Title of pull request:").prompt()?;
        let description = inquire::Editor::new("Pull request description").prompt()?;
        let submit_kind = inquire::Select::new(
            "Pull request kind",
            vec![SubmitKind::Draft, SubmitKind::Ready],
        )
        .prompt()?;

        // Submit PR.
        // TODO
        let _ = gh_client
            .pulls(org, repo)
            .create(title, "test", "main")
            .body(description)
            .draft(matches!(submit_kind, SubmitKind::Draft))
            .send()
            .await?;

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
