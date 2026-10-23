//! Localization of plugin data with Fluent (`.ftl`) files.
//!
//! Any plugin may ship `locales/<lang>/*.ftl`; a plugin can also consist of
//! locales only (a *language pack*). Keys are stable ids, never English text:
//!
//! ```ftl
//! trait-combat-reflexes = Боевые рефлексы
//!     .description =
//!         Быстрая реакция в бою.
//!         - реже теряется от неожиданности
//! ```
//!
//! Message id = `<kind>-<entry id>` where kind is one of [`KINDS`]; the message
//! value is the translated name, the `.description` attribute the Markdown
//! explanation. Only text and string-literal placeables are used (no variables);
//! `{ "{" }` escapes a brace. Later plugins (load order) override earlier ones.

use fluent_syntax::ast::{Entry, Expression, InlineExpression, Pattern, PatternElement};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Entry kinds usable as key prefixes.
pub const KINDS: [&str; 6] = ["trait", "skill", "spell", "equipment", "weapon", "modifier"];

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocEntry {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// `lang -> message id -> entry`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalog {
    pub langs: BTreeMap<String, BTreeMap<String, LocEntry>>,
}

#[derive(Debug, thiserror::Error)]
#[error("fluent parse error in {path}: {msg}")]
pub struct FtlError {
    pub path: String,
    pub msg: String,
}

fn pattern_text(p: &Pattern<&str>) -> String {
    let mut s = String::new();
    for el in &p.elements {
        match el {
            PatternElement::TextElement { value } => s.push_str(value),
            PatternElement::Placeable { expression } => match expression {
                Expression::Inline(InlineExpression::StringLiteral { value }) => {
                    s.push_str(&unescape(value))
                }
                Expression::Inline(InlineExpression::NumberLiteral { value }) => s.push_str(value),
                _ => {} // variables/selectors are not used for static data
            },
        }
    }
    s.trim_end().to_string()
}

fn unescape(v: &str) -> String {
    v.replace("\\\"", "\"").replace("\\\\", "\\")
}

/// Parses one `.ftl` source into `id -> entry`.
pub fn parse_ftl(src: &str, path: &str) -> Result<BTreeMap<String, LocEntry>, FtlError> {
    let res = fluent_syntax::parser::parse(src).map_err(|(_, errs)| FtlError {
        path: path.into(),
        msg: errs
            .iter()
            .map(|e| format!("{:?} at {:?}", e.kind, e.pos))
            .collect::<Vec<_>>()
            .join("; "),
    })?;
    let mut out = BTreeMap::new();
    for e in &res.body {
        if let Entry::Message(m) = e {
            let name = m.value.as_ref().map(pattern_text).filter(|t| !t.is_empty());
            let description = m
                .attributes
                .iter()
                .find(|a| a.id.name == "description")
                .map(|a| pattern_text(&a.value))
                .filter(|t| !t.trim().is_empty());
            out.insert(m.id.name.to_string(), LocEntry { name, description });
        }
    }
    Ok(out)
}

/// Loads `dir/locales/<lang>/*.ftl` (sorted) into a catalog.
pub fn load_dir(dir: &Path) -> Result<Catalog, crate::PluginError> {
    let mut cat = Catalog::default();
    let root = dir.join("locales");
    if !root.is_dir() {
        return Ok(cat);
    }
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| crate::PluginError::Io { path, source }
    };
    let mut langs: Vec<_> = std::fs::read_dir(&root)
        .map_err(io(&root))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    langs.sort();
    for ld in langs {
        let lang = ld.file_name().unwrap().to_string_lossy().to_string();
        let mut files: Vec<_> = std::fs::read_dir(&ld)
            .map_err(io(&ld))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "ftl"))
            .collect();
        files.sort();
        let map = cat.langs.entry(lang).or_default();
        for f in files {
            let src = std::fs::read_to_string(&f).map_err(io(&f))?;
            let parsed = parse_ftl(&src, &f.display().to_string())
                .map_err(|e| crate::PluginError::Locale(e.to_string()))?;
            map.extend(parsed);
        }
    }
    Ok(cat)
}

impl Catalog {
    /// Merges `other` on top of `self` (field-wise: a pack may translate only names).
    pub fn merge(&mut self, other: &Catalog) {
        for (lang, m) in &other.langs {
            let dst = self.langs.entry(lang.clone()).or_default();
            for (k, v) in m {
                let e = dst.entry(k.clone()).or_default();
                if v.name.is_some() {
                    e.name = v.name.clone();
                }
                if v.description.is_some() {
                    e.description = v.description.clone();
                }
            }
        }
    }

    pub fn languages(&self) -> Vec<String> {
        self.langs.keys().cloned().collect()
    }

    /// Entry for `kind`/`id` in `locale` (exact, then language part `ru-RU` -> `ru`).
    pub fn get(&self, locale: &str, kind: &str, id: &str) -> Option<&LocEntry> {
        let key = format!("{kind}-{id}");
        let lang = locale.split(['-', '_']).next().unwrap_or(locale);
        [locale, lang]
            .iter()
            .find_map(|l| self.langs.get(*l).and_then(|m| m.get(&key)))
    }

    pub fn name(&self, locale: &str, kind: &str, id: &str) -> Option<&str> {
        self.get(locale, kind, id).and_then(|e| e.name.as_deref())
    }

    pub fn description(&self, locale: &str, kind: &str, id: &str) -> Option<&str> {
        self.get(locale, kind, id)
            .and_then(|e| e.description.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"
# comment
trait-combat-reflexes = Боевые рефлексы
    .description =
        Быстрая реакция.

        - пункт **один**
        - скобка { "{" } и звезда { "*" }
skill-sword = Меч
trait-only-desc =
    .description = Только пояснение
"#;

    #[test]
    fn parses_names_and_multiline_descriptions() {
        let m = parse_ftl(SRC, "t.ftl").unwrap();
        let cr = &m["trait-combat-reflexes"];
        assert_eq!(cr.name.as_deref(), Some("Боевые рефлексы"));
        assert_eq!(
            cr.description.as_deref(),
            Some("Быстрая реакция.\n\n- пункт **один**\n- скобка { и звезда *")
        );
        assert_eq!(m["skill-sword"].description, None);
        assert_eq!(m["trait-only-desc"].name, None);
    }

    #[test]
    fn rejects_broken_ftl() {
        assert!(parse_ftl("trait-x = ok\n  * bad = {", "bad.ftl").is_err());
    }

    #[test]
    fn merge_and_lookup() {
        let mut a = Catalog::default();
        a.langs
            .entry("ru".into())
            .or_default()
            .extend(parse_ftl(SRC, "a").unwrap());
        let mut b = Catalog::default();
        b.langs
            .entry("ru".into())
            .or_default()
            .extend(parse_ftl("trait-combat-reflexes = Рефлексы\n", "b").unwrap());
        a.merge(&b);
        assert_eq!(
            a.name("ru-RU", "trait", "combat-reflexes"),
            Some("Рефлексы")
        );
        assert!(a
            .description("ru", "trait", "combat-reflexes")
            .unwrap()
            .starts_with("Быстрая"));
        assert_eq!(a.name("en", "trait", "combat-reflexes"), None);
        assert_eq!(a.name("ru", "skill", "sword"), Some("Меч"));
    }
}
