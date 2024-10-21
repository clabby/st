//! `delete` subcommand.

use crate::{actions::Action, ctx::StContext, git::RepositoryExt};
use anyhow::{anyhow, bail, Result};
use clap::Args;
use git2::BranchType;
use nu_ansi_term::Color;

/// CLI arguments for the `delete` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct DeleteCmd {
    /// Name of the new branch to delete.
    #[clap(index = 1)]
    branch_name: Option<String>,
}

impl DeleteCmd {
    /// Run the `delete` subcommand.
    pub async fn run(self, mut ctx: StContext<'_>) -> Result<()> {
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

        Action::DeleteBranch {
            branch_name: &branch_name,
            must_delete_from_tree: false,
        }
        .dispatch(&mut ctx)
        .await?;

        println!(
            "Successfully deleted branch `{}`.",
            Color::Blue.paint(&branch_name)
        );
        Ok(())
    }
}
