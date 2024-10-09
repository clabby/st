//! Constants for the `st` application.

use nu_ansi_term::Color;

/// Name of the store file, within `.git`.
pub(crate) const ST_STORE_FILE_NAME: &str = ".st_store.toml";

/// Array of colors used for displaying stacks in the terminal.
pub(crate) const COLORS: [Color; 6] = [
    Color::Blue,
    Color::Cyan,
    Color::Green,
    Color::Purple,
    Color::Yellow,
    Color::Red,
];

pub(crate) const QUOTE_CHAR: char = '▌';
pub(crate) const FILLED_CIRCLE: char = '●';
pub(crate) const EMPTY_CIRCLE: char = '○';
pub(crate) const BOTTOM_LEFT_BOX: char = '└';
pub(crate) const LEFT_FORK_BOX: char = '├';
pub(crate) const VERTICAL_BOX: char = '│';
pub(crate) const HORIZONTAL_BOX: char = '─';
