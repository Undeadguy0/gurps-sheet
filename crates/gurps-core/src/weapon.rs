//! Weapons (data only; levels, damage and defences are computed in `gurps-rules`).

use crate::damage::DiceSpec;
use crate::{is_blank, LocalizedText, SkillDefault};
use serde::{Deserialize, Serialize};

/// Extra damage from high skill, e.g. Brawling at DX+2 or better: +1 per die (B182, # unverified).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillDamageBonus {
    /// Skill level minus its controlling attribute needed for the bonus.
    pub min_relative: i32,
    /// Added per die of damage.
    pub per_die: i32,
}

/// One way to attack with a weapon (swing, thrust, punch...).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackDef {
    /// "swing", "thrust" or free text.
    pub mode: String,
    /// "sw+1", "thr-1", "thr" or fixed dice like "2d+1".
    pub damage: String,
    /// "cut", "imp", "cr", "pi"...
    pub damage_type: String,
    /// "1", "C", "C,1"...
    #[serde(default)]
    pub reach: String,
    /// Parry modifier; `None` = cannot parry with this attack.
    #[serde(default)]
    pub parry: Option<i32>,
    /// Skill id used to attack (defaults to the weapon's `skill`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    /// Added to the skill level when the skill (not the technique) is used, e.g. Kick at Brawling-2.
    #[serde(default, skip_serializing_if = "crate::weapon::is_zero")]
    pub skill_modifier: i32,
    /// Technique id that replaces the skill level when the character has it (e.g. Kicking).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technique: Option<String>,
    /// Levels used when the character lacks the skill (e.g. DX, Brawling-2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub defaults: Vec<SkillDefault>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_damage_bonus: Option<SkillDamageBonus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponDef {
    pub id: String,
    pub name: String,
    /// Equipment id this weapon belongs to; `None` = always available (unarmed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment: Option<String>,
    /// Default skill id for all attacks.
    pub skill: String,
    pub attacks: Vec<AttackDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

impl WeaponDef {
    pub fn explanation(&self, locale: &str) -> Option<&str> {
        self.description.as_ref().and_then(|d| d.get(locale))
    }
}

/// A weapon on the character sheet: definition snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Weapon {
    #[serde(flatten)]
    pub def: WeaponDef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

impl Weapon {
    pub fn new(def: WeaponDef) -> Self {
        Self {
            def,
            source: None,
            explanation: None,
        }
    }
}

fn is_zero(v: &i32) -> bool {
    *v == 0
}

/// Parsed damage expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageExpr {
    Thrust(i32),
    Swing(i32),
    Fixed(DiceSpec),
}

impl DamageExpr {
    /// Parses "sw+1", "thr-1", "thr", "2d+1", "1d".
    pub fn parse(s: &str) -> Option<Self> {
        let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        let s = s.to_ascii_lowercase();
        let adds = |rest: &str| -> Option<i32> {
            if rest.is_empty() {
                Some(0)
            } else {
                rest.strip_prefix('+').unwrap_or(rest).parse().ok()
            }
        };
        if let Some(r) = s.strip_prefix("thr") {
            return adds(r).map(DamageExpr::Thrust);
        }
        if let Some(r) = s.strip_prefix("sw") {
            return adds(r).map(DamageExpr::Swing);
        }
        let (d, r) = s.split_once('d')?;
        Some(DamageExpr::Fixed(DiceSpec {
            dice: d.parse().ok()?,
            adds: adds(r)?,
        }))
    }
}

/// Parses "DB 2" / "DB2" from an equipment stats line.
pub fn parse_db(stats: &str) -> Option<i32> {
    let i = stats.find("DB")?;
    let rest = stats[i + 2..].trim_start();
    let n: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    n.parse().ok()
}

/// Parses "DR 4/2*" -> ("4/2*", 4) from an equipment stats line.
pub fn parse_dr(stats: &str) -> Option<(String, i32)> {
    let i = stats.find("DR")?;
    let text: String = stats[i + 2..]
        .trim_start()
        .chars()
        .take_while(|c| !c.is_whitespace() && *c != ',' && *c != ';')
        .collect();
    let n: String = text.chars().take_while(|c| c.is_ascii_digit()).collect();
    Some((text.clone(), n.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_damage() {
        assert_eq!(DamageExpr::parse("sw+1"), Some(DamageExpr::Swing(1)));
        assert_eq!(DamageExpr::parse("thr - 1"), Some(DamageExpr::Thrust(-1)));
        assert_eq!(DamageExpr::parse("thr"), Some(DamageExpr::Thrust(0)));
        assert_eq!(
            DamageExpr::parse("2d+1"),
            Some(DamageExpr::Fixed(DiceSpec { dice: 2, adds: 1 }))
        );
        assert_eq!(DamageExpr::parse("bogus"), None);
    }

    #[test]
    fn parse_stats() {
        assert_eq!(parse_db("DB 2"), Some(2));
        assert_eq!(parse_db("DB1"), Some(1));
        assert_eq!(parse_db("DR 4"), None);
        assert_eq!(parse_dr("DR 4/2*"), Some(("4/2*".into(), 4)));
        assert_eq!(parse_dr("DB 2, DR 1"), Some(("1".into(), 1)));
    }
}
