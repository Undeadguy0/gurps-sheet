//! Skills and techniques (data only; levels are computed in `gurps-rules`).

use crate::{is_blank, LocalizedText, Stat};
use serde::{Deserialize, Serialize};

/// Skill difficulty. Serialized as `E`, `A`, `H`, `VH`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Difficulty {
    #[serde(rename = "E")]
    Easy,
    #[default]
    #[serde(rename = "A")]
    Average,
    #[serde(rename = "H")]
    Hard,
    #[serde(rename = "VH")]
    VeryHard,
}

impl Difficulty {
    pub fn key(self) -> &'static str {
        match self {
            Difficulty::Easy => "E",
            Difficulty::Average => "A",
            Difficulty::Hard => "H",
            Difficulty::VeryHard => "VH",
        }
    }

    /// Relative level of a 1-point skill (E 0, A -1, H -2, VH -3).
    pub fn base_offset(self) -> i32 {
        match self {
            Difficulty::Easy => 0,
            Difficulty::Average => -1,
            Difficulty::Hard => -2,
            Difficulty::VeryHard => -3,
        }
    }
}

/// A default a skill can be used at without points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillDefault {
    /// `{ attribute = "DX", modifier = -5 }`
    Attribute { attribute: Stat, modifier: i32 },
    /// `{ skill = "broadsword", modifier = -2 }` (any specialization of that skill).
    Skill { skill: String, modifier: i32 },
}

/// Skill definition from a plugin library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    /// Controlling attribute (default DX).
    #[serde(default = "default_attr")]
    pub attribute: Stat,
    #[serde(default)]
    pub difficulty: Difficulty,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub defaults: Vec<SkillDefault>,
    /// The skill must be taken with a specialization (e.g. Survival (Woodlands)).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub specialization_required: bool,
    /// Suggested specializations (free text is allowed too).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub specializations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

fn default_attr() -> Stat {
    Stat::DX
}

impl SkillDef {
    pub fn explanation(&self, locale: &str) -> Option<&str> {
        self.description.as_ref().and_then(|d| d.get(locale))
    }
}

/// A skill known by a character: definition snapshot + points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    #[serde(flatten)]
    pub def: SkillDef,
    /// Character points spent (0 = used at default).
    #[serde(default)]
    pub points: i32,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub specialization: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

impl Skill {
    pub fn new(def: SkillDef, points: i32) -> Self {
        Self {
            def,
            points,
            specialization: None,
            source: None,
            explanation: None,
        }
    }

    /// "Survival (Woodlands)".
    pub fn display_name(&self) -> String {
        match self.specialization.as_deref().map(str::trim) {
            Some(s) if !s.is_empty() => format!("{} ({s})", self.def.name),
            _ => self.def.name.clone(),
        }
    }

    pub fn has_custom_explanation(&self) -> bool {
        !is_blank(&self.explanation)
    }
}

/// Technique definition: a narrow manoeuvre bought up from a parent skill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechniqueDef {
    pub id: String,
    pub name: String,
    /// Parent skill id.
    pub skill: String,
    /// Average or Hard only.
    #[serde(default)]
    pub difficulty: Difficulty,
    /// Default: parent skill + modifier (usually negative).
    pub modifier: i32,
    /// Highest level relative to the parent skill (0 = up to the skill itself).
    #[serde(default)]
    pub max: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

impl TechniqueDef {
    pub fn explanation(&self, locale: &str) -> Option<&str> {
        self.description.as_ref().and_then(|d| d.get(locale))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Technique {
    #[serde(flatten)]
    pub def: TechniqueDef,
    #[serde(default)]
    pub points: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

impl Technique {
    pub fn new(def: TechniqueDef, points: i32) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_shapes() {
        let s: SkillDef = toml::from_str(
            r#"
id = "survival"
name = "Survival"
attribute = "Per"
difficulty = "A"
specialization_required = true
defaults = [{ attribute = "Per", modifier = -5 }, { skill = "naturalist", modifier = -3 }]
"#,
        )
        .unwrap();
        assert_eq!(s.attribute, Stat::Per);
        assert_eq!(
            s.defaults[1],
            SkillDefault::Skill {
                skill: "naturalist".into(),
                modifier: -3
            }
        );
        let min: SkillDef = toml::from_str("id = \"x\"\nname = \"X\"\n").unwrap();
        assert_eq!(
            (min.attribute, min.difficulty),
            (Stat::DX, Difficulty::Average)
        );
        let mut sk = Skill::new(s, 2);
        sk.specialization = Some("Woodlands".into());
        assert_eq!(sk.display_name(), "Survival (Woodlands)");
        let j = serde_json::to_string(&sk).unwrap();
        assert!(j.contains("\"difficulty\":\"A\""));
        assert_eq!(serde_json::from_str::<Skill>(&j).unwrap(), sk);
    }
}
