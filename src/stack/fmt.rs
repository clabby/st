//! Formatting for the [StackedBranch] type.

use super::StackedBranch;
use crate::constants::{
    BOTTOM_LEFT_BOX, COLORS, EMPTY_CIRCLE, FILLED_CIRCLE, HORIZONTAL_BOX, LEFT_FORK_BOX,
    VERTICAL_BOX,
};
use anyhow::Result;
use std::fmt::{Display, Write};

impl StackedBranch {
    /// Returns a vector of branch names within the [StackedBranch].
    pub fn branches(&self) -> Vec<String> {
        let mut branches = Vec::default();
        self.fill_branches(&mut branches);
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
        self.write_tree_recursive(
            w,
            checked_out.unwrap_or_default(),
            0,
            true,
            Default::default(),
            Default::default(),
        )
    }

    /// Recursively writes a pretty-printed representation of the [StackedBranch] tree to the passed
    /// [Write]r.
    fn write_tree_recursive<W: Write>(
        &self,
        w: &mut W,
        checked_out: &str,
        depth: usize,
        is_last: bool,
        prefix: &str,
        connection: &str,
    ) -> Result<()> {
        let self_borrow = self.borrow();
        let branch_char = (self_borrow.local.branch_name == checked_out)
            .then_some(FILLED_CIRCLE)
            .unwrap_or(EMPTY_CIRCLE);

        write!(
            w,
            "{}{}\n",
            prefix,
            COLORS[depth % COLORS.len()].paint(format!(
                "{}{} {}",
                connection, branch_char, self_borrow.local.branch_name
            ))
        )?;

        let mut children = self_borrow.children.iter().peekable();
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

            child.write_tree_recursive(
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
    fn fill_branches(&self, branch_names: &mut Vec<String>) {
        let self_borrow = self.borrow();

        branch_names.push(self_borrow.local.branch_name.clone());
        self_borrow
            .children
            .iter()
            .for_each(|child| child.fill_branches(branch_names))
    }
}

/// A pair of a log-line and a branch name.
#[derive(Debug)]
pub(crate) struct DisplayBranch {
    /// The log-line to display.
    pub(crate) line: String,
    /// The branch name corresponding to the log-line.
    pub(crate) branch_name: String,
}

impl Display for DisplayBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.line)
    }
}
