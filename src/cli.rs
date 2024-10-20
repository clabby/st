//! The CLI for `st`.

use crate::{ctx::StContext, subcommands::Subcommands};
use anyhow::{anyhow, Result};
use clap::{
    builder::styling::{AnsiColor, Color, Style},
    ArgAction, Parser,
};
use git2::{BranchType, Repository};
use inquire::Select;
use nu_ansi_term::Color::Blue;

const ABOUT: &str = "st is a CLI application for working with stacked PRs locally and on GitHub.";

/// The CLI application for `st`.
#[derive(Parser, Debug, Clone, Eq, PartialEq)]
#[command(about = ABOUT, version, styles = cli_styles(), arg_required_else_help(true))]
pub struct Cli {
    /// Verbosity level (0-4)
    #[arg(short, action = ArgAction::Count)]
    pub v: u8,
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: Subcommands,
}

impl Cli {
    /// Run the CLI application with the given arguments.
    pub async fn run(self) -> Result<()> {
        // Load the active repository.
        let repo = crate::git::active_repository()
            .ok_or_else(|| anyhow!("`st` only functions within a git repository."))?;
        let context = Self::load_ctx_or_initialize(&repo)?;

        self.subcommand.run(context).await
    }

    /// Loads the [StContext] for the given [Repository]. If the context does not exist,
    /// prompts the user to set up the repository with `st`.
    ///
    /// ## Takes
    /// - `repo` - The repository to load the context for.
    ///
    /// ## Returns
    /// - `Result<StContext>` - The context for the repository.
    pub(crate) fn load_ctx_or_initialize<'a>(repo: &'a Repository) -> Result<StContext> {
        // Attempt to load the repository store, or create a new one if it doesn't exist.
        let store = StContext::try_load(&repo)?;
        match store {
            Some(store) => Ok(store),
            None => {
                let setup_message = format!(
                    "Repo not configured with `{}`. Select the trunk branch for the repository.",
                    Blue.paint("st")
                );

                // Ask the user to specify the trunk branch of the repository.
                // The trunk branch must be a local branch.
                let branches = repo
                    .branches(Some(BranchType::Local))?
                    .into_iter()
                    .map(|b| {
                        let (b, _) = b?;
                        b.name()?
                            .map(ToOwned::to_owned)
                            .ok_or(anyhow!("Branch name invalid."))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let trunk_branch = Select::new(&setup_message, branches).prompt()?;

                // Print the welcome message.
                println!(
                    "\nSuccessfully set up repository with `{}`. Happy stacking ✨📚\n",
                    Blue.paint("st")
                );

                Ok(StContext::fresh(&repo, trunk_branch))
            }
        }
    }
}

/// Styles for the CLI application.
const fn cli_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))))
}
