//! Domain model for a GURPS 4th edition character (sheet file format v2).
//!
//! A character stores references `(pack, sid)` to the shared catalog plus inline
//! definitions for the user's own content; codes (`ST`, `H`, `adv`) are plain strings.
//! All calculations live in `gurps-calc`.

pub mod defs;
pub mod ledger;
pub mod modifier;

pub use defs::{
    ArmorLoc, AttackDefault, AttackMode, Damage, Def, ItemDef, ModifierDef, ShieldDef,
    SkillDamageBonus, SkillDef, SkillDefault, SpellDef, TechniqueDef, TraitCost, TraitDef,
};
pub use ledger::{LedgerEntry, LedgerKind, LedgerTotals};
pub use modifier::{modified_cost, modified_cost_with, NegativeRounding};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Pack code of the user's own content (stored inline).
pub const CUSTOM_PACK: &str = "custom";

/// Attribute codes the sheet buys levels in (Speed in quarter-steps).
pub const ATTRIBUTES: [&str; 10] = [
    "ST", "DX", "IQ", "HT", "HP", "Will", "Per", "FP", "Speed", "Move",
];

/// Where a definition comes from: a catalog reference or an inline definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Src<D> {
    Inline { inline: D },
    Ref { pack: String, sid: String },
}

impl<D: Def> Src<D> {
    pub fn catalog(pack: impl Into<String>, sid: impl Into<String>) -> Self {
        Src::Ref {
            pack: pack.into(),
            sid: sid.into(),
        }
    }
    pub fn sid(&self) -> &str {
        match self {
            Src::Inline { inline } => inline.sid(),
            Src::Ref { sid, .. } => sid,
        }
    }
    /// Catalog pack, `custom` for inline definitions.
    pub fn pack(&self) -> &str {
        match self {
            Src::Inline { .. } => CUSTOM_PACK,
            Src::Ref { pack, .. } => pack,
        }
    }
    pub fn inline(&self) -> Option<&D> {
        match self {
            Src::Inline { inline } => Some(inline),
            Src::Ref { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    #[serde(flatten)]
    pub src: Src<ModifierDef>,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trait {
    #[serde(flatten)]
    pub src: Src<TraitDef>,
    /// Level of a leveled trait (absent for fixed-cost traits).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<Modifier>,
    /// User's own explanation (Markdown); overrides the catalog text when non-blank.
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    #[serde(flatten)]
    pub src: Src<SkillDef>,
    #[serde(default)]
    pub points: i32,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub spec: Option<String>,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Technique {
    #[serde(flatten)]
    pub src: Src<TechniqueDef>,
    #[serde(default)]
    pub points: i32,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spell {
    #[serde(flatten)]
    pub src: Src<SpellDef>,
    #[serde(default)]
    pub points: i32,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

fn one() -> u32 {
    1
}
fn yes() -> bool {
    true
}
fn is_one(v: &u32) -> bool {
    *v == 1
}
fn is_true(v: &bool) -> bool {
    *v
}

/// Owned item; `contents` for containers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    #[serde(flatten)]
    pub src: Src<ItemDef>,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub qty: u32,
    /// Carried/equipped (counts for weight, armor, attacks).
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub equipped: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contents: Vec<Item>,
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

impl Item {
    pub fn new(src: Src<ItemDef>) -> Self {
        Self {
            src,
            qty: 1,
            equipped: true,
            contents: Vec::new(),
            explanation: None,
        }
    }
}

/// Multi-line Markdown text from plugin data: either a plain string or a
/// table keyed by locale (`ru`, `en`, ...).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LocalizedText {
    Plain(String),
    Localized(BTreeMap<String, String>),
}

impl LocalizedText {
    /// Text for `locale` with fallbacks: exact locale, its language part
    /// (`ru-RU` -> `ru`), then `en`, then `ru`, then any non-empty entry.
    /// Blank strings count as missing.
    pub fn get(&self, locale: &str) -> Option<&str> {
        fn ok(s: &str) -> Option<&str> {
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        }
        match self {
            LocalizedText::Plain(s) => ok(s),
            LocalizedText::Localized(m) => {
                let lang = locale.split(['-', '_']).next().unwrap_or(locale);
                [locale, lang, "en", "ru"]
                    .iter()
                    .find_map(|k| m.get(*k).and_then(|s| ok(s)))
                    .or_else(|| m.values().find_map(|s| ok(s)))
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.get("en").is_none()
    }
}

impl From<&str> for LocalizedText {
    fn from(s: &str) -> Self {
        LocalizedText::Plain(s.to_string())
    }
}

/// True for `None` or whitespace-only text (used to skip empty fields in JSON).
pub fn is_blank(s: &Option<String>) -> bool {
    s.as_deref().is_none_or(|t| t.trim().is_empty())
}

/// Normalises user input: whitespace-only text becomes `None`.
pub fn non_blank(s: Option<String>) -> Option<String> {
    s.filter(|t| !t.trim().is_empty())
}

/// Current values at the table. `None` = at maximum.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fp: Option<i32>,
}

impl Session {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// Per-character rules options.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RulesOptions {
    /// Rounding of fractional negative trait costs after modifiers.
    pub negative_rounding: NegativeRounding,
    /// Disadvantage limit in points (positive); None = 50% of starting points.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disadvantage_limit: Option<i32>,
}

impl RulesOptions {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// The user's own definitions (shown in the libraries as «своё»; entries embed copies inline).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomContent {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<TraitDef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<SkillDef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spells: Vec<SpellDef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemDef>,
}

impl CustomContent {
    pub fn is_empty(&self) -> bool {
        self.traits.is_empty()
            && self.skills.is_empty()
            && self.spells.is_empty()
            && self.items.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Character {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub player: String,
    /// Point pool ledger (starting points, earned points, spends outside the sheet).
    pub ledger: Vec<LedgerEntry>,
    #[serde(skip_serializing_if = "CustomContent::is_empty")]
    pub custom: CustomContent,
    /// Purchased levels by attribute code (`ST`, ..., `Speed` in quarters, `Move`).
    pub attributes: BTreeMap<String, i32>,
    pub traits: Vec<Trait>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<Skill>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub techniques: Vec<Technique>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spells: Vec<Spell>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Item>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(skip_serializing_if = "RulesOptions::is_default")]
    pub rules: RulesOptions,
    #[serde(skip_serializing_if = "Session::is_default")]
    pub session: Session,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            name: "New character".into(),
            player: String::new(),
            ledger: vec![LedgerEntry {
                id: 1,
                date: String::new(),
                amount: 150,
                kind: LedgerKind::Starting,
                note: String::new(),
                skill: None,
            }],
            custom: CustomContent::default(),
            attributes: BTreeMap::new(),
            traits: Vec::new(),
            skills: Vec::new(),
            techniques: Vec::new(),
            spells: Vec::new(),
            items: Vec::new(),
            notes: String::new(),
            rules: RulesOptions::default(),
            session: Session::default(),
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

    pub fn ledger_totals(&self) -> LedgerTotals {
        ledger::totals(&self.ledger)
    }

    /// Points the sheet may cost (earned − spends outside the sheet).
    pub fn available_points(&self) -> i32 {
        self.ledger_totals().available
    }

    /// Purchased levels of an attribute code (0 when absent).
    pub fn attribute(&self, code: &str) -> i32 {
        self.attributes.get(code).copied().unwrap_or(0)
    }

    /// Sets purchased levels (0 removes the key).
    pub fn set_attribute(&mut self, code: &str, levels: i32) {
        if levels == 0 {
            self.attributes.remove(code);
        } else {
            self.attributes.insert(code.to_string(), levels);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn localized_fallback() {
        let mut m = BTreeMap::new();
        m.insert("ru".to_string(), "по-русски".to_string());
        m.insert("en".to_string(), "in English".to_string());
        let t = LocalizedText::Localized(m.clone());
        assert_eq!(t.get("ru"), Some("по-русски"));
        assert_eq!(t.get("ru-RU"), Some("по-русски"));
        assert_eq!(t.get("de"), Some("in English"));
        m.remove("en");
        let t = LocalizedText::Localized(m);
        assert_eq!(t.get("en"), Some("по-русски"));
        let mut only_de = BTreeMap::new();
        only_de.insert("de".to_string(), "Deutsch".to_string());
        only_de.insert("en".to_string(), "  ".to_string());
        assert_eq!(LocalizedText::Localized(only_de).get("ru"), Some("Deutsch"));
        assert_eq!(LocalizedText::from("plain").get("ru"), Some("plain"));
        assert!(LocalizedText::from("   ").is_empty());
    }

    #[test]
    fn src_forms() {
        let t: Trait =
            serde_json::from_value(json!({"pack": "basic", "sid": "fit", "level": 0})).unwrap();
        assert_eq!(t.level, Some(0));
        assert_eq!(t.src, Src::catalog("basic", "fit"));
        let t: Trait = serde_json::from_value(json!({"inline": {"sid": "x", "name": "X", "kind": "adv", "cost": {"type": "fixed", "base": 5}}, "level": 2})).unwrap();
        assert_eq!(t.src.sid(), "x");
        assert_eq!(t.src.pack(), CUSTOM_PACK);
        assert_eq!(t.level, Some(2));
        let back = serde_json::to_value(&t).unwrap();
        assert_eq!(back["inline"]["cost"]["base"], 5);
        let i: Item = serde_json::from_value(json!({"pack": "basic", "sid": "rope"})).unwrap();
        assert!(i.equipped);
        assert_eq!(i.qty, 1);
    }
}
