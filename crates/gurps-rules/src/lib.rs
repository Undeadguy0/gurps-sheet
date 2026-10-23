//! GURPS 4e rules engine.
//!
//! * [`value`] — explainable values (`base + contributions`).
//! * [`sheet`] — computes all attributes/secondary characteristics of a character.
//! * [`points`] — point costs and the total breakdown.
//! * [`dice`] — 3d6 success rolls with critical rules.
#![cfg_attr(test, allow(clippy::field_reassign_with_default))]

pub mod dice;
pub mod encumbrance;
pub mod points;
pub mod sheet;
pub mod skills;
pub mod validate;
pub mod value;

pub use sheet::{compute_sheet, Sheet, StatRow};
pub use value::{Contribution, SourceKind, Value};

/// English point count with correct number: "1 pt", "2 pts", "-1 pt".
pub fn pts(n: i32) -> String {
    if n.abs() == 1 {
        format!("{n} pt")
    } else {
        format!("{n} pts")
    }
}

#[cfg(test)]
mod pts_tests {
    #[test]
    fn english_points() {
        assert_eq!(super::pts(1), "1 pt");
        assert_eq!(super::pts(2), "2 pts");
        assert_eq!(super::pts(0), "0 pts");
    }
}
