//! The data store for `st` configuration and stack state.

use crate::{
    constants::ST_STORE_FILE_NAME,
    git::RepositoryExt,
    stack::{DisplayBranch, LocalMetadata, StackedBranch, StackedBranchInner},
};
use anyhow::{anyhow, Result};
use git2::{BranchType, Repository};
use nu_ansi_term::Color::Blue;
use std::{collections::VecDeque, fmt::Write, path::PathBuf};

/// The data store for `st` configuration, with its associated [Repository]
pub struct StoreWithRepository<'a> {
    /// The repository associated with the store.
    pub repository: &'a Repository,
    /// The store for the repository.
    pub stack: StackedBranch,
}

impl<'a> StoreWithRepository<'a> {
    /// Creates a new [StoreWithRepository] with the given [Repository] and trunk branch name.
    pub fn new(repository: &'a Repository, trunk: String) -> Self {
        let local_meta = LocalMetadata {
            branch_name: trunk,
            ..Default::default()
        };
        let branch = StackedBranchInner::new(local_meta, None);
        Self {
            repository,
            stack: StackedBranch::new(branch),
        }
    }

    /// Loads the root [StackNode] for the given [Repository], and assembles a [StoreWithRepository].
    pub fn try_load(repository: &'a Repository) -> Result<Option<Self>> {
        let store_path = store_path(&repository).ok_or(anyhow!("Store path not found"))?;

        // If the store doesn't exist, return None.
        if !store_path.exists() {
            return Ok(None);
        }

        let store: StackedBranch = toml::from_str(&std::fs::read_to_string(store_path)?)?;
        let mut store_with_repo = Self {
            repository,
            stack: store,
        };
        store_with_repo.prune()?;
        store_with_repo.write()?;

        Ok(Some(store_with_repo))
    }

    /// Updates the [StackNode] store with the current branches and their children. If any of the branches
    /// have been deleted, they are pruned from the store.
    pub fn prune(&mut self) -> Result<()> {
        let branches = self.branches();

        for branch in branches {
            if self
                .repository
                .find_branch(branch.as_str(), BranchType::Local)
                .is_err()
            {
                self.stack.delete_child(branch.as_str());
            }
        }

        Ok(())
    }

    /// Persists the [StackNode] to the given [Repository] on disk.
    pub fn write(&self) -> Result<()> {
        let store_path = store_path(&self.repository).ok_or(anyhow!("Store path not found."))?;
        let store = toml::to_string_pretty(&self.stack)?;
        std::fs::write(store_path, store)?;
        Ok(())
    }

    /// Returns the current stack node, if the current branch exists within a tracked stack.
    pub fn current_stack_node(&self) -> Option<StackedBranch> {
        let current_branch = self.repository.current_branch().ok()?;
        let current_branch_name = current_branch.name().ok()??;
        self.stack.find_child(current_branch_name)
    }

    /// Attempts to resolve the current stack in full. In the worst case, there is a fork upstack,
    /// in which case this function will terminate at the fork, as it can not determine which
    /// path to take.
    pub fn resolve_active_stack(&self) -> Result<Vec<StackedBranch>> {
        let mut stack = VecDeque::new();
        let mut current = self
            .current_stack_node()
            .ok_or(anyhow!("Not within a stack"))?;

        // Resolve downstack
        while let Some(parent) = {
            let parent_ref = current.borrow().parent.clone();
            parent_ref.and_then(|p| p.upgrade())
        } {
            stack.push_front(current.clone());
            current = StackedBranch::from_shared(parent);
        }

        // Resolve upstack
        while current.borrow().children.len() == 1 {
            let child = current
                .borrow()
                .children
                .first()
                .expect("Cannot be empty")
                .clone();
            stack.push_back(child.clone());
            current = child;
        }

        Ok(stack.into())
    }

    /// Restacks the active stack.
    pub fn restack_current(&self) -> Result<()> {
        let stack = self.resolve_active_stack()?;

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

            let parent_ref = self
                .repository
                .find_branch(parent_name, BranchType::Local)?;
            let parent_ref_str = parent_ref
                .get()
                .target()
                .ok_or(anyhow!("Parent ref target not found"))?
                .to_string();
            if current.local.parent_oid_cache == parent_ref_str {
                println!(
                    "Branch `{}` does not need to be restacked onto {}.",
                    Blue.paint(current_name),
                    Blue.paint(parent_name)
                );
                continue;
            }

            // Attempt to rebase the current branch onto the parent branch.
            self.repository
                .rebase_branch_onto(current_name, parent_name)?;

            // Update the parent oid cache.
            node.borrow_mut().local.parent_oid_cache = parent_ref_str;

            println!(
                "Restacked `{}` onto `{}` successfully.",
                Blue.paint(current_name),
                Blue.paint(parent_name)
            );
        }

        // Write the store to disk.
        self.write()
    }

    /// Returns a vector of branch names within the [StackedBranch].
    pub fn branches(&self) -> Vec<String> {
        let mut branches = Vec::default();
        self.stack.fill_branches(&mut branches);
        branches
    }

    /// Returns a vector of [DisplayBranch]es for the stack node and its children.
    ///
    /// ## Takes
    /// - `checked_out` - The name of the branch that is currently checked out.
    ///                   If [None], the current branch is not highlighted.
    ///
    /// ## Returns
    /// - `Ok(Vec<DisplayBranch>)` - The branches of the stack node and its children,
    ///                              in the order they are logged.
    /// - `Err(_)` - If an error occurs while gathering the [DisplayBranch]es.
    pub fn display_branches(&self, checked_out: Option<&str>) -> Result<Vec<DisplayBranch>> {
        // Collect the branch names.
        let branches = self.branches();

        // Write the log of the stacks.
        let mut buf = String::new();
        self.write_tree(&mut buf, checked_out)?;

        // Zip the pretty-printed tree with the branch names to assemble the DisplayBranches.
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

    /// Writes a pretty-printed representation of the [StackedBranch] tree to the passed [Write]r.
    ///
    /// ## Takes
    /// - `w` - The writer to write the log to.
    /// - `checked_out` - The name of the branch that is currently checked out.
    ///
    /// ## Returns
    /// - `Ok(_)` - Tree successfully written.
    /// - `Err(_)` - If an error occurs while writing the Tree.
    pub fn write_tree<W: Write>(&self, w: &mut W, checked_out: Option<&str>) -> Result<()> {
        // Find all nodes that need to be restacked.
        let needs_restack = self
            .resolve_active_stack()?
            .iter()
            .filter_map(|n| {
                let b = n.borrow();

                let parent_node = b.parent.clone().map(|p| p.upgrade()).flatten()?;
                let parent_branch = self
                    .repository
                    .find_branch(&parent_node.borrow().local.branch_name, BranchType::Local);
                let parent_ref = parent_branch.ok()?.get().target()?.to_string();

                (b.local.parent_oid_cache != parent_ref).then(|| b.local.branch_name.clone())
            })
            .collect::<Vec<_>>();

        self.stack.write_tree_recursive(
            w,
            checked_out.unwrap_or_default(),
            needs_restack.as_slice(),
            0,
            true,
            Default::default(),
            Default::default(),
        )
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
