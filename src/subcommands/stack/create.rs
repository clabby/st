//! `create` subcommand.

use crate::{
    git::RepositoryExt,
    stack::{LocalMetadata, StackedBranch, StackedBranchInner},
    store::StoreWithRepository,
};
use anyhow::{anyhow, Result};
use clap::Args;
use nu_ansi_term::Color::Blue;

/// CLI arguments for the `create` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct CreateCmd {
    /// Name of the new branch to create.
    #[clap(index = 1)]
    branch_name: Option<String>,
}

impl CreateCmd {
    /// Run the `create` subcommand.
    pub fn run(self, store: StoreWithRepository<'_>) -> Result<()> {
        let head = store.repository.head()?;
        let head_name = head.name().ok_or(anyhow!("Name of head not found"))?;
        let head_commit = head.peel_to_commit()?;

        // Prompt the user for the name of their new branch, or use the provided name.
        let branch_name = match self.branch_name {
            Some(name) => name,
            None => inquire::Text::new("Name of new branch:").prompt()?,
        };

        // Write the new branch to the store in-memory.
        let stack_node = store
            .current_stack_node()
            .ok_or(anyhow!("Not currently on a branch within a tracked stack."))?;
        let child_local_meta = LocalMetadata {
            branch_name: branch_name.clone(),
            parent_oid_cache: head_commit.id().to_string(),
        };
        stack_node.insert_child(StackedBranch::new(StackedBranchInner::new(
            child_local_meta,
            None,
        )));

        // Create the new branch and check it out.
        store.repository.branch(&branch_name, &head_commit, false)?;
        store
            .repository
            .checkout_branch(branch_name.as_str(), None)?;

        // Update the store on disk.
        store.write()?;

        // Inform user of success.
        println!(
            "Successfully created and tracked new branch `{}` on top of `{}`.",
            Blue.paint(branch_name),
            Blue.paint(head_name)
        );
        Ok(())
    }
}
