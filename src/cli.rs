//! The CLI for `st`.

use crate::{
    config::{prompt_for_configuration, StConfig},
    ctx::StContext,
    errors::{StError, StResult},
    subcommands::Subcommands,
};
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
    pub async fn run(self) -> StResult<()> {
        // Load the active repository.
        let repo = crate::git::active_repository().ok_or(StError::NotAGitRepository)?;
        let config = Self::load_cfg_or_initialize()?;
        let context = Self::load_ctx_or_initialize(config, &repo)?;
        self.subcommand.run(context).await
    }

    /// Loads the [StConfig]. If the config does not exist or is the default config, prompts
    /// the user to set up the `st` for the first time.
    ///
    /// ## Returns
    /// - `Result<StConfig>` - The global `st` config.
    pub(crate) fn load_cfg_or_initialize() -> StResult<StConfig> {
        // Load the global configuration for `st`, or initialize it if it doesn't exist.
        match StConfig::try_load()? {
            Some(config) if config.validate().is_ok() => Ok(config),
            _ => prompt_for_configuration(None),
        }
    }

    /// Loads the [StContext] for the given [Repository]. If the context does not exist,
    /// prompts the user to set up the repository with `st`.
    ///
    /// ## Takes
    /// - `repo` - The repository to load the context for.
    ///
    /// ## Returns
    /// - `Result<StContext>` - The context for the repository.
    pub(crate) fn load_ctx_or_initialize(
        config: StConfig,
        repo: &Repository,
    ) -> StResult<StContext> {
        // Attempt to load the repository store, or create a new one if it doesn't exist.
        if let Some(ctx) = StContext::try_load(config.clone(), repo)? {
            return Ok(ctx);
        }

        let setup_message = format!(
            "Repo not configured with `{}`. Select the trunk branch for the repository.",
            Blue.paint("st")
        );

        // Ask the user to specify the trunk branch of the repository.
        // The trunk branch must be a local branch.
        let branches = repo
            .branches(Some(BranchType::Local))?
            .map(|b| {
                let (b, _) = b?;
                b.name()?
                    .map(ToOwned::to_owned)
                    .ok_or(StError::BranchUnavailable)
            })
            .collect::<StResult<Vec<_>>>()?;
        let trunk_branch = Select::new(&setup_message, branches).prompt()?;

        // Print the welcome message.
        println!(
            "\nSuccessfully set up repository with `{}`. Happy stacking ✨📚\n",
            Blue.paint("st")
        );

        Ok(StContext::fresh(config, repo, trunk_branch))
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
