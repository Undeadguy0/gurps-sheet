//! Character file format: `{ "format": "gurps-sheet", "version": 1, "character": {...} }`.
#![cfg_attr(test, allow(clippy::field_reassign_with_default))]

use gurps_core::Character;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const FORMAT: &str = "gurps-sheet";
pub const VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a gurps-sheet file (format = {0:?})")]
    WrongFormat(String),
    #[error("unsupported file version {found} (max supported {VERSION})")]
    UnsupportedVersion { found: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterFile {
    pub format: String,
    pub version: u32,
    pub character: Character,
}

impl CharacterFile {
    pub fn new(character: Character) -> Self {
        Self {
            format: FORMAT.into(),
            version: VERSION,
            character,
        }
    }
}

pub fn to_string(ch: &Character) -> Result<String, IoError> {
    Ok(serde_json::to_string_pretty(&CharacterFile::new(
        ch.clone(),
    ))?)
}

pub fn from_str(s: &str) -> Result<Character, IoError> {
    let f: CharacterFile = serde_json::from_str(s)?;
    if f.format != FORMAT {
        return Err(IoError::WrongFormat(f.format));
    }
    if f.version == 0 || f.version > VERSION {
        return Err(IoError::UnsupportedVersion { found: f.version });
    }
    Ok(f.character)
}

pub fn load(path: impl AsRef<Path>) -> Result<Character, IoError> {
    from_str(&std::fs::read_to_string(path)?)
}

pub fn save(path: impl AsRef<Path>, ch: &Character) -> Result<(), IoError> {
    let mut s = to_string(ch)?;
    s.push('\n');
    std::fs::write(path, s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gurps_core::{Equipment, Feature, Stat, Trait, TraitCategory, TraitDef};

    fn sample() -> Character {
        let mut ch = Character::new("Test");
        ch.attributes.st = 2;
        ch.attributes.basic_speed_quarters = 1;
        ch.traits.push(Trait::new(
            TraitDef {
                id: "inc".into(),
                name: "Increased ST (example)".into(),
                category: TraitCategory::Advantage,
                cost: 0,
                cost_per_level: 10,
                max_levels: None,
                page: Some("example".into()),
                features: vec![Feature::AttributeBonus {
                    target: Stat::ST,
                    amount: 1.0,
                    per_level: true,
                }],
            },
            1,
        ));
        ch.equipment.push(Equipment {
            name: "Rope".into(),
            quantity: 1,
            weight: 1.5,
            cost: 5.0,
        });
        ch
    }

    #[test]
    fn roundtrip() {
        let ch = sample();
        let s = to_string(&ch).unwrap();
        assert!(s.contains("\"version\": 1"));
        assert_eq!(from_str(&s).unwrap(), ch);
    }

    #[test]
    fn file_roundtrip() {
        let p = std::env::temp_dir().join(format!("gurps-io-{}.json", std::process::id()));
        save(&p, &sample()).unwrap();
        assert_eq!(load(&p).unwrap(), sample());
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn rejects_bad_header() {
        let s = r#"{"format":"other","version":1,"character":{}}"#;
        assert!(matches!(from_str(s), Err(IoError::WrongFormat(_))));
        let s = r#"{"format":"gurps-sheet","version":99,"character":{}}"#;
        assert!(matches!(
            from_str(s),
            Err(IoError::UnsupportedVersion { found: 99 })
        ));
    }

    #[test]
    fn missing_fields_default() {
        let s = r#"{"format":"gurps-sheet","version":1,"character":{"name":"X"}}"#;
        let ch = from_str(s).unwrap();
        assert_eq!(ch.points_budget, 150);
        assert_eq!(ch.attributes.st, 0);
    }

    #[test]
    fn hero_fixture_loads() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/hero.json");
        let ch = load(p).unwrap();
        assert_eq!(ch.attributes.st, 2);
        assert!(!ch.traits.is_empty());
        // fixture is canonical: re-serialising yields identical content
        let again = from_str(&to_string(&ch).unwrap()).unwrap();
        assert_eq!(again, ch);
    }
}
