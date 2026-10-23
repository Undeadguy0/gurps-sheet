//! GURPS 4e rules engine.
//!
//! * [`value`] — explainable values (`base + contributions`).
//! * [`sheet`] — computes all attributes/secondary characteristics of a character.
//! * [`points`] — point costs and the total breakdown.
//! * [`dice`] — 3d6 success rolls with critical rules.

pub mod dice;
pub mod points;
pub mod sheet;
pub mod value;

pub use sheet::{compute_sheet, Sheet, StatRow};
pub use value::{Contribution, SourceKind, Value};
