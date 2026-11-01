//! Definitions (catalog entries and the user's own inline content).
//!
//! Shapes follow the shared DB/calc format: references are plain codes
//! (`ST`, `H`, `adv`) or catalog sids (`broadsword`), never enums.

use crate::LocalizedText;
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn zero(v: &i64) -> bool {
    *v == 0
}

/// Common identity of every definition.
pub trait Def {
    fn sid(&self) -> &str;
    fn name(&self) -> &str;
    fn page(&self) -> Option<&str>;
    fn description(&self) -> Option<&LocalizedText>;
    fn explanation(&self, locale: &str) -> Option<&str> {
        self.description().and_then(|d| d.get(locale))
    }
}

macro_rules! def {
    ($($t:ty),*) => {$(
        impl Def for $t {
            fn sid(&self) -> &str { &self.sid }
            fn name(&self) -> &str { &self.name }
            fn page(&self) -> Option<&str> { self.page.as_deref() }
            fn description(&self) -> Option<&LocalizedText> { self.description.as_ref() }
        }
    )*};
}
def!(
    TraitDef,
    ModifierDef,
    SkillDef,
    TechniqueDef,
    SpellDef,
    ItemDef
);

/// Trait cost: `fixed` (base), `per_level` (per_level × level) or
/// `base_plus_level` (base + per_level × level).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TraitCost {
    #[serde(rename = "type")]
    pub kind: String,
    pub base: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_level: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TraitDef {
    pub sid: String,
    pub name: String,
    /// `adv` / `disadv` / `perk` / `quirk`.
    pub kind: String,
    pub cost: TraitCost,
    /// Features in the DB/calc shape (`{kind, attribute, amount, per_level, ...}`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<Value>,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub prereqs: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

impl TraitDef {
    pub fn is_leveled(&self) -> bool {
        self.cost.per_level.is_some_and(|p| p != 0)
    }
    /// Effective minimum level (1 for leveled traits unless overridden, else 0).
    pub fn min_level(&self) -> u32 {
        self.cost.min_level.unwrap_or(u32::from(
            self.cost.kind == "per_level" && self.is_leveled(),
        ))
    }
    pub fn base_cost(&self, level: i32) -> i64 {
        let pl = self.cost.per_level.unwrap_or(0) * level as i64;
        match self.cost.kind.as_str() {
            "per_level" => pl,
            "base_plus_level" => self.cost.base + pl,
            _ => self.cost.base,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModifierDef {
    pub sid: String,
    pub name: String,
    /// `enh` / `lim`.
    #[serde(rename = "type")]
    pub kind: String,
    /// `%`, `per_level`, `flat`.
    pub value_kind: String,
    /// Signed amount (limitations negative).
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

impl ModifierDef {
    /// Signed percentage (0 for non-percent modifiers).
    pub fn percent(&self) -> i32 {
        if self.value_kind != "%" {
            return 0;
        }
        let a = self.amount.abs().round() as i32;
        if self.kind == "lim" {
            -a
        } else {
            a
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillDefault {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    #[serde(rename = "mod")]
    pub modifier: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillDef {
    pub sid: String,
    pub name: String,
    /// Attribute code (`DX`, `IQ`, ...).
    pub attribute: String,
    /// Difficulty code (`E`, `A`, `H`, `VH`).
    pub difficulty: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub defaults: Vec<SkillDefault>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub spec_required: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub specializations: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<Value>,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub prereqs: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TechniqueDef {
    pub sid: String,
    pub name: String,
    /// Parent skill sid.
    pub parent_skill: String,
    /// `A` / `H`.
    pub difficulty: String,
    pub default_mod: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_mod: Option<i64>,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub prereqs: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpellDef {
    pub sid: String,
    pub name: String,
    /// College codes.
    pub colleges: Vec<String>,
    pub difficulty: String,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub prereqs: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ArmorLoc {
    pub loc: String,
    pub dr: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShieldDef {
    pub db: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_skill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_spec: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Damage {
    /// `thr` / `sw` / `fixed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dice: Option<i64>,
    #[serde(rename = "mod", skip_serializing_if = "zero")]
    pub add: i64,
    /// Damage type code (`cut`, `imp`, `cr`, ...).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AttackDefault {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    #[serde(rename = "mod")]
    pub modifier: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillDamageBonus {
    pub min_relative: i64,
    pub per_die: i64,
}

/// Attack mode of an item (or a natural attack).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AttackMode {
    pub sid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub natural: bool,
    /// Skill sid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    pub damage: Damage,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reach: Vec<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parry: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub defaults: Vec<AttackDefault>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_damage_bonus: Option<SkillDamageBonus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technique: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ItemDef {
    pub sid: String,
    pub name: String,
    /// Pounds.
    pub weight: f64,
    /// $.
    pub cost: f64,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub container: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub armor: Vec<ArmorLoc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shield: Option<ShieldDef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attack_modes: Vec<AttackMode>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_cost_kinds() {
        let mut d = TraitDef {
            cost: TraitCost {
                kind: "per_level".into(),
                per_level: Some(3),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(d.base_cost(4), 12);
        assert_eq!(d.min_level(), 1);
        d.cost.kind = "base_plus_level".into();
        d.cost.base = 5;
        d.cost.per_level = Some(10);
        assert_eq!(d.base_cost(0), 5);
        assert_eq!(d.min_level(), 0);
    }

    #[test]
    fn modifier_percent_sign() {
        let m = ModifierDef {
            kind: "lim".into(),
            value_kind: "%".into(),
            amount: -10.0,
            ..Default::default()
        };
        assert_eq!(m.percent(), -10);
    }
}
