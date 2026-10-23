//! Domain model for a GURPS 4th edition character.
//!
//! This crate contains only data types; all calculations live in `gurps-rules`.

pub mod damage;

use serde::{Deserialize, Serialize};
use std::fmt;

/// A value that can be targeted by features and shown on the sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stat {
    ST,
    DX,
    IQ,
    HT,
    HP,
    Will,
    Per,
    FP,
    BasicSpeed,
    BasicMove,
    BasicLift,
}

impl Stat {
    pub const PRIMARY: [Stat; 4] = [Stat::ST, Stat::DX, Stat::IQ, Stat::HT];
    pub const SECONDARY: [Stat; 6] = [
        Stat::HP,
        Stat::Will,
        Stat::Per,
        Stat::FP,
        Stat::BasicSpeed,
        Stat::BasicMove,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Stat::ST => "ST",
            Stat::DX => "DX",
            Stat::IQ => "IQ",
            Stat::HT => "HT",
            Stat::HP => "HP",
            Stat::Will => "Will",
            Stat::Per => "Per",
            Stat::FP => "FP",
            Stat::BasicSpeed => "BasicSpeed",
            Stat::BasicMove => "BasicMove",
            Stat::BasicLift => "BasicLift",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Stat::BasicSpeed => "Basic Speed",
            Stat::BasicMove => "Basic Move",
            Stat::BasicLift => "Basic Lift",
            s => s.key(),
        }
    }

    pub fn parse(s: &str) -> Option<Stat> {
        let all = [Stat::PRIMARY.as_slice(), Stat::SECONDARY.as_slice(), &[Stat::BasicLift]];
        all.iter()
            .flat_map(|x| x.iter())
            .copied()
            .find(|st| st.key().eq_ignore_ascii_case(s) || st.label().eq_ignore_ascii_case(s))
    }

    /// Size of one purchasable step (Basic Speed is bought in 0.25 steps).
    pub fn step(self) -> f64 {
        if self == Stat::BasicSpeed {
            0.25
        } else {
            1.0
        }
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Purchased levels (above/below the base value) for attributes and secondary characteristics.
/// Basic Speed is stored in quarter-steps (1 = +0.25).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Attributes {
    pub st: i32,
    pub dx: i32,
    pub iq: i32,
    pub ht: i32,
    pub hp: i32,
    pub will: i32,
    pub per: i32,
    pub fp: i32,
    pub basic_speed_quarters: i32,
    pub basic_move: i32,
}

impl Attributes {
    pub fn purchased(&self, stat: Stat) -> i32 {
        match stat {
            Stat::ST => self.st,
            Stat::DX => self.dx,
            Stat::IQ => self.iq,
            Stat::HT => self.ht,
            Stat::HP => self.hp,
            Stat::Will => self.will,
            Stat::Per => self.per,
            Stat::FP => self.fp,
            Stat::BasicSpeed => self.basic_speed_quarters,
            Stat::BasicMove => self.basic_move,
            Stat::BasicLift => 0,
        }
    }

    /// Sets purchased levels. Returns false for stats that cannot be bought directly.
    pub fn set_purchased(&mut self, stat: Stat, levels: i32) -> bool {
        let slot = match stat {
            Stat::ST => &mut self.st,
            Stat::DX => &mut self.dx,
            Stat::IQ => &mut self.iq,
            Stat::HT => &mut self.ht,
            Stat::HP => &mut self.hp,
            Stat::Will => &mut self.will,
            Stat::Per => &mut self.per,
            Stat::FP => &mut self.fp,
            Stat::BasicSpeed => &mut self.basic_speed_quarters,
            Stat::BasicMove => &mut self.basic_move,
            Stat::BasicLift => return false,
        };
        *slot = levels;
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TraitCategory {
    #[default]
    Advantage,
    Perk,
    Disadvantage,
    Quirk,
}

/// A mechanical effect of a trait.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Feature {
    /// Adds `amount` (per level if `per_level`) to a stat.
    AttributeBonus {
        target: Stat,
        amount: f64,
        #[serde(default)]
        per_level: bool,
    },
    /// Adds to ST only for the purpose of Basic Lift (e.g. Lifting ST).
    LiftingStBonus {
        amount: f64,
        #[serde(default)]
        per_level: bool,
    },
}

/// Trait definition (from a plugin library).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: TraitCategory,
    /// Fixed cost.
    #[serde(default)]
    pub cost: i32,
    /// Additional cost per level (leveled traits).
    #[serde(default)]
    pub cost_per_level: i32,
    #[serde(default)]
    pub max_levels: Option<u32>,
    /// Page reference, e.g. "B65".
    #[serde(default)]
    pub page: Option<String>,
    #[serde(default)]
    pub features: Vec<Feature>,
}

impl TraitDef {
    pub fn is_leveled(&self) -> bool {
        self.cost_per_level != 0
    }
}

/// A trait taken by a character: a snapshot of its definition plus chosen levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trait {
    #[serde(flatten)]
    pub def: TraitDef,
    #[serde(default)]
    pub levels: u32,
    /// Plugin the trait came from (informational).
    #[serde(default)]
    pub source: Option<String>,
}

impl Trait {
    pub fn new(def: TraitDef, levels: u32) -> Self {
        Self {
            def,
            levels,
            source: None,
        }
    }

    pub fn cost(&self) -> i32 {
        self.def.cost + self.def.cost_per_level * self.levels as i32
    }

    /// Display name including level, e.g. "Lifting ST 3".
    pub fn display_name(&self) -> String {
        if self.def.is_leveled() {
            format!("{} {}", self.def.name, self.levels)
        } else {
            self.def.name.clone()
        }
    }
}

/// Equipment stub (weight in lbs, cost in $).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Equipment {
    pub name: String,
    pub quantity: u32,
    pub weight: f64,
    pub cost: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Character {
    pub name: String,
    pub player: String,
    /// Starting point budget.
    pub points_budget: i32,
    pub attributes: Attributes,
    pub traits: Vec<Trait>,
    pub equipment: Vec<Equipment>,
    pub notes: String,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            name: "New character".into(),
            player: String::new(),
            points_budget: 150,
            attributes: Attributes::default(),
            traits: Vec::new(),
            equipment: Vec::new(),
            notes: String::new(),
        }
    }
}

impl Character {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_parse_roundtrip() {
        for s in Stat::PRIMARY.iter().chain(Stat::SECONDARY.iter()) {
            assert_eq!(Stat::parse(s.key()), Some(*s));
        }
        assert_eq!(Stat::parse("basic speed"), Some(Stat::BasicSpeed));
        assert_eq!(Stat::parse("nope"), None);
    }

    #[test]
    fn trait_cost_and_name() {
        let def = TraitDef {
            id: "x".into(),
            name: "Lifting ST".into(),
            category: TraitCategory::Advantage,
            cost: 0,
            cost_per_level: 3,
            max_levels: None,
            page: Some("B65".into()),
            features: vec![],
        };
        let t = Trait::new(def, 4);
        assert_eq!(t.cost(), 12);
        assert_eq!(t.display_name(), "Lifting ST 4");
    }

    #[test]
    fn set_purchased() {
        let mut a = Attributes::default();
        assert!(a.set_purchased(Stat::ST, 2));
        assert_eq!(a.purchased(Stat::ST), 2);
        assert!(!a.set_purchased(Stat::BasicLift, 1));
    }
}
