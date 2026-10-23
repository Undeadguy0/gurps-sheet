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
    pub name: String,
    pub category: TraitCategory,
    pub levels: u32,
    pub leveled: bool,
    pub min_level: u32,
    pub cost: i32,
    pub page: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryEntry {
    pub plugin: String,
    #[serde(flatten)]
    pub def: TraitDef,
}

#[derive(Debug, Clone, Serialize)]
pub struct SheetView {
    pub sheet: Sheet,
    pub traits: Vec<TraitView>,
    pub issues: Vec<Issue>,
    pub path: Option<String>,
    pub budget: i32,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub character: Character,
    pub library: Vec<LibraryEntry>,
    pub path: Option<PathBuf>,
    pub plugin_errors: Vec<String>,
}

impl AppState {
    pub fn with_plugins(dir: Option<PathBuf>) -> Self {
        let mut s = Self::default();
        if let Some(dir) = dir {
            match gurps_plugins::load_all(&dir) {
                Ok(plugins) => {
                    s.library = gurps_plugins::trait_library(&plugins)
                        .into_iter()
                        .map(|(plugin, def)| LibraryEntry { plugin, def })
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
                    name: t.display_name(),
                    category: t.def.category,
                    levels: t.levels,
                    leveled: t.def.is_leveled(),
                    min_level: t.def.min_level(),
                    cost: t.cost(),
                    page: t.def.page.clone(),
                    source: t.source.clone(),
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
        let mut t = Trait::new(entry.def.clone(), lv);
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
}
