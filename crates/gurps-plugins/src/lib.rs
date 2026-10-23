//! Plugin loading.
//!
//! A plugin is a directory containing `plugin.toml`:
//! ```toml
//! id = "basic-set-sample"
//! name = "Basic Set (sample)"
//! version = "0.1.0"
//! depends = []
//! ```
//! and any number of `data/*.toml` files with `[[traits]]` tables
//! (see [`gurps_core::TraitDef`]), `[[skills]]` ([`gurps_core::SkillDef`]),
//! `[[techniques]]` ([`gurps_core::TechniqueDef`]) and, optionally, `[[spells]]`,
//! `[[equipment]]`, `[[weapons]]`, `[[modifiers]]` (see [`gurps_core::LibraryItemDef`]).
//!
//! Every entry may carry an optional Markdown `description`, either a plain
//! string or a localized table:
//! ```toml
//! description = "Short text"
//! # or
//! [traits.description]
//! ru = '''Многострочный текст'''
//! en = '''Multi-line text'''
//! ```
#![cfg_attr(test, allow(clippy::field_reassign_with_default))]

pub mod l10n;
pub mod script;

pub use l10n::{Catalog, LocEntry};

use gurps_core::{LibraryItemDef, SkillDef, TechniqueDef, TraitDef};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parse error in {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("plugin '{plugin}' depends on missing plugin '{missing}'")]
    MissingDependency { plugin: String, missing: String },
    #[error("dependency cycle: {}", .0.join(" -> "))]
    Cycle(Vec<String>),
    #[error("{0}")]
    Locale(String),
    #[error("duplicate plugin id '{0}'")]
    Duplicate(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub depends: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DataFile {
    #[serde(default)]
    pub traits: Vec<TraitDef>,
    #[serde(default)]
    pub skills: Vec<SkillDef>,
    #[serde(default)]
    pub techniques: Vec<TechniqueDef>,
    #[serde(default)]
    pub spells: Vec<LibraryItemDef>,
    #[serde(default)]
    pub equipment: Vec<LibraryItemDef>,
    #[serde(default)]
    pub weapons: Vec<LibraryItemDef>,
    #[serde(default)]
    pub modifiers: Vec<LibraryItemDef>,
}

impl DataFile {
    fn extend(&mut self, o: DataFile) {
        self.traits.extend(o.traits);
        self.skills.extend(o.skills);
        self.techniques.extend(o.techniques);
        self.spells.extend(o.spells);
        self.equipment.extend(o.equipment);
        self.weapons.extend(o.weapons);
        self.modifiers.extend(o.modifiers);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plugin {
    pub manifest: Manifest,
    pub traits: Vec<TraitDef>,
    #[serde(default)]
    pub skills: Vec<SkillDef>,
    #[serde(default)]
    pub techniques: Vec<TechniqueDef>,
    #[serde(default)]
    pub spells: Vec<LibraryItemDef>,
    #[serde(default)]
    pub equipment: Vec<LibraryItemDef>,
    #[serde(default)]
    pub weapons: Vec<LibraryItemDef>,
    #[serde(default)]
    pub modifiers: Vec<LibraryItemDef>,
    /// Fluent translations from `locales/<lang>/*.ftl`.
    #[serde(default)]
    pub locales: Catalog,
    #[serde(skip)]
    pub path: PathBuf,
}

fn read(path: &Path) -> Result<String, PluginError> {
    std::fs::read_to_string(path).map_err(|source| PluginError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn parse<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, PluginError> {
    toml::from_str(&read(path)?).map_err(|source| PluginError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

/// Loads one plugin directory.
pub fn load_plugin(dir: &Path) -> Result<Plugin, PluginError> {
    let manifest: Manifest = parse(&dir.join("plugin.toml"))?;
    let mut all = DataFile::default();
    let data = dir.join("data");
    if data.is_dir() {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&data)
            .map_err(|source| PluginError::Io {
                path: data.clone(),
                source,
            })?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .collect();
        files.sort();
        for f in files {
            let df: DataFile = parse(&f)?;
            all.extend(df);
        }
    }
    Ok(Plugin {
        manifest,
        traits: all.traits,
        skills: all.skills,
        techniques: all.techniques,
        spells: all.spells,
        equipment: all.equipment,
        weapons: all.weapons,
        modifiers: all.modifiers,
        locales: l10n::load_dir(dir)?,
        path: dir.to_path_buf(),
    })
}

/// Loads every sub-directory of `root` that contains a `plugin.toml`,
/// returned in dependency order.
pub fn load_all(root: &Path) -> Result<Vec<Plugin>, PluginError> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
        .map_err(|source| PluginError::Io {
            path: root.to_path_buf(),
            source,
        })?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.join("plugin.toml").is_file())
        .collect();
    dirs.sort();
    let plugins = dirs
        .iter()
        .map(|d| load_plugin(d))
        .collect::<Result<Vec<_>, _>>()?;
    let order = resolve_order(
        &plugins
            .iter()
            .map(|p| p.manifest.clone())
            .collect::<Vec<_>>(),
    )?;
    let mut by_id: BTreeMap<String, Plugin> = plugins
        .into_iter()
        .map(|p| (p.manifest.id.clone(), p))
        .collect();
    Ok(order
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .collect())
}

/// Topological order (dependencies first; ties broken alphabetically).
pub fn resolve_order(manifests: &[Manifest]) -> Result<Vec<String>, PluginError> {
    let mut map: BTreeMap<&str, &Manifest> = BTreeMap::new();
    for m in manifests {
        if map.insert(m.id.as_str(), m).is_some() {
            return Err(PluginError::Duplicate(m.id.clone()));
        }
    }
    for m in manifests {
        for d in &m.depends {
            if !map.contains_key(d.as_str()) {
                return Err(PluginError::MissingDependency {
                    plugin: m.id.clone(),
                    missing: d.clone(),
                });
            }
        }
    }
    // DFS with colors for deterministic order + cycle path reporting.
    let mut done: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    fn visit<'a>(
        id: &'a str,
        map: &BTreeMap<&'a str, &'a Manifest>,
        stack: &mut Vec<&'a str>,
        done: &mut BTreeSet<&'a str>,
        out: &mut Vec<String>,
    ) -> Result<(), PluginError> {
        if done.contains(id) {
            return Ok(());
        }
        if let Some(pos) = stack.iter().position(|s| *s == id) {
            let mut cycle: Vec<String> = stack[pos..].iter().map(|s| s.to_string()).collect();
            cycle.push(id.to_string());
            return Err(PluginError::Cycle(cycle));
        }
        stack.push(id);
        let mut deps: Vec<&str> = map[id].depends.iter().map(String::as_str).collect();
        deps.sort();
        for d in deps {
            visit(d, map, stack, done, out)?;
        }
        stack.pop();
        done.insert(id);
        out.push(id.to_string());
        Ok(())
    }
    let ids: Vec<&str> = map.keys().copied().collect();
    for id in ids {
        visit(id, &map, &mut Vec::new(), &mut done, &mut out)?;
    }
    Ok(out)
}

/// Merged translations of all plugins (later plugins override earlier ones).
pub fn translations(plugins: &[Plugin]) -> Catalog {
    let mut c = Catalog::default();
    for p in plugins {
        c.merge(&p.locales);
    }
    c
}

/// Flattened trait library; later plugins override earlier ones by id.
pub fn trait_library(plugins: &[Plugin]) -> Vec<(String, TraitDef)> {
    let mut lib: BTreeMap<String, (String, TraitDef)> = BTreeMap::new();
    for p in plugins {
        for t in &p.traits {
            lib.insert(t.id.clone(), (p.manifest.id.clone(), t.clone()));
        }
    }
    lib.into_values().collect()
}

fn flatten<T: Clone>(
    plugins: &[Plugin],
    items: impl Fn(&Plugin) -> &[T],
    id: impl Fn(&T) -> &str,
) -> Vec<(String, T)> {
    let mut lib: BTreeMap<String, (String, T)> = BTreeMap::new();
    for p in plugins {
        for t in items(p) {
            lib.insert(id(t).to_string(), (p.manifest.id.clone(), t.clone()));
        }
    }
    lib.into_values().collect()
}

/// Flattened skill library; later plugins override earlier ones by id.
pub fn skill_library(plugins: &[Plugin]) -> Vec<(String, SkillDef)> {
    flatten(plugins, |p| &p.skills, |s| &s.id)
}

/// Flattened technique library; later plugins override earlier ones by id.
pub fn technique_library(plugins: &[Plugin]) -> Vec<(String, TechniqueDef)> {
    flatten(plugins, |p| &p.techniques, |s| &s.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(id: &str, deps: &[&str]) -> Manifest {
        Manifest {
            id: id.into(),
            name: id.into(),
            version: "1".into(),
            depends: deps.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn topo_order() {
        let order = resolve_order(&[
            m("c", &["b"]),
            m("a", &[]),
            m("b", &["a"]),
            m("d", &["a", "c"]),
        ])
        .unwrap();
        assert_eq!(order, vec!["a", "b", "c", "d"]);
        let pos = |x: &str| order.iter().position(|o| o == x).unwrap();
        assert!(pos("a") < pos("b") && pos("b") < pos("c") && pos("c") < pos("d"));
    }

    #[test]
    fn missing_dependency() {
        let e = resolve_order(&[m("a", &["zzz"])]).unwrap_err();
        assert!(
            matches!(e, PluginError::MissingDependency { ref missing, .. } if missing == "zzz")
        );
    }

    #[test]
    fn cycle_detected() {
        let e = resolve_order(&[m("a", &["b"]), m("b", &["c"]), m("c", &["a"])]).unwrap_err();
        match e {
            PluginError::Cycle(path) => assert_eq!(path, vec!["a", "b", "c", "a"]),
            other => panic!("unexpected {other}"),
        }
    }

    #[test]
    fn self_cycle_and_duplicate() {
        assert!(matches!(
            resolve_order(&[m("a", &["a"])]),
            Err(PluginError::Cycle(_))
        ));
        assert!(matches!(
            resolve_order(&[m("a", &[]), m("a", &[])]),
            Err(PluginError::Duplicate(_))
        ));
    }

    #[test]
    fn load_sample_plugin() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins");
        let plugins = load_all(&root).unwrap();
        let sample = plugins
            .iter()
            .find(|p| p.manifest.id == "basic-set-sample")
            .expect("sample plugin");
        assert!(sample.traits.iter().any(|t| t.id == "lifting-st"));
        let lib = trait_library(&plugins);
        let inc = lib
            .iter()
            .find(|(_, t)| t.id == "increased-st-example")
            .unwrap();
        assert_eq!(inc.1.cost_per_level, 10);
        assert!(!inc.1.features.is_empty());
    }

    #[test]
    fn load_from_temp_dir() {
        let dir = std::env::temp_dir().join(format!("gurps-plugin-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (id, deps) in [("base", ""), ("ext", "\"base\"")] {
            let p = dir.join(id);
            std::fs::create_dir_all(p.join("data")).unwrap();
            std::fs::write(
                p.join("plugin.toml"),
                format!(
                    "id = \"{id}\"\nname = \"{id}\"\nversion = \"0.1.0\"\ndepends = [{deps}]\n"
                ),
            )
            .unwrap();
            std::fs::write(
                p.join("data/t.toml"),
                format!("[[traits]]\nid = \"t-{id}\"\nname = \"T {id}\"\ncost = 5\n"),
            )
            .unwrap();
        }
        let plugins = load_all(&dir).unwrap();
        let ids: Vec<_> = plugins.iter().map(|p| p.manifest.id.as_str()).collect();
        assert_eq!(ids, vec!["base", "ext"]);
        assert_eq!(plugins[1].traits[0].cost, 5);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn description_plain_and_localized() {
        let src = r#"
[[traits]]
id = "plain"
name = "Plain"
cost = 5
description = """
Line one.

* item
"""

[[traits]]
id = "loc"
name = "Localized"
cost = 5
[traits.description]
ru = '''Русский текст.'''
en = '''English text.'''

[[traits]]
id = "ru-only"
name = "Ru only"
cost = 1
description = { ru = "Только по-русски" }

[[traits]]
id = "none"
name = "None"
cost = 1

[[skills]]
id = "sk"
name = "Skill"
description = "skill text"
[[spells]]
id = "sp"
name = "Spell"
description = { en = "spell text" }
[[equipment]]
id = "eq"
name = "Rope"
[[weapons]]
id = "w"
name = "Sword"
description = "w"
[[modifiers]]
id = "m"
name = "Mod"
description = "m"
"#;
        let df: DataFile = toml::from_str(src).unwrap();
        let t = |id: &str| df.traits.iter().find(|t| t.id == id).unwrap();
        assert_eq!(t("plain").explanation("ru"), Some("Line one.\n\n* item\n"));
        assert_eq!(t("loc").explanation("ru"), Some("Русский текст."));
        assert_eq!(t("loc").explanation("en"), Some("English text."));
        assert_eq!(t("loc").explanation("de"), Some("English text."));
        assert_eq!(t("ru-only").explanation("en"), Some("Только по-русски"));
        assert_eq!(t("none").explanation("ru"), None);
        assert_eq!(df.skills[0].explanation("ru"), Some("skill text"));
        assert_eq!(df.spells[0].explanation("ru"), Some("spell text"));
        assert_eq!(df.equipment[0].explanation("ru"), None);
        assert_eq!(df.weapons.len() + df.modifiers.len(), 2);
    }

    #[test]
    fn sample_plugin_has_explanations() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins");
        let lib = trait_library(&load_all(&root).unwrap());
        let cr = &lib
            .iter()
            .find(|(_, t)| t.id == "combat-reflexes")
            .unwrap()
            .1;
        assert!(
            cr.explanation("en").unwrap().contains("\n\n"),
            "two paragraphs"
        );
        // Russian comes from the standalone language pack (Fluent).
        let plugins = load_all(&root).unwrap();
        let pack = plugins.iter().find(|p| p.manifest.id == "lang-ru").unwrap();
        assert!(pack.traits.is_empty());
        let cat = translations(&plugins);
        assert_eq!(cat.languages(), vec!["ru"]);
        assert_eq!(
            cat.name("ru", "trait", "combat-reflexes"),
            Some("Боевые рефлексы")
        );
        assert!(cat
            .description("ru", "trait", "combat-reflexes")
            .unwrap()
            .starts_with("Персонаж реагирует"));
        // every sample trait has a Russian name; keys are ids, not English text
        for (_, t) in &lib {
            assert!(cat.name("ru", "trait", &t.id).is_some(), "{}", t.id);
            let ru = cat
                .description("ru", "trait", &t.id)
                .unwrap_or_else(|| panic!("ru {}", t.id));
            let en = t.explanation("en").unwrap_or_else(|| panic!("en {}", t.id));
            assert!(ru.len() > 200 && en.len() > 200, "{} too short", t.id);
            assert!(
                ru.contains("\n\n") && en.contains("\n\n"),
                "{} paragraphs",
                t.id
            );
        }
    }
}
