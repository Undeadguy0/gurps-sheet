//! Spells (data only; levels are computed in `gurps-rules`).
//!
//! A spell is learned like an IQ-based Hard or Very Hard skill; Magery adds its
//! level to every spell (B235-B236, page unverified). Prerequisites are kept as
//! text and are not checked.

use crate::{is_blank, Difficulty, LocalizedText, Stat};
use serde::{Deserialize, Serialize};

fn iq() -> Stat {
    Stat::IQ
}
fn hard() -> Difficulty {
    Difficulty::Hard
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpellDef {
    pub id: String,
    pub name: String,
    /// Controlling attribute (IQ).
    #[serde(default = "iq")]
    pub attribute: Stat,
    /// Hard (most spells) or Very Hard.
    #[serde(default = "hard")]
    pub difficulty: Difficulty,
    /// College(s), e.g. ["Fire"].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub college: Vec<String>,
    /// Prerequisites as free text (not checked), e.g. "Magery 1, Ignite Fire".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerequisites: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

impl SpellDef {
    pub fn explanation(&self, locale: &str) -> Option<&str> {
        self.description.as_ref().and_then(|d| d.get(locale))
    }
}

/// A spell known by a character: definition snapshot + points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spell {
    #[serde(flatten)]
    pub def: SpellDef,
    #[serde(default)]
    pub points: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

impl Spell {
    pub fn new(def: SpellDef, points: i32) -> Self {
        Self {
            def,
            points,
            source: None,
            explanation: None,
        }
    }
    pub fn has_custom_explanation(&self) -> bool {
        !is_blank(&self.explanation)
    }
}
