//! `restack` subcommand.

use crate::{git::RepositoryExt, store::StoreWithRepository};
use anyhow::{anyhow, Result};
use clap::Args;
use nu_ansi_term::Color::Blue;

/// CLI arguments for the `restack` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct RestackCmd;

impl RestackCmd {
    /// Run the `restack` subcommand.
    pub fn run(self, store: StoreWithRepository<'_>) -> Result<()> {
        let stack = store.resolve_active_stack()?;

        for node in stack.iter() {
            let parent_node = node
                .borrow()
                .parent
                .clone()
                .map(|p| {
                    p.upgrade()
                        .ok_or(anyhow!("Weak reference to parent is dead."))
                })
                .transpose()?
                .ok_or(anyhow!("No parent found."))?;

            let current = node.borrow();
            let parent = parent_node.borrow();

            let current_name = current.local.branch_name.as_str();
            let parent_name = parent.local.branch_name.as_str();
 
            // Attempt to rebase the current branch onto the parent branch.
            store
                .repository
                .rebase_branch_onto(current_name, parent_name)?;

            println!(
                "Restacked `{}` onto `{}` successfully.",
                Blue.paint(current_name),
                Blue.paint(parent_name)
            );
        }

        Ok(())
    }
}
