//! Contains the formatting logic for the [StContext] struct.

use super::StContext;
use crate::{
    constants::{
        BOTTOM_LEFT_BOX, COLORS, EMPTY_CIRCLE, FILLED_CIRCLE, HORIZONTAL_BOX, LEFT_FORK_BOX,
        VERTICAL_BOX,
    },
    git::RepositoryExt,
};
use anyhow::{anyhow, Result};
use nu_ansi_term::Color;
use std::fmt::{Display, Write};

impl<'a> StContext<'a> {
    /// Gathers an in-order list of [DisplayBranch]es, containing the log-line and branch name.
    ///
    /// This function is particularly useful when creating prompts with [inquire::Select].
    pub fn display_branches(&self) -> Result<Vec<DisplayBranch>> {
        // Collect the branches in the tree.
        let branches = self.tree.branches()?;

        // Render the branches.
        let mut buf = String::new();
        self.write_tree(&mut buf)?;

        // Break up the buffer into lines, after trimming whitespace.
        let log_lines = buf.trim().lines().collect::<Vec<_>>();

        // Pair the log-lines with the branches.
        if branches.len() != log_lines.len() {
            return Err(anyhow!(
                "Mismatch between branches and log-lines: {} branches, {} log-lines",
                branches.len(),
                log_lines.len()
            ));
        }

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
    pub fn print_tree(&self) -> Result<()> {
        let mut buf = String::new();
        self.write_tree(&mut buf)?;
        print!("{}", buf);
        Ok(())
    }

    /// Writes the tree of branches contained within the [StContext] to the given [Write]r.
    pub fn write_tree<W: Write>(&self, w: &mut W) -> Result<()> {
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
    ) -> Result<()> {
        // Grab the checked out branch.
        let checked_out = self.repository.current_branch_name()?;
        let current = self
            .tree
            .get(branch)
            .ok_or(anyhow!("Branch {} not tracked with `st`.", branch))?;

        // Form the log-line for the current branch.
        let checked_out_icon = (branch == checked_out)
            .then_some(FILLED_CIRCLE)
            .unwrap_or(EMPTY_CIRCLE);
        let rendered_branch = COLORS[depth % COLORS.len()]
            .paint(format!("{}{} {}", connection, checked_out_icon, branch));
        let branch_metadata = {
            let needs_restack = self
                .needs_restack(branch)?
                .then_some(" (needs restack)")
                .unwrap_or("");
            let pull_request = current
                .remote
                .map(|r| {
                    let (owner, repo) = self.owner_and_repository()?;
                    Ok::<_, anyhow::Error>(Color::Cyan.italic().paint(format!(
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
        write!(w, "{}{}{}\n", prefix, rendered_branch, branch_metadata)?;

        // Write the children of the branch recursively.
        let mut children = current.children.iter().peekable();
        while let Some(child) = children.next() {
            // Form the connection between the previous log-line and the current log-line.
            let is_last_child = children.peek().is_none();
            let connection = format!(
                "{}{}",
                is_last_child
                    .then_some(BOTTOM_LEFT_BOX)
                    .unwrap_or(LEFT_FORK_BOX),
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

