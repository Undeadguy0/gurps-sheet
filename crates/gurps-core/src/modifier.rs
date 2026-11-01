//! Trait cost modifiers: percentage math and rounding options.

use serde::{Deserialize, Serialize};

/// Lowest total modifier (limitations cannot reduce a cost by more than 80%).
pub const MIN_TOTAL_PERCENT: i32 = -80;

/// How fractional negative costs are rounded (a per-character rules option).
/// Positive costs always round up.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegativeRounding {
    /// Round up towards +∞: -7.5 -> -7 (GM-confirmed default).
    #[default]
    TowardPositive,
    /// Round the magnitude up: -7.5 -> -8.
    ByMagnitude,
}

/// Cost after modifiers: base × (100% + total%), total floored at -80%,
/// fractions rounded up (towards +∞, so -7.5 becomes -7; GM-confirmed).
pub fn modified_cost(base: i32, total_percent: i32) -> i32 {
    modified_cost_with(base, total_percent, NegativeRounding::TowardPositive)
}

/// [`modified_cost`] with an explicit rounding rule for negative results.
pub fn modified_cost_with(base: i32, total_percent: i32, r: NegativeRounding) -> i32 {
    let pct = 100 + total_percent.max(MIN_TOTAL_PERCENT);
    let n = base as i64 * pct as i64;
    let up = n.div_euclid(100) as i32 + i32::from(n.rem_euclid(100) != 0);
    if n < 0 && r == NegativeRounding::ByMagnitude && n.rem_euclid(100) != 0 {
        up - 1
    } else {
        up
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rounding_and_floor() {
        assert_eq!(modified_cost(20, 0), 20);
        assert_eq!(modified_cost(20, 40), 28);
        assert_eq!(modified_cost(15, -25), 12); // 11.25 -> 12
        assert_eq!(modified_cost(10, -95), 2); // floored at -80%
        assert_eq!(modified_cost(5, 10), 6); // 5.5 -> 6
        assert_eq!(modified_cost(-10, -25), -7); // -7.5 -> -7 (towards +inf)
        assert_eq!(modified_cost(0, 50), 0);
    }

    #[test]
    fn negative_rounding_option() {
        use NegativeRounding::*;
        assert_eq!(modified_cost_with(-10, -25, TowardPositive), -7);
        assert_eq!(modified_cost_with(-10, -25, ByMagnitude), -8);
        assert_eq!(modified_cost_with(-10, -20, ByMagnitude), -8); // exact
        assert_eq!(modified_cost_with(-13, -30, ByMagnitude), -10); // -9.1 -> -10
        assert_eq!(modified_cost_with(5, -30, ByMagnitude), 4); // positive: still up
    }
}
