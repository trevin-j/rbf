//! Contains BF-related errors that can happen.

use core::fmt;
use std::error;

/// Represents the kind of `BracketMismatch`.
#[derive(Debug, Clone)]
pub enum BFErrorKind {
    /// When a closing bracket has no opening.
    MissingOpen,
    /// When an opening bracket has no closing.
    MissingClose,
    /// When an invalid value was entered to BF input.
    InvalidInput,
    /// When trying to access cell where cell pointer is out of the cells bounds.
    CellBoundsError,
    /// When the instruction pointer is out of the bounds of the instructions vec.
    InstructionBoundsError,
}

/// Represents a BF error where not all brackets have matches.
///
/// Match `BracketMismatch::kind` to determine if it's an open bracket or close bracket error.
#[derive(Debug, Clone)]
pub struct BFError {
    /// Kind of mismatch.
    pub kind: BFErrorKind,
}

impl fmt::Display for BFError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.kind {
                BFErrorKind::MissingClose =>
                    "The program has an open bracket with no close bracket.",
                BFErrorKind::MissingOpen => "The program has a close bracket with no open bracket.",
                BFErrorKind::InvalidInput => "An invalid value was passed to BF input.",
                BFErrorKind::CellBoundsError => "Tried to access cell out of bounds",
                BFErrorKind::InstructionBoundsError =>
                    "Tried to process instruction out of bounds.",
            }
        )
    }
}

impl error::Error for BFError {}
