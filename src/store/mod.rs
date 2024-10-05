//! The data store for `st` configuration and stack state.

use crate::constants::{
    BOTTOM_LEFT_BOX, COLORS, EMPTY_CIRCLE, FILLED_CIRCLE, HORIZONTAL_BOX, LEFT_FORK_BOX,
    ST_STORE_FILE_NAME, VERTICAL_BOX,
};
use anyhow::{anyhow, Result};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, path::PathBuf};

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
                children: Vec::default(),
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

        let ser_store = std::fs::read_to_string(store_path)?;
        let store: StackNode = toml::from_str(&ser_store)?;

        Ok(Some(Self {
            repository,
            stacks: store,
        }))
    }

    /// Writes the [StackNode] to the given [Repository].
    pub fn write(&self) -> Result<()> {
        let store_path = store_path(&self.repository).ok_or(anyhow!("Store path not found."))?;
        let store = toml::to_string_pretty(&self.stacks)?;
        std::fs::write(store_path, store)?;
        Ok(())
    }
}

/// A [StackNode] represents a branch within a stack, optionally with children.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StackNode {
    /// The name of the branch for the stack.
    pub branch: String,
    /// The branches within the stack.
    pub children: Vec<StackNode>,
}

impl StackNode {
    /// Prints the stack node and its children for the termnal, in short-form.
    pub fn write_log_short<T: Write>(&self, w: &mut T, checked_out: &str) -> Result<()> {
        self.write_log_short_recursive(
            w,
            checked_out,
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
        let branch_char = if self.branch == checked_out {
            FILLED_CIRCLE
        } else {
            EMPTY_CIRCLE
        };

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
