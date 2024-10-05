//! Constants for the `st` application.

use nu_ansi_term::Color;

pub(crate) const ST_STORE_FILE_NAME: &str = ".st_store.toml";

pub(crate) const COLORS: [Color; 6] = [
    Color::Blue,
    Color::Cyan,
    Color::Green,
    Color::Red,
    Color::Yellow,
    Color::Purple,
];

pub(crate) const FILLED_CIRCLE: char = '●';
pub(crate) const EMPTY_CIRCLE: char = '○';
pub(crate) const BOTTOM_LEFT_BOX: char = '└';
pub(crate) const LEFT_FORK_BOX: char = '├';
pub(crate) const VERTICAL_BOX: char = '│';
pub(crate) const HORIZONTAL_BOX: char = '─';
