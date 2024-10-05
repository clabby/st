//! `create` subcommand.

use crate::{git::RepositoryExt, store::StoreWithRepository};
use anyhow::{anyhow, Result};
use clap::Args;
use git2::{Config, Signature};
use nu_ansi_term::Color::Blue;

/// CLI arguments for the `create` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct CreateArgs;

impl CreateArgs {
    /// Run the `create` subcommand.
    pub async fn run(self, store: StoreWithRepository<'_>) -> Result<()> {
        let head = store.repository.head()?;
        let head_name = head.name().ok_or(anyhow!("Name of head not found"))?;
        let head_commit = head.peel_to_commit()?;

        // FLOW:
        // 1. If the current branch is tracked, create a new branch on top of the current stack.
        // 2. If the current branch is not tracked, error.
        // TODO: Do above.
        if !head.is_branch() {
            return Err(anyhow!("Cannot create a stack on a detached HEAD."));
        } else if head.is_remote() {
            return Err(anyhow!("Cannot create a stack on top of a remote branch."));
        }

        // Prompt the user for the name of their new branch.
        let branch_name = inquire::Text::new("Name of new branch:").prompt()?;

        // Create the new branch and check it out.
        store.repository.branch(&branch_name, &head_commit, false)?;
        store
            .repository
            .checkout_branch(branch_name.as_str(), None)?;

        // Inform user of success.
        println!(
            "Successfully created and tracked new branch `{}` on top of `{}`.",
            Blue.paint(branch_name),
            Blue.paint(head_name)
        );

        let commit_staged = inquire::Confirm::new("Commit staged? [y/n]").prompt()?;
        if commit_staged {
            // Grab the index.
            let mut index = store.repository.index()?;

            // Fetch the write tree.
            let tree_id = index.write_tree()?;
            let tree = store.repository.find_tree(tree_id)?;

            // Fetch the git config
            let config = Config::open_default()?;

            // Craft the signature for the commit.
            let signature = Signature::now(
                config.get_string("user.name")?.as_str(),
                config.get_string("user.email")?.as_str(),
            )?;

            // Commit the staged items.
            let oid = store.repository.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "test",
                &tree,
                &[&head_commit],
            )?;

            println!(
                "Successfully committed staged changes. Commit oid: `{}`",
                Blue.paint(oid.to_string())
            )
        }

        Ok(())
    }
}
