//! The CLI for `st`.

use crate::{store::StoreWithRepository, subcommands::Subcommands};
use anyhow::{anyhow, Result};
use clap::{
    builder::styling::{AnsiColor, Color, Style},
    ArgAction, Parser,
};
use git2::{BranchType, Repository};
use inquire::Select;
use nu_ansi_term::Color::Blue;
use tracing::Level;

const ABOUT: &str = "st is a CLI application for working with stacked PRs on GitHub.";

/// The CLI application for `st`.
#[derive(Parser, Debug, Clone, Eq, PartialEq)]
#[command(about = ABOUT, version, styles = cli_styles())]
pub struct Cli {
    /// Verbosity level (0-4)
    #[arg(short, action = ArgAction::Count)]
    pub v: u8,
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: Option<Subcommands>,
}

impl Cli {
    /// Run the CLI application with the given arguments.
    pub async fn run(self) -> Result<()> {
        // Load the active repository.
        let repo = crate::git::active_repository()
            .ok_or_else(|| anyhow!("`st` only functions within a git repository."))?;
        let store = Self::load_store(&repo)?;

        match self.subcommand {
            Some(subcommand) => subcommand.run(store).await?,
            None => todo!("Print help menu."),
        }

        Ok(())
    }

    /// Loads the [StoreWithRepository] for the given [Repository].
    ///
    /// ## Takes
    /// - `repo` - The repository to load the store for.
    ///
    /// ## Returns
    /// - `Result<StoreWithRepository>` - The store for the repository.
    pub(crate) fn load_store<'a>(repo: &'a Repository) -> Result<StoreWithRepository> {
        // Attempt to load the repository store, or create a new one if it doesn't exist.
        let store = StoreWithRepository::try_load(&repo)?;
        match store {
            Some(store) => Ok(store),
            None => {
                let setup_message = format!(
                    "Repo not configured with `{}`. Select the trunk branch for the repository.",
                    Blue.paint("st")
                );

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

                let new_state = StoreWithRepository::new(trunk_branch, &repo);
                new_state.write()?;
                Ok(new_state)
            }
        }
    }

    /// Initializes the tracing subscriber
    ///
    /// ## Returns
    /// - `Result<()>` - Ok if successful, Err otherwise.
    pub(crate) fn init_tracing_subscriber(self) -> Result<Self> {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(match self.v {
                0 => Level::ERROR,
                1 => Level::WARN,
                2 => Level::INFO,
                3 => Level::DEBUG,
                _ => Level::TRACE,
            })
            .finish();

        tracing::subscriber::set_global_default(subscriber).map_err(|e| anyhow!(e))?;

        Ok(self)
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
