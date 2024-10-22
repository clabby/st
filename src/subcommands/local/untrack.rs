//! `untrack` subcommand.

use crate::{ctx::StContext, errors::StResult};
use clap::Args;
use nu_ansi_term::Color;

/// CLI arguments for the `untrack` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct UntrackCmd {
    /// Name of the new branch untrack.
    #[clap(index = 1)]
    branch_name: Option<String>,
}

impl UntrackCmd {
    /// Run the `untrack` subcommand.
    pub fn run(self, mut ctx: StContext<'_>) -> StResult<()> {
        // Gather the display branches.
        let display_branches = ctx.display_branches()?;

        // Prompt the user for the name of the branch to delete, or use the provided name.
        let branch_name = match self.branch_name {
            Some(name) => name,
            None => {
                inquire::Select::new("Select a branch to delete", display_branches)
                    .with_formatter(&|f| f.value.branch_name.clone())
                    .prompt()?
                    .branch_name
            }
        };

        ctx.tree.delete(&branch_name)?;

        println!(
            "Successfully untracked branch `{}`.",
            Color::Blue.paint(&branch_name)
        );
        Ok(())
    }
}
