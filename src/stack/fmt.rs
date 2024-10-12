//! Formatting for the [StackedBranch] type.

use super::TrackedBranch;
use crate::constants::{
    BOTTOM_LEFT_BOX, COLORS, EMPTY_CIRCLE, FILLED_CIRCLE, HORIZONTAL_BOX, LEFT_FORK_BOX,
    VERTICAL_BOX,
};
use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    fmt::{Display, Write},
};

impl TrackedBranch {
    /// Recursively writes a pretty-printed representation of the [StackedBranch] tree to the passed
    /// [Write]r.
    pub(crate) fn write_tree_recursive<W: Write>(
        &self,
        w: &mut W,
        branches: &HashMap<String, TrackedBranch>,
        checked_out: &str,
        needs_restack: &[String],
        depth: usize,
        is_last: bool,
        prefix: &str,
        connection: &str,
    ) -> Result<()> {
        let branch_char = (self.local.branch_name == checked_out)
            .then_some(FILLED_CIRCLE)
            .unwrap_or(EMPTY_CIRCLE);

        let rendered = COLORS[depth % COLORS.len()].paint(format!(
            "{}{} {}",
            connection, branch_char, self.local.branch_name,
        ));
        let needs_restack_notif = needs_restack
            .contains(&self.local.branch_name)
            .then(|| " (needs restack)")
            .unwrap_or_default();
        write!(
            w,
            "{}{}\n",
            prefix,
            format!("{}{}", rendered, needs_restack_notif)
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

            let child = branches.get(child).ok_or(anyhow!("Child does not exist"))?;
            child.write_tree_recursive(
                w,
                branches,
                checked_out,
                needs_restack,
                depth + 1,
                is_last_child,
                &prefix,
                &connection,
            )?;
        }

        Ok(())
    }

    /// Recursively a vector with the branches of the stack node and its children.
    pub(crate) fn fill_branches(
        &self,
        branches: &HashMap<String, TrackedBranch>,
        branch_names: &mut Vec<String>,
    ) -> Result<()> {
        branch_names.push(self.local.branch_name.clone());
        self.children.iter().try_for_each(|child| {
            let child = branches.get(child).ok_or(anyhow!("Child does not exist"))?;
            child.fill_branches(branches, branch_names)
        })
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
