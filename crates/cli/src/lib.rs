//! The `planner` command's adapters: the layout file reader and the text report.
//!
//! Kept in a library so tests can drive [`check`] without spawning a process.

pub mod file;
pub mod report;

use planner_core::{Layout, bom, footprint};

/// The result of checking a layout file's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The layout can be built; the report lists its parts and size.
    Valid(String),
    /// The layout breaks a structural rule; the report lists every problem.
    Refused(String),
}

/// Reads a layout file's text and checks it against the core rules.
pub fn check(text: &str) -> Result<Outcome, file::FileError> {
    let draft = file::parse(text)?;
    Ok(match Layout::new(draft) {
        Ok(layout) => Outcome::Valid(report::valid(&bom(&layout), &footprint(&layout))),
        Err(errors) => Outcome::Refused(report::refused(&errors)),
    })
}
