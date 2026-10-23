//! Trait modifiers: enhancements (+%) and limitations (-%) (B101-B102, page unverified).

use crate::{is_blank, LocalizedText};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifierDef {
    pub id: String,
    pub name: String,
    /// Percentage of the trait's cost: positive = enhancement, negative = limitation.
    pub cost_percent: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

impl ModifierDef {
    pub fn explanation(&self, locale: &str) -> Option<&str> {
        self.description.as_ref().and_then(|d| d.get(locale))
    }
    pub fn is_enhancement(&self) -> bool {
        self.cost_percent > 0
    }
}

/// A modifier applied to a character's trait.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitModifier {
    #[serde(flatten)]
    pub def: ModifierDef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

impl TraitModifier {
    pub fn new(def: ModifierDef) -> Self {
        Self {
            def,
            source: None,
            explanation: None,
        }
    }
    pub fn has_custom_explanation(&self) -> bool {
        !is_blank(&self.explanation)
    }
}

/// Lowest total modifier (limitations cannot reduce a cost by more than 80%).
pub const MIN_TOTAL_PERCENT: i32 = -80;

/// Cost after modifiers: base × (100% + total%), total floored at -80%,
/// fractions rounded up (towards +∞, so -7.5 becomes -7 — see the report flag).
pub fn modified_cost(base: i32, total_percent: i32) -> i32 {
    let pct = 100 + total_percent.max(MIN_TOTAL_PERCENT);
    let n = base as i64 * pct as i64;
    n.div_euclid(100) as i32 + i32::from(n.rem_euclid(100) != 0)
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
}
