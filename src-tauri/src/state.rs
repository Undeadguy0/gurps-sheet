//! UI-independent application state; all Tauri commands delegate here so the
//! logic can be unit-tested without a webview.

use gurps_core::{Character, Stat, Trait, TraitCategory, TraitDef};
use gurps_rules::dice::{success_roll, RollResult, SplitMix};
use gurps_rules::validate::{validate, Issue};
use gurps_rules::Sheet;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct TraitView {
    pub index: usize,
    pub id: String,
    /// Display name in the UI language (with level), falls back to the original.
    pub name: String,
    /// Original (plugin) display name when `name` is a translation.
    pub original_name: Option<String>,
    pub category: TraitCategory,
    pub levels: u32,
    pub leveled: bool,
    pub min_level: u32,
    pub cost: i32,
    pub page: Option<String>,
    pub source: Option<String>,
    /// Effective explanation (user text, else plugin text), Markdown.
    pub explanation: Option<String>,
    /// Plugin text for the current locale (for "reset to default").
    pub default_explanation: Option<String>,
    /// True when the user override is in effect.
    pub custom_explanation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryEntry {
    pub plugin: String,
    #[serde(flatten)]
    pub def: TraitDef,
    /// Plugin explanation resolved for the UI locale.
    pub explanation: Option<String>,
    /// Translated name (None when showing originals or no translation exists).
    pub translated_name: Option<String>,
}

/// Language settings for the UI switcher.
#[derive(Debug, Clone, Serialize)]
pub struct LocaleInfo {
    /// Languages with translations, plus "en" for the original data.
    pub available: Vec<String>,
    pub locale: String,
    pub original_only: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SheetView {
    pub sheet: Sheet,
    pub traits: Vec<TraitView>,
    pub issues: Vec<Issue>,
    pub path: Option<String>,
    pub budget: i32,
}

#[derive(Debug)]
pub struct AppState {
    pub character: Character,
    pub library: Vec<LibraryEntry>,
    pub path: Option<PathBuf>,
    pub plugin_errors: Vec<String>,
    /// UI locale used to resolve localized plugin text (default "ru").
    pub locale: String,
    /// Show plugin originals only (no translations).
    pub original_only: bool,
    /// Merged Fluent translations of all plugins.
    pub catalog: gurps_plugins::Catalog,
}

pub const DEFAULT_LOCALE: &str = "ru";

impl Default for AppState {
    fn default() -> Self {
        Self {
            character: Character::default(),
            library: Vec::new(),
            path: None,
            plugin_errors: Vec::new(),
            locale: DEFAULT_LOCALE.into(),
            original_only: false,
            catalog: Default::default(),
        }
    }
}

impl AppState {
    pub fn with_plugins(dir: Option<PathBuf>) -> Self {
        let mut s = Self::default();
        if let Some(dir) = dir {
            match gurps_plugins::load_all(&dir) {
                Ok(plugins) => {
                    s.catalog = gurps_plugins::translations(&plugins);
                    s.library = gurps_plugins::trait_library(&plugins)
                        .into_iter()
                        .map(|(plugin, def)| LibraryEntry {
                            plugin,
                            explanation: None,
                            translated_name: None,
                            def,
                        })
                        .collect();
                }
                Err(e) => s.plugin_errors.push(e.to_string()),
            }
        }
        s
    }

    pub fn view(&self) -> SheetView {
        SheetView {
            sheet: gurps_rules::compute_sheet(&self.character),
            traits: self
                .character
                .traits
                .iter()
                .enumerate()
                .map(|(index, t)| TraitView {
                    index,
                    id: t.def.id.clone(),
                    name: self
                        .translated(&t.def.id)
                        .map(|n| {
                            if t.def.is_leveled() {
                                format!("{n} {}", t.levels)
                            } else {
                                n.to_owned()
                            }
                        })
                        .unwrap_or_else(|| t.display_name()),
                    original_name: self.translated(&t.def.id).map(|_| t.display_name()),
                    category: t.def.category,
                    levels: t.levels,
                    leveled: t.def.is_leveled(),
                    min_level: t.def.min_level(),
                    cost: t.cost(),
                    page: t.def.page.clone(),
                    source: t.source.clone(),
                    explanation: gurps_core::non_blank(t.explanation.clone())
                        .or_else(|| self.plugin_explanation(t)),
                    default_explanation: self.plugin_explanation(t),
                    custom_explanation: t.has_custom_explanation(),
                })
                .collect(),
            issues: validate(&self.character),
            path: self.path.as_ref().map(|p| p.display().to_string()),
            budget: self.character.points_budget,
        }
    }

    pub fn new_character(&mut self, name: Option<String>) -> SheetView {
        self.character = Character::new(name.unwrap_or_else(|| "New character".into()));
        self.path = None;
        self.view()
    }

    pub fn load(&mut self, path: &str) -> Result<SheetView, String> {
        self.character = gurps_io::load(path).map_err(|e| e.to_string())?;
        self.path = Some(path.into());
        Ok(self.view())
    }

    pub fn save(&mut self, path: Option<String>) -> Result<String, String> {
        let p = path
            .map(PathBuf::from)
            .or_else(|| self.path.clone())
            .ok_or("no file path given")?;
        gurps_io::save(&p, &self.character).map_err(|e| e.to_string())?;
        self.path = Some(p.clone());
        Ok(p.display().to_string())
    }

    pub fn set_attribute(&mut self, key: &str, level: i32) -> Result<SheetView, String> {
        let stat = Stat::parse(key).ok_or_else(|| format!("unknown attribute {key}"))?;
        if !self.character.attributes.set_purchased(stat, level) {
            return Err(format!("{key} cannot be bought directly"));
        }
        Ok(self.view())
    }

    pub fn set_name(&mut self, name: String) -> SheetView {
        self.character.name = name;
        self.view()
    }

    pub fn add_trait(&mut self, id: &str, levels: Option<u32>) -> Result<SheetView, String> {
        let entry = self
            .library
            .iter()
            .find(|e| e.def.id == id)
            .ok_or_else(|| format!("trait {id} not in library"))?;
        let lv = levels.unwrap_or(if entry.def.is_leveled() { 1 } else { 0 });
        // Plugin text stays in the library; the file stores only the user's override.
        let mut def = entry.def.clone();
        def.description = None;
        let mut t = Trait::new(def, lv);
        t.source = Some(entry.plugin.clone());
        self.character.traits.push(t);
        Ok(self.view())
    }

    pub fn set_trait_levels(&mut self, index: usize, levels: u32) -> Result<SheetView, String> {
        let t = self
            .character
            .traits
            .get_mut(index)
            .ok_or("no such trait")?;
        t.levels = levels;
        Ok(self.view())
    }

    pub fn remove_trait(&mut self, index: usize) -> Result<SheetView, String> {
        if index >= self.character.traits.len() {
            return Err("no such trait".into());
        }
        self.character.traits.remove(index);
        Ok(self.view())
    }

    /// Locale used for the plugin's own (TOML) text.
    fn data_locale(&self) -> &str {
        if self.original_only {
            "en"
        } else {
            &self.locale
        }
    }

    /// Translated trait name for the current language, unless showing originals.
    fn translated(&self, id: &str) -> Option<&str> {
        if self.original_only {
            return None;
        }
        self.catalog.name(&self.locale, "trait", id)
    }

    /// Plugin explanation for a trait id: Fluent translation, else the plugin's
    /// own text (current library entry, else the snapshot `fallback`).
    fn explanation_for(&self, id: &str, fallback: Option<&TraitDef>) -> Option<String> {
        let tr = (!self.original_only)
            .then(|| self.catalog.description(&self.locale, "trait", id))
            .flatten();
        tr.or_else(|| {
            self.library
                .iter()
                .find(|e| e.def.id == id)
                .map(|e| &e.def)
                .or(fallback)
                .and_then(|d| d.explanation(self.data_locale()))
        })
        .map(str::to_owned)
    }

    fn plugin_explanation(&self, t: &Trait) -> Option<String> {
        self.explanation_for(&t.def.id, Some(&t.def))
    }

    /// Library with names/explanations resolved for the current language.
    pub fn library_view(&self) -> Vec<LibraryEntry> {
        self.library
            .iter()
            .map(|e| LibraryEntry {
                explanation: self.explanation_for(&e.def.id, None),
                translated_name: self.translated(&e.def.id).map(str::to_owned),
                ..e.clone()
            })
            .collect()
    }

    pub fn locale_info(&self) -> LocaleInfo {
        let mut available = self.catalog.languages();
        if !available.iter().any(|l| l == "en") {
            available.push("en".into());
        }
        available.sort();
        LocaleInfo {
            available,
            locale: self.locale.clone(),
            original_only: self.original_only,
        }
    }

    pub fn set_locale(&mut self, locale: String, original_only: bool) -> SheetView {
        if !locale.trim().is_empty() {
            self.locale = locale;
        }
        self.original_only = original_only;
        self.view()
    }

    /// Sets (or with `None`/blank text, resets to plugin default) a trait's explanation.
    pub fn set_trait_explanation(
        &mut self,
        index: usize,
        text: Option<String>,
    ) -> Result<SheetView, String> {
        let t = self
            .character
            .traits
            .get_mut(index)
            .ok_or("no such trait")?;
        t.explanation = gurps_core::non_blank(text);
        Ok(self.view())
    }

    pub fn roll(target: i32, modifier: i32) -> RollResult {
        success_roll(target, modifier, &mut SplitMix::from_time())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> AppState {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../plugins");
        AppState::with_plugins(Some(dir))
    }

    #[test]
    fn acceptance_flow() {
        let mut s = state();
        assert!(s.plugin_errors.is_empty());
        s.new_character(Some("Hero".into()));
        s.set_attribute("ST", 2).unwrap();
        let v = s.add_trait("increased-st-example", Some(1)).unwrap();
        let st = v.sheet.row(Stat::ST).unwrap();
        assert_eq!(st.total, 13.0);
        assert_eq!(st.value.base, 10.0);
        assert_eq!(st.value.contributions[0].amount, 2.0);
        assert_eq!(st.value.contributions[1].amount, 1.0);
        assert_eq!(v.sheet.points.total, 30);
        let v = s.remove_trait(0).unwrap();
        assert_eq!(v.sheet.points.total, 20);
        assert!(s.remove_trait(5).is_err());
        assert!(s.set_attribute("BasicLift", 1).is_err());
        assert!(s.add_trait("nope", None).is_err());
    }

    #[test]
    fn save_load() {
        let mut s = state();
        s.set_attribute("DX", 1).unwrap();
        let p = std::env::temp_dir().join(format!("app-{}.json", std::process::id()));
        s.save(Some(p.display().to_string())).unwrap();
        let mut s2 = state();
        let v = s2.load(&p.display().to_string()).unwrap();
        assert_eq!(v.sheet.points.attributes, 20);
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn explanation_override_and_reset() {
        let mut s = state();
        let v = s.add_trait("combat-reflexes", None).unwrap();
        let plugin = v.traits[0].explanation.clone().expect("plugin text");
        assert_eq!(v.traits[0].default_explanation.as_ref(), Some(&plugin));
        assert!(!v.traits[0].custom_explanation);
        assert!(s.library_view().iter().any(|e| e.explanation.is_some()));

        let v = s
            .set_trait_explanation(0, Some("**Моё** пояснение".into()))
            .unwrap();
        assert_eq!(
            v.traits[0].explanation.as_deref(),
            Some("**Моё** пояснение")
        );
        assert!(v.traits[0].custom_explanation);

        let p = std::env::temp_dir().join(format!("app-expl-{}.json", std::process::id()));
        s.save(Some(p.display().to_string())).unwrap();
        let mut s2 = state();
        let v = s2.load(&p.display().to_string()).unwrap();
        assert_eq!(
            v.traits[0].explanation.as_deref(),
            Some("**Моё** пояснение")
        );
        std::fs::remove_file(p).unwrap();

        let v = s2.set_trait_explanation(0, Some("  ".into())).unwrap();
        assert_eq!(v.traits[0].explanation, Some(plugin));
        assert!(!v.traits[0].custom_explanation);
        assert!(s2.set_trait_explanation(9, None).is_err());
    }

    #[test]
    fn localization_switch() {
        let mut s = state();
        assert!(s.locale_info().available.contains(&"ru".to_string()));
        let v = s.add_trait("combat-reflexes", None).unwrap();
        assert_eq!(v.traits[0].name, "Боевые рефлексы");
        assert_eq!(
            v.traits[0].original_name.as_deref(),
            Some("Combat Reflexes")
        );
        assert!(v.traits[0]
            .explanation
            .as_deref()
            .unwrap()
            .starts_with("Быстрая"));
        let lib = s.library_view();
        let e = lib.iter().find(|e| e.def.id == "combat-reflexes").unwrap();
        assert_eq!(e.translated_name.as_deref(), Some("Боевые рефлексы"));
        assert_eq!(e.def.name, "Combat Reflexes");

        let v = s.set_locale("ru".into(), true);
        assert_eq!(v.traits[0].name, "Combat Reflexes");
        assert_eq!(v.traits[0].original_name, None);
        assert!(v.traits[0]
            .explanation
            .as_deref()
            .unwrap()
            .starts_with("Fast"));
        assert!(s.library_view().iter().all(|e| e.translated_name.is_none()));

        let v = s.set_locale("en".into(), false);
        assert_eq!(v.traits[0].name, "Combat Reflexes");
        s.set_locale("ru".into(), false);
        // leveled traits keep their level after the translated name
        let v2 = s.add_trait("lifting-st", Some(2)).unwrap();
        assert_eq!(v2.traits[1].name, "Подъёмная сила 2");
        // user override wins over the translation
        let v = s.set_trait_explanation(0, Some("своё".into())).unwrap();
        assert_eq!(v.traits[0].explanation.as_deref(), Some("своё"));
    }
}
