//! Character file format: `{ "format": "gurps-sheet", "version": 2, "character": {...} }`.
//!
//! Version 2 stores catalog references `(pack, sid)` plus inline definitions for own
//! content. Version 1 (definition snapshots) is not read: such files are rejected.

use gurps_core::Character;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const FORMAT: &str = "gurps-sheet";
pub const VERSION: u32 = 2;

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a gurps-sheet file (format = {0:?})")]
    WrongFormat(String),
    #[error("unsupported file version {found} (supported: {VERSION})")]
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
    #[derive(Deserialize)]
    struct Header {
        format: String,
        version: u32,
    }
    let h: Header = serde_json::from_str(s)?;
    if h.format != FORMAT {
        return Err(IoError::WrongFormat(h.format));
    }
    if h.version != VERSION {
        return Err(IoError::UnsupportedVersion { found: h.version });
    }
    let f: CharacterFile = serde_json::from_str(s)?;
    Ok(f.character)
}

pub fn load(path: impl AsRef<Path>) -> Result<Character, IoError> {
    from_str(&std::fs::read_to_string(path)?)
}

/// Atomic save: writes `<name>.tmp` next to the target, flushes it, then renames
/// it over the target, so a failed or interrupted save never truncates the old file.
pub fn save(path: impl AsRef<Path>, ch: &Character) -> Result<(), IoError> {
    use std::io::Write;
    let path = path.as_ref();
    let mut s = to_string(ch)?;
    s.push('\n');
    let mut tmp_name = path.file_name().unwrap_or_default().to_os_string();
    tmp_name.push(".tmp");
    let tmp = path.with_file_name(tmp_name);
    let res = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(s.as_bytes())?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if res.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    res?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gurps_core::{Item, Src, Trait, TraitCost, TraitDef};

    fn sample() -> Character {
        let mut c = Character::new("Test");
        c.set_attribute("ST", 2);
        c.traits.push(Trait {
            src: Src::catalog("basic", "combat_reflexes"),
            level: None,
            modifiers: vec![],
            explanation: Some("mine".into()),
        });
        c.traits.push(Trait {
            src: Src::Inline {
                inline: TraitDef {
                    sid: "lucky_hat".into(),
                    name: "Lucky Hat".into(),
                    kind: "perk".into(),
                    cost: TraitCost {
                        kind: "fixed".into(),
                        base: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            level: None,
            modifiers: vec![],
            explanation: None,
        });
        let mut bag = Item::new(Src::catalog("basic", "backpack_small"));
        bag.contents
            .push(Item::new(Src::catalog("basic", "rope_10_yards")));
        c.items.push(bag);
        c
    }

    #[test]
    fn roundtrip() {
        let c = sample();
        let s = to_string(&c).unwrap();
        assert!(s.contains("\"version\": 2"));
        assert_eq!(from_str(&s).unwrap(), c);
    }

    #[test]
    fn atomic_save_replaces_and_leaves_no_temp() {
        let dir = std::env::temp_dir().join(format!("gurps-io-atomic-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("c.gurps.json");
        std::fs::write(&p, "old").unwrap();
        save(&p, &sample()).unwrap();
        assert_eq!(load(&p).unwrap().name, "Test");
        assert!(!dir.join("c.gurps.json.tmp").exists());
        assert!(save(dir.join("nope/c.json"), &sample()).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_bad_header_and_v1() {
        assert!(matches!(
            from_str(r#"{"format":"x","version":2,"character":{}}"#),
            Err(IoError::WrongFormat(_))
        ));
        assert!(matches!(
            from_str(r#"{"format":"gurps-sheet","version":1,"character":{"points_budget":100}}"#),
            Err(IoError::UnsupportedVersion { found: 1 })
        ));
    }

    #[test]
    fn missing_fields_default() {
        let c =
            from_str(r#"{"format":"gurps-sheet","version":2,"character":{"name":"N"}}"#).unwrap();
        assert_eq!(c.name, "N");
        assert_eq!(c.available_points(), 150);
        assert!(c.traits.is_empty());
    }

    #[test]
    fn hero_example_loads() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/hero.json");
        let c = load(p).unwrap();
        assert!(!c.traits.is_empty());
    }
}
