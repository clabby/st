//! The data store for `st` configuration and stack state.

use crate::{
    constants::{
        BOTTOM_LEFT_BOX, COLORS, EMPTY_CIRCLE, FILLED_CIRCLE, HORIZONTAL_BOX, LEFT_FORK_BOX,
        ST_STORE_FILE_NAME, VERTICAL_BOX,
    },
    git::RepositoryExt,
};
use anyhow::{anyhow, Result};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Write},
    path::PathBuf,
};

/// The data store for `st` configuration, with its associated [Repository]
pub struct StoreWithRepository<'a> {
    /// The repository associated with the store.
    pub repository: &'a Repository,
    /// The store for the repository.
    pub stacks: StackNode,
}

impl<'a> StoreWithRepository<'a> {
    /// Creates a new [StoreWithRepository] with the given [Repository] and trunk branch name.
    pub fn new(repository: &'a Repository, trunk: String) -> Self {
        Self {
            repository,
            stacks: StackNode {
                branch: trunk,
                ..Default::default()
            },
        }
    }

    /// Loads the root [StackNode] for the given [Repository], and assembles a [StoreWithRepository].
    pub fn try_load(repository: &'a Repository) -> Result<Option<Self>> {
        let store_path = store_path(&repository).ok_or(anyhow!("Store path not found"))?;

        // If the store doesn't exist, return None.
        if !store_path.exists() {
            return Ok(None);
        }

        let store: StackNode = toml::from_str(&std::fs::read_to_string(store_path)?)?;
        let mut store_with_repo = Self {
            repository,
            stacks: store,
        };
        store_with_repo.prune()?;
        store_with_repo.write()?;

        Ok(Some(store_with_repo))
    }

    /// Updates the [StackNode] store with the current branches and their children. If any of the branches
    /// have been deleted, they are pruned from the store.
    pub fn prune(&mut self) -> Result<()> {
        let mut branches = Vec::default();
        self.stacks.fill_branches(&mut branches);

        for branch in branches {
            if self
                .repository
                .find_branch(branch.as_str(), git2::BranchType::Local)
                .is_err()
            {
                self.stacks.delete_stack_node(branch.as_str());
            }
        }

        Ok(())
    }

    /// Persists the [StackNode] to the given [Repository] on disk.
    pub fn write(&self) -> Result<()> {
        let store_path = store_path(&self.repository).ok_or(anyhow!("Store path not found."))?;
        let store = toml::to_string_pretty(&self.stacks)?;
        std::fs::write(store_path, store)?;
        Ok(())
    }

    /// Returns the current stack node, if the current branch exists within a tracked stack.
    pub fn current_stack_node(&mut self) -> Option<&mut StackNode> {
        let current_branch = self.repository.current_branch().ok()?;
        let current_branch_name = current_branch.name().ok()??;
        self.stacks.find_stack_node(current_branch_name)
    }

    /// Returns a list of the branches within the current stack, in order of closest to the trunk.
    pub fn current_stack(&mut self) -> Vec<String> {
        let mut stack = Vec::default();

        let mut current_stack_node = self.current_stack_node();

        stack
    }

    /// Returns a vector of [DisplayBranch]es for the stack node and its children.
    ///
    /// ## Returns
    /// - `Ok(Vec<DisplayBranch>)` - The branches of the stack node and its children,
    ///                              in the order they are logged.
    /// - `Err(_)` - If an error occurs while gathering the [DisplayBranch]es.
    pub fn display_branches(&self) -> Result<Vec<DisplayBranch>> {
        let mut branches = Vec::default();
        self.stacks.fill_branches(&mut branches);

        let current_branch = self.repository.current_branch()?;
        let current_branch_name = current_branch
            .name()?
            .ok_or(anyhow!("Name of current branch not found"))?;

        // Write the log of the stacks.
        let mut buf = String::new();
        self.stacks
            .write_log_short(&mut buf, Some(current_branch_name))?;

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

        Ok(items)
    }
}

/// A [StackNode] represents a branch within a stack, optionally with children.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StackNode {
    /// The name of the branch for the stack.
    pub branch: String,
    /// The branches within the stack.
    pub children: Vec<StackNode>,
    /// The open PR number for the branch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<u64>,
    /// The cached parent reference [git2::Oid], in [String] form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_oid: Option<String>,
}

impl StackNode {
    /// Creates a new [StackNode] with the given branch name and no children.
    pub fn new(branch: String) -> Self {
        Self {
            branch,
            children: Vec::default(),
            pr: None,
            parent_oid: None,
        }
    }

    /// Attempts to find a branch within the stack node and its children.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to find.
    ///
    /// ## Returns
    /// - `Some(&mut StackNode)` - The stack node with the branch name.
    /// - `None` - If the branch name is not found within the stack node or its children.
    pub fn find_stack_node(&mut self, branch_name: &str) -> Option<&mut StackNode> {
        if self.branch == branch_name {
            return Some(self);
        }

        self.children
            .iter_mut()
            .find_map(|child| child.find_stack_node(branch_name))
    }

    /// Recursively searches for a [StackNode] with the branch name specified and prunes
    /// it and any of its children.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to delete.
    ///
    /// ## Returns
    /// - `Some(StackNode)` - The deleted stack node.
    /// - `None` - If the branch name is not found within the stack node or its children.
    pub fn delete_stack_node(&mut self, branch_name: &str) -> Option<StackNode> {
        if let Some(index) = self
            .children
            .iter()
            .position(|child| child.branch == branch_name)
        {
            return Some(self.children.remove(index));
        }

        self.children
            .iter_mut()
            .find_map(|child| child.delete_stack_node(branch_name))
    }

    /// Prints the stack node and its children for the termnal, in short-form.
    ///
    /// ## Takes
    /// - `w` - The writer to write the log to.
    /// - `checked_out` - The name of the branch that is currently checked out.
    ///
    /// ## Returns
    /// - `Ok(_)` - Log successfully written.
    /// - `Err(_)` - If an error occurs while writing the log.
    pub fn write_log_short<T: Write>(&self, w: &mut T, checked_out: Option<&str>) -> Result<()> {
        self.write_log_short_recursive(
            w,
            checked_out.unwrap_or_default(),
            0,
            true,
            Default::default(),
            Default::default(),
        )
    }

    /// Recursively logs the stack node and its children for the terminal, in short-form.
    fn write_log_short_recursive<T: Write>(
        &self,
        w: &mut T,
        checked_out: &str,
        depth: usize,
        is_last: bool,
        prefix: &str,
        connection: &str,
    ) -> Result<()> {
        let branch_char = (self.branch == checked_out)
            .then_some(FILLED_CIRCLE)
            .unwrap_or(EMPTY_CIRCLE);

        write!(
            w,
            "{}{}\n",
            prefix,
            COLORS[depth % COLORS.len()]
                .paint(format!("{}{} {}", connection, branch_char, self.branch))
        )?;

        let mut children = self.children.iter().peekable();
        while let Some(child) = children.next() {
            let is_last_child = children.peek().is_none();
            let connection = format!(
                "{}{}",
                if is_last_child {
                    BOTTOM_LEFT_BOX
                } else {
                    LEFT_FORK_BOX
                },
                HORIZONTAL_BOX
            );
            let prefix = if depth > 0 {
                let color = COLORS[depth % COLORS.len()];
                is_last.then(|| format!("{}  ", prefix)).unwrap_or(format!(
                    "{}{} ",
                    prefix,
                    color.paint(VERTICAL_BOX.to_string())
                ))
            } else {
                prefix.to_owned()
            };

            child.write_log_short_recursive(
                w,
                checked_out,
                depth + 1,
                is_last_child,
                &prefix,
                &connection,
            )?;
        }

        Ok(())
    }

    /// Recursively a vector with the branches of the stack node and its children.
    fn fill_branches(&self, branches: &mut Vec<String>) {
        branches.push(self.branch.clone());
        self.children
            .iter()
            .for_each(|child| child.fill_branches(branches))
    }
}

#[derive(Debug)]
pub(crate) struct DisplayBranch {
    pub(crate) line: String,
    pub(crate) branch_name: String,
}

impl Display for DisplayBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.line)
    }
}

/// Returns the path to the [StackNode] store for the given [Repository].
///
/// ## Takes
/// - `repository` - The repository to get the store path for.
///
/// ## Returns
/// - `Some(PathBuf)` - The path to the store.
/// - `None` - If the repository does not have a workdir.
pub fn store_path<'a>(repository: &Repository) -> Option<PathBuf> {
    repository
        .workdir()
        .map(|p| p.join(".git").join(ST_STORE_FILE_NAME))
}
