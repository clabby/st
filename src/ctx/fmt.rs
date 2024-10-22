//! Contains the formatting logic for the [StContext] struct.

use super::StContext;
use crate::{
    constants::{
        BOTTOM_LEFT_BOX, COLORS, EMPTY_CIRCLE, FILLED_CIRCLE, HORIZONTAL_BOX, LEFT_FORK_BOX,
        VERTICAL_BOX,
    },
    errors::{StError, StResult},
    git::RepositoryExt,
};
use nu_ansi_term::Color;
use std::fmt::{Display, Write};

impl StContext<'_> {
    /// Gathers an in-order list of [DisplayBranch]es, containing the log-line and branch name.
    ///
    /// This function is particularly useful when creating prompts with [inquire::Select].
    pub fn display_branches(&self) -> StResult<Vec<DisplayBranch>> {
        // Collect the branches in the tree.
        let branches = self.tree.branches()?;

        // Render the branches.
        let mut buf = String::new();
        self.write_tree(&mut buf)?;

        // Break up the buffer into lines, after trimming whitespace.
        let log_lines = buf.trim().lines().collect::<Vec<_>>();

        let display_branches = branches
            .into_iter()
            .zip(log_lines)
            .map(|(branch, log_line)| DisplayBranch {
                display_value: log_line.to_string(),
                branch_name: branch.to_string(),
            })
            .collect();
        Ok(display_branches)
    }

    /// Prints the tree of branches contained within the [StContext].
    pub fn print_tree(&self) -> StResult<()> {
        let mut buf = String::new();
        self.write_tree(&mut buf)?;
        print!("{}", buf);
        Ok(())
    }

    /// Writes the tree of branches contained within the [StContext] to the given [Write]r.
    pub fn write_tree<W: Write>(&self, w: &mut W) -> StResult<()> {
        let trunk_name = self.tree.trunk_name.as_str();
        self.write_tree_recursive(w, trunk_name, 0, "", "", true)
    }

    /// Writes the tree of branches to the given writer recursively.
    fn write_tree_recursive<W: Write>(
        &self,
        w: &mut W,
        branch: &str,
        depth: usize,
        prefix: &str,
        connection: &str,
        is_parent_last_child: bool,
    ) -> StResult<()> {
        // Grab the checked out branch.
        let checked_out = self.repository.current_branch_name()?;
        let current = self
            .tree
            .get(branch)
            .ok_or_else(|| StError::BranchNotTracked(branch.to_string()))?;

        // Form the log-line for the current branch.
        let checked_out_icon = if branch == checked_out {
            FILLED_CIRCLE
        } else {
            EMPTY_CIRCLE
        };
        let rendered_branch = COLORS[depth % COLORS.len()]
            .paint(format!("{}{} {}", connection, checked_out_icon, branch));
        let branch_metadata = {
            let needs_restack = if self.needs_restack(branch)? {
                " (needs restack)"
            } else {
                ""
            };
            let pull_request = current
                .remote
                .map(|r| {
                    let (owner, repo) = self.owner_and_repository()?;
                    Ok::<_, StError>(Color::Purple.italic().paint(format!(
                        "https://github.com/{}/{}/pull/{}",
                        owner, repo, r.pr_number
                    )))
                })
                .transpose()?;
            format!(
                "{}{}",
                needs_restack,
                pull_request.map_or(String::new(), |s| format!(" ({})", s))
            )
        };

        // Write the current branch to the writer.
        writeln!(w, "{}{}{}", prefix, rendered_branch, branch_metadata)?;

        // Write the children of the branch recursively.
        let mut children = current.children.iter().peekable();
        while let Some(child) = children.next() {
            // Form the connection between the previous log-line and the current log-line.
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

            // Form the prefix for the current log-line
            let prefix = if depth > 0 {
                let color = COLORS[depth % COLORS.len()];
                is_parent_last_child
                    .then(|| format!("{}  ", prefix))
                    .unwrap_or(format!(
                        "{}{} ",
                        prefix,
                        color.paint(VERTICAL_BOX.to_string())
                    ))
            } else {
                prefix.to_string()
            };

            // Write the child and any of its children to the writer.
            self.write_tree_recursive(
                w,
                child,
                depth + 1,
                prefix.as_str(),
                connection.as_str(),
                is_last_child,
            )?;
        }

        Ok(())
    }
}

/// A pair of a log-line and a branch name, which implements [Display].
#[derive(Debug)]
pub struct DisplayBranch {
    /// The log-line to display.
    pub(crate) display_value: String,
    /// The branch name corresponding to the log-line.
    pub(crate) branch_name: String,
}

impl Display for DisplayBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_value)
    }
}
