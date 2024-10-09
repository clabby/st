//! Structured, [Serialize] + [Deserialize] representation of a stack of branches.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::{Rc, Weak},
};

mod fmt;
pub(crate) use fmt::DisplayBranch;

/// A wrapper type for a [StackedBranchInner] that allows for shared ownership and interior mutability.
#[derive(Debug, Clone)]
pub struct STree(Rc<RefCell<STreeInner>>);

impl STree {
    /// Creates a new [StackedBranch] from the given owned [StackedBranchInner].
    pub fn new(branch: STreeInner) -> Self {
        Self(Rc::new(RefCell::new(branch)))
    }

    /// Creates a new [StackedBranch] from the given shared reference to a [StackedBranchInner].
    pub fn from_shared(branch: Rc<RefCell<STreeInner>>) -> Self {
        Self(branch)
    }

    /// Returns a reference to the [StackedBranchInner] wrapped by this type.
    pub fn borrow(&self) -> Ref<STreeInner> {
        self.as_ref().borrow()
    }

    /// Returns a mutable reference to the [StackedBranchInner] wrapped by this type.
    pub fn borrow_mut(&self) -> RefMut<STreeInner> {
        self.as_ref().borrow_mut()
    }

    /// Adds a child branch to the passed parent, and establishes a weak reference to the parent
    /// within the child.
    ///
    /// ## Takes
    /// - `child` - The child branch to add to the parent.
    pub fn insert_child(&self, child: STree) {
        child.borrow_mut().parent = Some(Rc::downgrade(self.as_ref()));
        self.borrow_mut().children.push(child);
    }

    /// Attempts to delete a child branch from within the stack. If the child does not exist,
    /// returns [None].
    ///
    /// ## Takes
    /// - `child` - The name of the child branch to delete.
    ///
    /// ## returns
    /// - `Some(branch)` - The deleted child branch.
    /// - `None` - The child branch by the name of `child` was not found.
    pub fn delete_child(&self, child: &str) -> Option<STree> {
        // Check if the branch exists within the current branch's children.
        // If it does, remove it and return it.
        let pos = self
            .borrow()
            .children
            .iter()
            .position(|c| c.borrow().local.branch_name == child);
        if let Some(index) = pos {
            return Some(self.borrow_mut().children.remove(index));
        }

        // Continue recursion.
        self.borrow()
            .children
            .iter()
            .find_map(|c| c.delete_child(child))
    }

    /// Attempts to find a child branch within the stack. If the child does not exist, returns [None].
    ///
    /// ## Takes
    /// - `child` - The name of the child branch to find.
    ///
    /// ## returns
    /// - `Some(branch)` - The found child branch.
    /// - `None` - The child branch by the name of `child` was not found.
    pub fn find_branch(&self, branch: &str) -> Option<STree> {
        let borrow = self.borrow();
        if borrow.local.branch_name == branch {
            return Some(self.clone());
        }

        self.borrow()
            .children
            .iter()
            .find_map(|c| c.find_branch(branch))
    }
}

impl AsRef<Rc<RefCell<STreeInner>>> for STree {
    fn as_ref(&self) -> &Rc<RefCell<STreeInner>> {
        &self.0
    }
}

impl Serialize for STree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        /// Recursively converts a [StackedBranch] into a [SerdeStackedBranch].
        fn to_serializable(branch: &STree) -> SerdeStackedBranch {
            SerdeStackedBranch {
                local: branch.borrow().local.clone(),
                remote: branch.borrow().remote,
                children: branch
                    .borrow()
                    .children
                    .iter()
                    .map(|child| to_serializable(&child))
                    .collect(),
            }
        }

        to_serializable(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for STree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serde_repr = SerdeStackedBranch::deserialize(deserializer)?;

        /// Recursively converts a [SerdeStackedBranch] into a [StackedBranch], establishing
        /// two-way parent-child relationships along the way.
        fn from_serializable(serde_branch: SerdeStackedBranch) -> STree {
            let branch = STree::new(STreeInner::new(serde_branch.local, serde_branch.remote));

            serde_branch.children.into_iter().for_each(|child_serde| {
                let child = from_serializable(child_serde);
                branch.insert_child(child);
            });

            branch
        }

        Ok(from_serializable(serde_repr))
    }
}

/// In-memory, recursive representation of a stack of branches, with [LocalMetadata], [RemoteMetadata],
/// strong references to children, and a weak reference to the parent branch (if the branch is not trunk).
#[derive(Default, Debug, Clone)]
pub struct STreeInner {
    /// The local metadata for the branch.
    pub local: LocalMetadata,
    /// The remote metadata for the branch.
    ///
    /// [None] if a pull request has not yet been submitted, or the previous
    /// pull request was closed.
    pub remote: Option<RemoteMetadata>,
    /// The child branches of the current branch.
    pub children: Vec<STree>,
    /// The parent branch of the current branch, if it exists.
    pub parent: Option<Weak<RefCell<STreeInner>>>,
}

impl STreeInner {
    /// Create a new [StackedBranch] with the given local and remote metadata.
    ///
    /// ## Takes
    /// - `local` - The [LocalMetadata] for the branch.
    /// - `remote` - The [RemoteMetadata] for the branch.
    pub fn new(local: LocalMetadata, remote: Option<RemoteMetadata>) -> Self {
        Self {
            local,
            remote,
            ..Default::default()
        }
    }
}

/// Local metadata for a branch that is tracked by `st`.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LocalMetadata {
    /// The name of the branch.
    pub branch_name: String,
    /// The cached [git2::Oid] of the parent's target ref, in [String] form.
    ///
    /// Valid iff the parent branch's target ref is a commit with an equivalent [git2::Oid].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_oid_cache: Option<String>,
}

/// Remote metadata for a branch that is tracked by `st`.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteMetadata {
    /// The number of the pull request associated with the branch.
    pub pr_number: u64,
    /// The comment ID of the stack status comment on the pull request.
    pub comment_id: u64,
}

/// An intermediate [Serialize] + [Deserialize] representation of a [StackedBranch], for
/// persistence of `st` application state to disk.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SerdeStackedBranch {
    /// The local metadata for the branch.
    #[serde(flatten)]
    local: LocalMetadata,
    /// The remote metadata for the branch.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    remote: Option<RemoteMetadata>,
    /// The child branches of the current branch.
    children: Vec<SerdeStackedBranch>,
}
