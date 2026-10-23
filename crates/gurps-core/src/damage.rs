//! Basic damage (thrust/swing) by ST, as a dice + adds table.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiceSpec {
    pub dice: u32,
    pub adds: i32,
}

impl fmt::Display for DiceSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.adds {
            0 => write!(f, "{}d", self.dice),
            a if a > 0 => write!(f, "{}d+{}", self.dice, a),
            a => write!(f, "{}d{}", self.dice, a),
        }
    }
}

const fn d(dice: u32, adds: i32) -> DiceSpec {
    DiceSpec { dice, adds }
}

/// (thrust, swing) for ST 1..=40.
const TABLE: [(DiceSpec, DiceSpec); 40] = [
    (d(1, -6), d(1, -5)),
    (d(1, -6), d(1, -5)),
    (d(1, -5), d(1, -4)),
    (d(1, -5), d(1, -4)),
    (d(1, -4), d(1, -3)),
    (d(1, -4), d(1, -3)),
    (d(1, -3), d(1, -2)),
    (d(1, -3), d(1, -2)),
    (d(1, -2), d(1, -1)),
    (d(1, -2), d(1, 0)),
    (d(1, -1), d(1, 1)),
    (d(1, -1), d(1, 2)),
    (d(1, 0), d(2, -1)),
    (d(1, 0), d(2, 0)),
    (d(1, 1), d(2, 1)),
    (d(1, 1), d(2, 2)),
    (d(1, 2), d(3, -1)),
    (d(1, 2), d(3, 0)),
    (d(2, -1), d(3, 1)),
    (d(2, -1), d(3, 2)),
    (d(2, 0), d(4, -1)),
    (d(2, 0), d(4, 0)),
    (d(2, 1), d(4, 1)),
    (d(2, 1), d(4, 2)),
    (d(2, 2), d(5, -1)),
    (d(2, 2), d(5, 0)),
    (d(3, -1), d(5, 1)),
    (d(3, -1), d(5, 1)),
    (d(3, 0), d(5, 2)),
    (d(3, 0), d(5, 2)),
    (d(3, 1), d(6, -1)),
    (d(3, 1), d(6, -1)),
    (d(3, 2), d(6, 0)),
    (d(3, 2), d(6, 0)),
    (d(4, -1), d(6, 1)),
    (d(4, -1), d(6, 1)),
    (d(4, 0), d(6, 2)),
    (d(4, 0), d(6, 2)),
    (d(4, 1), d(7, -1)),
    (d(4, 1), d(7, -1)),
];

/// Thrust and swing damage for a given ST. ST is clamped to 1..=40
/// (values above 40 are not yet supported and use the ST 40 row).
pub fn basic_damage(st: i32) -> (DiceSpec, DiceSpec) {
    let idx = st.clamp(1, 40) as usize - 1;
    TABLE[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_rows() {
        let (t, s) = basic_damage(10);
        assert_eq!((t.to_string(), s.to_string()), ("1d-2".into(), "1d".into()));
        let (t, s) = basic_damage(13);
        assert_eq!((t.to_string(), s.to_string()), ("1d".into(), "2d-1".into()));
        let (t, s) = basic_damage(20);
        assert_eq!((t.to_string(), s.to_string()), ("2d-1".into(), "3d+2".into()));
        assert_eq!(basic_damage(0), basic_damage(1));
        assert_eq!(basic_damage(99), basic_damage(40));
    }
}
