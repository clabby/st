//! `checkout` subcommand.

use std::fmt::Display;

use crate::{git::RepositoryExt, store::StoreWithRepository};
use anyhow::Result;
use clap::Args;

/// CLI arguments for the `checkout` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct CheckoutArgs;

impl CheckoutArgs {
    /// Run the `checkout` subcommand.
    pub fn run(self, store: StoreWithRepository<'_>) -> Result<()> {
        let StoreWithRepository { stacks, repository } = store;

        // Write the log of the stacks.
        let mut buf = String::new();
        stacks.write_log_short(&mut buf, Some(repository.current_branch()?.as_str()))?;

        // Collect the branch names, in the order they appear in the log.
        let branches = stacks.branches();

        // Zip the log with the branch names.
        let items = buf
            .lines()
            .filter(|l| !l.is_empty())
            .zip(branches.iter())
            .map(|(line, branch_name)| DisplayBranch {
                line: line.to_string(),
                branch_name: branch_name.to_string(),
            })
            .collect::<Vec<_>>();

        let branch = inquire::Select::new("Select a branch to checkout", items)
            .with_formatter(&|l| branches[l.index].to_string())
            .prompt()?;

        repository.checkout_branch(branch.branch_name.as_str(), None)
    }
}

#[derive(Debug)]
struct DisplayBranch {
    line: String,
    branch_name: String,
}

impl Display for DisplayBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.line)
    }
}
