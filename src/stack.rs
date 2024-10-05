//! Structured representation of a stack of branches, with local and remote metadata for each branch.

use serde::{Deserialize, Serialize};

/// Local metadata for a branch that is tracked by `st`.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LocalMetadata {
    /// The name of the branch.
    pub branch_name: String,
    /// The cached [git2::Oid] of the parent's target ref, in [String] form.
    ///
    /// Valid iff the parent branch's target ref is a commit with an equivalent [git2::Oid].
    pub parent_oid: String,
}

/// Remote metadata for a branch that is tracked by `st`.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteMetadata {
    /// The number of the pull request associated with the branch.
    ///
    /// [None] if no pull request has been submitted, or the previous pull request was closed.
    pub pr_number: u64,
    /// The comment ID of the stack status comment on the pull request.
    ///
    /// [None] if no pull request has been submitted, or the previous pull request was closed.
    pub comment_id: u64,
}
