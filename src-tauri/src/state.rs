//! UI-independent application state; all Tauri commands delegate here so the
//! logic can be unit-tested without a webview.

use gurps_core::{
    Character, Equipment, EquipmentDef, LocalizedText, ModifierDef, Skill, SkillDef, SkillDefault,
    Spell, SpellDef, Stat, Technique, TechniqueDef, Trait, TraitCategory, TraitDef, TraitModifier,
    Weapon, WeaponDef,
};
use gurps_dice::{success_roll, RollResult, SplitMix};
use gurps_library::calc::{Calc, Issue};
use gurps_library::sheet::Sheet;
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
    /// Final cost (with modifiers).
    pub cost: i32,
    /// Cost before modifiers.
    pub base_cost: i32,
    /// Sum of modifier percentages (before the -80% floor).
    pub modifier_percent: i32,
    pub modifiers: Vec<ModifierView>,
    /// How the cost was obtained (empty without modifiers).
    pub cost_explain: Vec<String>,
    pub page: Option<String>,
    pub source: Option<String>,
    /// Effective explanation (user text, else plugin text), Markdown.
    pub explanation: Option<String>,
    /// Plugin text for the current locale (for "reset to default").
    pub default_explanation: Option<String>,
    /// True when the user override is in effect.
    pub custom_explanation: bool,
}

/// An enhancement/limitation on a character's trait.
#[derive(Debug, Clone, Serialize)]
pub struct ModifierView {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub original_name: Option<String>,
    pub cost_percent: i32,
    pub page: Option<String>,
    pub source: Option<String>,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModifierLibEntry {
    pub plugin: String,
    #[serde(flatten)]
    pub def: ModifierDef,
    pub explanation: Option<String>,
    pub translated_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpellView {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub original_name: Option<String>,
    pub college: Vec<String>,
    pub attribute: String,
    pub difficulty: String,
    pub points: i32,
    pub level: Option<i32>,
    pub relative: Option<String>,
    pub bonus: i32,
    pub prerequisites: Option<String>,
    pub explain: Vec<String>,
    pub page: Option<String>,
    pub source: Option<String>,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpellLibEntry {
    pub plugin: String,
    #[serde(flatten)]
    pub def: SpellDef,
    pub explanation: Option<String>,
    pub translated_name: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
pub struct SkillView {
    pub index: usize,
    pub id: String,
    /// Name in the UI language with specialization.
    pub name: String,
    pub original_name: Option<String>,
    pub attribute: String,
    pub difficulty: String,
    pub points: i32,
    pub level: Option<i32>,
    pub relative: Option<String>,
    pub from_default: bool,
    pub bonus: i32,
    pub explain: Vec<String>,
    /// Defaults as text, e.g. "DX-5", "Shortsword-2".
    pub defaults: Vec<String>,
    pub specialization: Option<String>,
    pub specialization_required: bool,
    pub specializations: Vec<String>,
    pub page: Option<String>,
    pub source: Option<String>,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TechniqueView {
    pub index: usize,
    pub id: String,
    /// E.g. "Пинок (Драка)".
    pub name: String,
    pub original_name: Option<String>,
    pub skill: String,
    pub difficulty: String,
    pub modifier: i32,
    pub max: i32,
    pub points: i32,
    pub level: Option<i32>,
    pub relative: Option<String>,
    pub max_level: Option<i32>,
    pub explain: Vec<String>,
    pub page: Option<String>,
    pub source: Option<String>,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillLibEntry {
    pub plugin: String,
    #[serde(flatten)]
    pub def: SkillDef,
    pub explanation: Option<String>,
    pub translated_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TechniqueLibEntry {
    pub plugin: String,
    #[serde(flatten)]
    pub def: TechniqueDef,
    pub explanation: Option<String>,
    pub translated_name: Option<String>,
    /// Parent skill name in the UI language.
    pub skill_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillLibrary {
    pub skills: Vec<SkillLibEntry>,
    pub techniques: Vec<TechniqueLibEntry>,
}

/// One equipment row, flattened from the container tree (`path` addresses it).
#[derive(Debug, Clone, Serialize)]
pub struct EquipmentView {
    pub path: Vec<usize>,
    /// Nesting depth (0 = top level).
    pub depth: usize,
    pub id: String,
    pub name: String,
    pub original_name: Option<String>,
    pub quantity: u32,
    pub weight: f64,
    pub cost: f64,
    /// quantity × weight + contents.
    pub total_weight: f64,
    pub total_cost: f64,
    pub carried: bool,
    /// False when an enclosing container is not carried.
    pub effectively_carried: bool,
    pub container: bool,
    pub stats: Option<String>,
    pub page: Option<String>,
    pub source: Option<String>,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquipmentLibEntry {
    pub plugin: String,
    #[serde(flatten)]
    pub def: EquipmentDef,
    pub explanation: Option<String>,
    pub translated_name: Option<String>,
}

/// One attack row; `skill` is translated, `original_skill` keeps the plugin name.
#[derive(Debug, Clone, Serialize)]
pub struct AttackView {
    #[serde(flatten)]
    pub row: gurps_library::sheet::AttackRow,
    pub original_skill: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeaponView {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub original_name: Option<String>,
    pub equipment: Option<String>,
    pub available: bool,
    pub ready: bool,
    pub attacks: Vec<AttackView>,
    pub page: Option<String>,
    pub source: Option<String>,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeaponLibEntry {
    pub plugin: String,
    #[serde(flatten)]
    pub def: WeaponDef,
    pub explanation: Option<String>,
    pub translated_name: Option<String>,
    /// Linked equipment name in the UI language.
    pub equipment_name: Option<String>,
    pub skill_name: String,
}

/// Where library data came from (shown in the UI; helps spot stale data).
#[derive(Debug, Clone, Serialize)]
pub struct PluginStatus {
    pub dir: Option<String>,
    pub plugins: Vec<String>,
    pub errors: Vec<String>,
    pub traits: usize,
    /// Skills + techniques.
    pub skills: usize,
    pub equipment: usize,
    pub weapons: usize,
    pub spells: usize,
    pub modifiers: usize,
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
    pub skills: Vec<SkillView>,
    pub techniques: Vec<TechniqueView>,
    pub spells: Vec<SpellView>,
    pub equipment: Vec<EquipmentView>,
    pub weapons: Vec<WeaponView>,
    pub issues: Vec<Issue>,
    pub path: Option<String>,
    pub budget: i32,
    pub rules: gurps_core::RulesOptions,
    /// Character notes (Markdown source).
    pub notes: String,
    pub pool: PoolView,
    /// User-made definitions stored in the character (for the «Своё» editors).
    pub custom: gurps_core::CustomContent,
    pub session: SessionView,
    pub history: HistoryView,
}

/// Point pool summary + ledger (character cost is live, never stored).
#[derive(Debug, Clone, Serialize)]
pub struct PoolView {
    pub entries: Vec<gurps_core::LedgerEntry>,
    pub starting: i32,
    pub income: i32,
    pub earned: i32,
    pub spent_outside: i32,
    pub available: i32,
    /// Live cost of the sheet.
    pub character_cost: i32,
    /// earned − character_cost − spent_outside (may be negative).
    pub unspent: i32,
    pub disadvantage_limit: i32,
    /// True when the limit was set by hand (else 50% of starting points).
    pub disadvantage_limit_custom: bool,
    /// Disadvantage points taken (positive, quirks excluded).
    pub disadvantages: i32,
    pub quirks: usize,
    pub quirk_points: i32,
}

/// Editable fields of a ledger entry (the id is assigned by the state).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LedgerInput {
    #[serde(default)]
    pub date: String,
    pub amount: i32,
    pub kind: gurps_core::LedgerKind,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub skill: Option<String>,
}

/// Library tag of user-made definitions.
pub const CUSTOM: &str = "custom";

/// Current HP/FP at the table (session state, not build points).
#[derive(Debug, Clone, Serialize)]
pub struct SessionView {
    pub hp: i32,
    pub hp_max: i32,
    pub fp: i32,
    pub fp_max: i32,
}

/// Undo/redo state for the header buttons; labels are Russian action names.
#[derive(Debug, Clone, Serialize)]
pub struct HistoryView {
    pub can_undo: bool,
    pub can_redo: bool,
    pub undo_label: Option<String>,
    pub redo_label: Option<String>,
    pub undo_depth: usize,
    /// Character differs from the last saved/loaded/new state.
    pub dirty: bool,
}

/// Maximum number of undo steps kept.
pub const HISTORY_LIMIT: usize = 200;

#[derive(Debug)]
pub struct AppState {
    pub character: Character,
    pub library: Vec<LibraryEntry>,
    /// (plugin id, definition)
    pub skill_library: Vec<(String, SkillDef)>,
    pub technique_library: Vec<(String, TechniqueDef)>,
    pub equipment_library: Vec<(String, EquipmentDef)>,
    pub weapon_library: Vec<(String, WeaponDef)>,
    pub spell_library: Vec<(String, SpellDef)>,
    pub modifier_library: Vec<(String, ModifierDef)>,
    pub path: Option<PathBuf>,
    /// Where relative file names are resolved (the user's documents folder in the app).
    /// Never the process CWD: under `tauri dev` that is src-tauri/, and writing there
    /// makes the dev watcher rebuild and restart the app.
    pub save_dir: Option<PathBuf>,
    pub plugin_errors: Vec<String>,
    /// Where the catalog comes from (shared SurrealDB; tests use the testkit DB).
    pub source: Option<gurps_library::LibrarySource>,
    /// Loaded catalog packs, in priority order.
    pub plugin_ids: Vec<String>,
    /// UI locale used to resolve localized plugin text (default "ru").
    pub locale: String,
    /// Show plugin originals only (no translations).
    pub original_only: bool,
    /// Catalog translations (non-`en`), `lang -> kind-id`.
    pub catalog: gurps_library::Translations,
    /// Undo stack: (action label, character before the action); newest last.
    pub undo_stack: std::collections::VecDeque<(String, Character)>,
    /// Redo stack: (action label, character after the action).
    pub redo_stack: Vec<(String, Character)>,
    /// Character as last saved/loaded/created (for the dirty flag).
    pub saved: Character,
    pub rng: SplitMix,
    /// Расчётчик gurps-calc (только при каталоге из БД).
    pub engine: Option<std::sync::Arc<gurps_library::calc::Engine>>,
}

pub const DEFAULT_LOCALE: &str = "ru";

impl Default for AppState {
    fn default() -> Self {
        Self {
            character: Character::default(),
            library: Vec::new(),
            skill_library: Vec::new(),
            technique_library: Vec::new(),
            equipment_library: Vec::new(),
            weapon_library: Vec::new(),
            spell_library: Vec::new(),
            modifier_library: Vec::new(),
            path: None,
            save_dir: None,
            plugin_errors: Vec::new(),
            source: None,
            plugin_ids: Vec::new(),
            locale: DEFAULT_LOCALE.into(),
            original_only: false,
            catalog: Default::default(),
            undo_stack: Default::default(),
            redo_stack: Vec::new(),
            saved: Character::default(),
            rng: SplitMix::from_time(),
            engine: None,
        }
    }
}

impl AppState {
    pub fn with_source(source: Option<gurps_library::LibrarySource>) -> Self {
        let mut s = Self {
            source,
            ..Self::default()
        };
        s.reload_plugins();
        s.character.migrate();
        s.saved = s.character.clone();
        s
    }

    /// (Re)reads the catalog from `source`; nothing is cached between runs.
    pub fn reload_plugins(&mut self) {
        self.library.clear();
        self.skill_library.clear();
        self.technique_library.clear();
        self.equipment_library.clear();
        self.weapon_library.clear();
        self.spell_library.clear();
        self.modifier_library.clear();
        self.catalog = Default::default();
        self.plugin_errors.clear();
        self.plugin_ids.clear();
        let Some(src) = self.source.clone() else {
            self.plugin_errors
                .push("no catalog source configured".into());
            return;
        };
        self.engine = match &src {
            gurps_library::LibrarySource::Db(cfg) => {
                match gurps_library::calc::Engine::connect(cfg) {
                    Ok(e) => Some(std::sync::Arc::new(e)),
                    Err(e) => {
                        self.plugin_errors.push(format!("расчёт: {e}"));
                        None
                    }
                }
            }
            #[allow(unreachable_patterns)]
            _ => None,
        };
        match gurps_library::Library::load(&src) {
            Ok(lib) => {
                for w in &lib.warnings {
                    eprintln!("gurps-sheet: catalog: {w}");
                }
                self.plugin_ids = lib.packs;
                self.catalog = lib.translations;
                self.skill_library = lib.skills;
                self.technique_library = lib.techniques;
                self.equipment_library = lib.equipment;
                self.weapon_library = lib.weapons;
                self.spell_library = lib.spells;
                self.modifier_library = lib.modifiers;
                self.library = lib
                    .traits
                    .into_iter()
                    .map(|(plugin, def)| LibraryEntry {
                        plugin,
                        def,
                        explanation: None,
                        translated_name: None,
                    })
                    .collect();
            }
            Err(e) => self.plugin_errors.push(e),
        }
        self.sync_custom();
    }

    pub fn plugin_status(&self) -> PluginStatus {
        PluginStatus {
            dir: self.source.as_ref().map(|s| s.describe()),
            plugins: self.plugin_ids.clone(),
            errors: self.plugin_errors.clone(),
            traits: self.library.len(),
            skills: self.skill_library.len() + self.technique_library.len(),
            equipment: self.equipment_library.len(),
            weapons: self.weapon_library.len(),
            spells: self.spell_library.len(),
            modifiers: self.modifier_library.len(),
        }
    }

    /// Расчёт через БД (`gurps-calc`); лист и проверки строятся из него.
    fn calc(&self) -> Result<Calc, String> {
        match &self.engine {
            None => Err("нет подключения к БД".into()),
            Some(e) => e.compute(&self.character),
        }
    }

    fn sheet_of(&self, c: &Result<Calc, String>) -> Sheet {
        match c {
            Ok(c) => gurps_library::sheet::build(&self.character, c, &|k, id| {
                self.translated_kind(k, id).map(str::to_owned)
            }),
            Err(_) => Sheet {
                name: self.character.name.clone(),
                ..Default::default()
            },
        }
    }

    pub fn view(&self) -> SheetView {
        let calc = self.calc();
        let sheet = self.sheet_of(&calc);
        let issues = self.issues(&calc, &sheet);
        SheetView {
            skills: self.skill_views(&sheet),
            techniques: self.technique_views(&sheet),
            spells: self.spell_views(&sheet),
            rules: self.character.rules.clone(),
            notes: self.character.notes.clone(),
            pool: self.pool_view(&sheet),
            custom: self.character.custom.clone(),
            session: self.session_view(&sheet),
            history: self.history_view(),
            equipment: self.equipment_views(),
            weapons: self.weapon_views(&sheet),
            sheet,
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
                    cost: t.cost_with(self.character.rules.negative_rounding),
                    base_cost: t.base_cost(),
                    modifier_percent: t.modifier_percent(),
                    modifiers: self.modifier_views(t),
                    cost_explain: trait_cost_explain(t, self.character.rules.negative_rounding),
                    page: t.def.page.clone(),
                    source: t.source.clone(),
                    explanation: gurps_core::non_blank(t.explanation.clone())
                        .or_else(|| self.plugin_explanation(t)),
                    default_explanation: self.plugin_explanation(t),
                    custom_explanation: t.has_custom_explanation(),
                })
                .collect(),
            issues,
            path: self.path.as_ref().map(|p| p.display().to_string()),
            budget: self.character.available_points(),
        }
    }

    pub fn new_character(&mut self, name: Option<String>) -> SheetView {
        self.character = Character::new(name.unwrap_or_else(|| "New character".into()));
        // everyone can punch and kick
        for (plugin, d) in &self.weapon_library {
            if d.equipment.is_none() {
                self.character
                    .weapons
                    .push(Self::weapon_snapshot(plugin, d));
            }
        }
        self.path = None;
        self.reset_history();
        self.view()
    }

    fn reset_history(&mut self) {
        self.character.migrate();
        self.sync_custom();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved = self.character.clone();
    }

    /// Runs one edit and records it for undo when the character changed.
    /// On error the character is restored, so failed edits never leave half-changes.
    pub fn record<T>(
        &mut self,
        label: impl Into<String>,
        f: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        let before = self.character.clone();
        match f(self) {
            Ok(v) => {
                if self.character != before {
                    let action: String = label.into();
                    let label = match self.changed_item(&before) {
                        Some(n) => format!("{action} «{n}»"),
                        None => action,
                    };
                    self.undo_stack.push_back((label, before));
                    while self.undo_stack.len() > HISTORY_LIMIT {
                        self.undo_stack.pop_front();
                    }
                    self.redo_stack.clear();
                }
                Ok(v)
            }
            Err(e) => {
                self.character = before;
                Err(e)
            }
        }
    }

    /// Name (UI language) of the first list item that differs between `before` and the
    /// current character: the added, removed or edited trait/skill/spell/…
    fn changed_item(&self, before: &Character) -> Option<String> {
        fn diff<T: PartialEq>(a: &[T], b: &[T]) -> Option<(bool, usize)> {
            let i = a.iter().zip(b).position(|(x, y)| x != y);
            match (i, a.len().cmp(&b.len())) {
                (Some(i), std::cmp::Ordering::Greater) | (Some(i), std::cmp::Ordering::Equal) => {
                    Some((a.len() > b.len(), i))
                }
                (Some(i), std::cmp::Ordering::Less) => Some((false, i)),
                (None, std::cmp::Ordering::Equal) => None,
                (None, std::cmp::Ordering::Greater) => Some((true, b.len())),
                (None, std::cmp::Ordering::Less) => Some((false, a.len())),
            }
        }
        let now = &self.character;
        macro_rules! check {
            ($field:ident, $kind:expr) => {
                if let Some((from_before, i)) = diff(&before.$field, &now.$field) {
                    let item = if from_before {
                        &before.$field[i]
                    } else {
                        &now.$field[i]
                    };
                    return Some(
                        self.translated_kind($kind, &item.def.id)
                            .map(str::to_owned)
                            .unwrap_or_else(|| item.def.name.clone()),
                    );
                }
            };
        }
        check!(traits, "trait");
        check!(skills, "skill");
        check!(techniques, "technique");
        check!(spells, "spell");
        check!(equipment, "equipment");
        check!(weapons, "weapon");
        None
    }

    pub fn undo(&mut self) -> Result<SheetView, String> {
        let (label, prev) = self.undo_stack.pop_back().ok_or("nothing to undo")?;
        let cur = std::mem::replace(&mut self.character, prev);
        self.redo_stack.push((label, cur));
        self.sync_custom();
        Ok(self.view())
    }

    pub fn redo(&mut self) -> Result<SheetView, String> {
        let (label, next) = self.redo_stack.pop().ok_or("nothing to redo")?;
        let cur = std::mem::replace(&mut self.character, next);
        self.undo_stack.push_back((label, cur));
        self.sync_custom();
        Ok(self.view())
    }

    fn issues(&self, calc: &Result<Calc, String>, sheet: &Sheet) -> Vec<Issue> {
        gurps_library::sheet::issues(&self.character, calc, sheet)
    }

    fn pool_view(&self, sheet: &Sheet) -> PoolView {
        let ch = &self.character;
        let t = ch.ledger_totals();
        let b = &sheet.points;
        PoolView {
            entries: ch.ledger.clone(),
            starting: t.starting,
            income: t.income,
            earned: t.earned,
            spent_outside: t.spent_outside,
            available: t.available,
            character_cost: b.total,
            unspent: t.available - b.total,
            disadvantage_limit: ch.disadvantage_limit(),
            disadvantage_limit_custom: ch.rules.disadvantage_limit.is_some(),
            disadvantages: -b.disadvantages,
            quirks: ch
                .traits
                .iter()
                .filter(|x| x.def.category == TraitCategory::Quirk)
                .count(),
            quirk_points: b.quirks,
        }
    }

    pub fn add_ledger_entry(&mut self, e: LedgerInput) -> Result<SheetView, String> {
        // Migrate only a legacy (non-zero) budget; an emptied ledger stays empty
        // instead of growing a 0-point starting entry.
        if self.character.ledger.is_empty() && self.character.points_budget != 0 {
            self.character.migrate();
        }
        let id = self
            .character
            .ledger
            .iter()
            .map(|x| x.id)
            .max()
            .unwrap_or(0)
            + 1;
        let entry = gurps_core::LedgerEntry {
            id,
            date: e.date,
            amount: e.amount,
            kind: e.kind,
            note: e.note,
            skill: gurps_core::non_blank(e.skill),
        }
        .normalized()?;
        self.character.ledger.push(entry);
        self.character.points_budget = self.character.available_points();
        Ok(self.view())
    }

    pub fn update_ledger_entry(&mut self, id: u32, e: LedgerInput) -> Result<SheetView, String> {
        self.character.migrate();
        let slot = self
            .character
            .ledger
            .iter_mut()
            .find(|x| x.id == id)
            .ok_or("no such ledger entry")?;
        *slot = gurps_core::LedgerEntry {
            id,
            date: e.date,
            amount: e.amount,
            kind: e.kind,
            note: e.note,
            skill: gurps_core::non_blank(e.skill),
        }
        .normalized()?;
        self.character.migrate();
        Ok(self.view())
    }

    pub fn remove_ledger_entry(&mut self, id: u32) -> Result<SheetView, String> {
        let n = self.character.ledger.len();
        self.character.ledger.retain(|x| x.id != id);
        if self.character.ledger.len() == n {
            return Err("no such ledger entry".into());
        }
        if self.character.ledger.is_empty() {
            // Removing everything means "no points", not "back to the legacy budget".
            self.character.points_budget = 0;
        }
        self.character.points_budget = self.character.available_points();
        Ok(self.view())
    }

    /// `None` = default (50% of starting points).
    pub fn set_disadvantage_limit(&mut self, limit: Option<i32>) -> Result<SheetView, String> {
        if limit.is_some_and(|l| l < 0) {
            return Err("limit must not be negative".into());
        }
        self.character.rules.disadvantage_limit = limit;
        Ok(self.view())
    }

    /// Puts the character's custom definitions into the libraries (tag "custom");
    /// they override plugin entries with the same id.
    pub fn sync_custom(&mut self) {
        let c = self.character.custom.clone();
        self.library
            .retain(|e| e.plugin != CUSTOM && !c.traits.iter().any(|d| d.id == e.def.id));
        self.library.extend(c.traits.iter().map(|d| LibraryEntry {
            plugin: CUSTOM.into(),
            def: d.clone(),
            explanation: None,
            translated_name: None,
        }));
        fn merge<D: Clone>(lib: &mut Vec<(String, D)>, custom: &[D], id: impl Fn(&D) -> &str) {
            lib.retain(|(p, d)| p != CUSTOM && !custom.iter().any(|c| id(c) == id(d)));
            lib.extend(custom.iter().map(|d| (CUSTOM.to_string(), d.clone())));
        }
        merge(&mut self.skill_library, &c.skills, |d| &d.id);
        merge(&mut self.spell_library, &c.spells, |d| &d.id);
        merge(&mut self.equipment_library, &c.equipment, |d| &d.id);
    }

    fn new_custom_id(&self, kind: &str) -> String {
        let c = &self.character.custom;
        let used: Vec<&str> = c
            .traits
            .iter()
            .map(|d| d.id.as_str())
            .chain(c.skills.iter().map(|d| d.id.as_str()))
            .chain(c.spells.iter().map(|d| d.id.as_str()))
            .chain(c.equipment.iter().map(|d| d.id.as_str()))
            .collect();
        (1..)
            .map(|n| format!("custom-{kind}-{n}"))
            .find(|id| !used.contains(&id.as_str()))
            .unwrap()
    }

    /// Adds or replaces (by id) a user-made definition. `kind`: trait | skill | spell | equipment.
    /// An empty id creates a new entry. Items already on the sheet get the new definition.
    pub fn save_custom(&mut self, kind: &str, def: serde_json::Value) -> Result<SheetView, String> {
        fn parse<D: serde::de::DeserializeOwned>(v: serde_json::Value) -> Result<D, String> {
            serde_json::from_value(v).map_err(|e| format!("invalid definition: {e}"))
        }
        fn upsert<D: Clone>(list: &mut Vec<D>, d: D, id: impl Fn(&D) -> &str) {
            match list.iter().position(|x| id(x) == id(&d)) {
                Some(i) => list[i] = d,
                None => list.push(d),
            }
        }
        let mut v = def;
        let blank_id = v
            .get("id")
            .and_then(|x| x.as_str())
            .is_none_or(|x| x.trim().is_empty());
        if blank_id {
            v["id"] = self.new_custom_id(kind).into();
        }
        let name_ok = v
            .get("name")
            .and_then(|x| x.as_str())
            .is_some_and(|x| !x.trim().is_empty());
        if !name_ok {
            return Err("a custom entry needs a name".into());
        }
        let ch = &mut self.character;
        match kind {
            "trait" => {
                let d: TraitDef = parse(v)?;
                for t in ch.traits.iter_mut().filter(|t| t.def.id == d.id) {
                    t.def = TraitDef {
                        description: None,
                        ..d.clone()
                    };
                }
                upsert(&mut ch.custom.traits, d, |d| &d.id);
            }
            "skill" => {
                let d: SkillDef = parse(v)?;
                for t in ch.skills.iter_mut().filter(|t| t.def.id == d.id) {
                    t.def = SkillDef {
                        description: None,
                        ..d.clone()
                    };
                }
                upsert(&mut ch.custom.skills, d, |d| &d.id);
            }
            "spell" => {
                let d: SpellDef = parse(v)?;
                for t in ch.spells.iter_mut().filter(|t| t.def.id == d.id) {
                    t.def = SpellDef {
                        description: None,
                        ..d.clone()
                    };
                }
                upsert(&mut ch.custom.spells, d, |d| &d.id);
            }
            "equipment" => {
                let d: EquipmentDef = parse(v)?;
                upsert(&mut ch.custom.equipment, d, |d| &d.id);
            }
            _ => return Err(format!("unknown custom kind '{kind}'")),
        }
        self.sync_custom();
        Ok(self.view())
    }

    pub fn remove_custom(&mut self, kind: &str, id: &str) -> Result<SheetView, String> {
        let c = &mut self.character.custom;
        let n = match kind {
            "trait" => {
                let n = c.traits.len();
                c.traits.retain(|d| d.id != id);
                n - c.traits.len()
            }
            "skill" => {
                let n = c.skills.len();
                c.skills.retain(|d| d.id != id);
                n - c.skills.len()
            }
            "spell" => {
                let n = c.spells.len();
                c.spells.retain(|d| d.id != id);
                n - c.spells.len()
            }
            "equipment" => {
                let n = c.equipment.len();
                c.equipment.retain(|d| d.id != id);
                n - c.equipment.len()
            }
            _ => return Err(format!("unknown custom kind '{kind}'")),
        };
        if n == 0 {
            return Err("no such custom entry".into());
        }
        self.sync_custom();
        Ok(self.view())
    }

    pub fn history_view(&self) -> HistoryView {
        HistoryView {
            can_undo: !self.undo_stack.is_empty(),
            can_redo: !self.redo_stack.is_empty(),
            undo_label: self.undo_stack.back().map(|(l, _)| l.clone()),
            redo_label: self.redo_stack.last().map(|(l, _)| l.clone()),
            undo_depth: self.undo_stack.len(),
            dirty: self.character != self.saved,
        }
    }

    fn session_view(&self, sheet: &Sheet) -> SessionView {
        let max = |st: Stat| sheet.row(st).map(|r| r.total.floor() as i32).unwrap_or(10);
        let (hp_max, fp_max) = (max(Stat::HP), max(Stat::FP));
        let ss = &self.character.session;
        SessionView {
            hp: ss.hp.unwrap_or(hp_max),
            hp_max,
            fp: ss.fp.unwrap_or(fp_max),
            fp_max,
        }
    }

    /// Sets current HP or FP (`None` = reset to maximum). HP is kept within
    /// [-5×max, max] (death threshold, B327 # unverified), FP within [-max, max].
    pub fn set_current(&mut self, stat: &str, value: Option<i32>) -> Result<SheetView, String> {
        let sheet = self.sheet_of(&self.calc());
        let sv = self.session_view(&sheet);
        let ss = &mut self.character.session;
        match stat {
            "HP" => {
                ss.hp = value
                    .map(|v| v.clamp(-5 * sv.hp_max, sv.hp_max))
                    .filter(|&v| v != sv.hp_max)
            }
            "FP" => {
                ss.fp = value
                    .map(|v| v.clamp(-sv.fp_max, sv.fp_max))
                    .filter(|&v| v != sv.fp_max)
            }
            _ => return Err(format!("unknown session value {stat}")),
        }
        Ok(self.view())
    }

    /// Relative names go to `save_dir`; absolute paths are kept.
    pub fn resolve_path(&self, path: &str) -> PathBuf {
        let p = PathBuf::from(path.trim());
        match &self.save_dir {
            Some(d) if p.is_relative() => d.join(p),
            _ => p,
        }
    }

    pub fn load(&mut self, path: &str) -> Result<SheetView, String> {
        let path = self.resolve_path(path);
        let ch = gurps_io::load(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        self.character = ch;
        self.path = Some(path);
        self.reset_history();
        let v = self.view();
        self.saved = self.character.clone();
        Ok(v)
    }

    pub fn save(&mut self, path: Option<String>) -> Result<String, String> {
        let p = path
            .filter(|p| !p.trim().is_empty())
            .map(|p| self.resolve_path(&p))
            .or_else(|| self.path.clone())
            .ok_or("no file path given")?;
        if let Some(dir) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        gurps_io::save(&p, &self.character).map_err(|e| format!("{}: {e}", p.display()))?;
        self.path = Some(p.clone());
        self.saved = self.character.clone();
        Ok(p.display().to_string())
    }

    pub fn set_attribute(&mut self, key: &str, level: i32) -> Result<SheetView, String> {
        let stat = Stat::parse(key).ok_or_else(|| format!("unknown attribute {key}"))?;
        if !self.character.attributes.set_purchased(stat, level) {
            return Err(format!("{key} cannot be bought directly"));
        }
        Ok(self.view())
    }

    /// Rules option: rounding of fractional negative trait costs.
    pub fn set_negative_rounding(&mut self, mode: &str) -> Result<SheetView, String> {
        self.character.rules.negative_rounding = match mode {
            "toward_positive" => gurps_core::NegativeRounding::TowardPositive,
            "by_magnitude" => gurps_core::NegativeRounding::ByMagnitude,
            _ => return Err(format!("unknown rounding mode {mode}")),
        };
        Ok(self.view())
    }

    /// Replaces the notes; trailing whitespace-only text counts as empty.
    pub fn set_notes(&mut self, text: String) -> SheetView {
        self.character.notes = if text.trim().is_empty() {
            String::new()
        } else {
            text
        };
        self.view()
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
        self.translated_kind("trait", id)
    }

    fn translated_kind(&self, kind: &str, id: &str) -> Option<&str> {
        if self.original_only {
            return None;
        }
        self.catalog.name(&self.locale, kind, id)
    }

    /// Fluent description, else the plugin's own text.
    fn text_for(&self, kind: &str, id: &str, own: Option<&LocalizedText>) -> Option<String> {
        (!self.original_only)
            .then(|| self.catalog.description(&self.locale, kind, id))
            .flatten()
            .or_else(|| own.and_then(|d| d.get(self.data_locale())))
            .map(str::to_owned)
    }

    fn skill_def(&self, id: &str) -> Option<&SkillDef> {
        self.skill_library
            .iter()
            .map(|(_, d)| d)
            .find(|d| d.id == id)
    }

    fn technique_def(&self, id: &str) -> Option<&TechniqueDef> {
        self.technique_library
            .iter()
            .map(|(_, d)| d)
            .find(|d| d.id == id)
    }

    /// Skill name in the UI language (no specialization).
    fn skill_name(&self, id: &str) -> String {
        self.translated_kind("skill", id)
            .map(str::to_owned)
            .or_else(|| self.skill_def(id).map(|d| d.name.clone()))
            .or_else(|| {
                self.character
                    .skills
                    .iter()
                    .find(|s| s.def.id == id)
                    .map(|s| s.def.name.clone())
            })
            .unwrap_or_else(|| id.to_owned())
    }

    fn default_text(&self, d: &SkillDefault) -> String {
        let sg = |m: i32| {
            if m >= 0 {
                format!("+{m}")
            } else {
                m.to_string()
            }
        };
        match d {
            SkillDefault::Attribute {
                attribute,
                modifier,
            } => format!("{}{}", attribute.key(), sg(*modifier)),
            SkillDefault::Skill { skill, modifier } => {
                format!("{}{}", self.skill_name(skill), sg(*modifier))
            }
        }
    }

    fn skill_views(&self, sheet: &Sheet) -> Vec<SkillView> {
        self.character
            .skills
            .iter()
            .zip(&sheet.skills)
            .map(|(s, r)| {
                let tr = self.translated_kind("skill", &s.def.id).map(|n| {
                    match gurps_core::non_blank(s.specialization.clone()) {
                        Some(sp) => format!("{n} ({})", sp.trim()),
                        None => n.to_owned(),
                    }
                });
                let own = self
                    .skill_def(&s.def.id)
                    .and_then(|d| d.description.as_ref())
                    .or(s.def.description.as_ref());
                let default_explanation = self.text_for("skill", &s.def.id, own);
                SkillView {
                    index: r.index,
                    id: s.def.id.clone(),
                    original_name: tr.as_ref().map(|_| s.display_name()),
                    name: tr.unwrap_or_else(|| s.display_name()),
                    attribute: s.def.attribute.key().to_string(),
                    difficulty: s.def.difficulty.key().to_string(),
                    points: s.points,
                    level: r.level,
                    relative: r.relative.clone(),
                    from_default: r.from_default,
                    bonus: 0,
                    explain: r.explain.clone(),
                    defaults: s
                        .def
                        .defaults
                        .iter()
                        .map(|d| self.default_text(d))
                        .collect(),
                    specialization: s.specialization.clone(),
                    specialization_required: s.def.specialization_required,
                    specializations: s.def.specializations.clone(),
                    page: s.def.page.clone(),
                    source: s.source.clone(),
                    explanation: gurps_core::non_blank(s.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: s.has_custom_explanation(),
                }
            })
            .collect()
    }

    fn technique_views(&self, sheet: &Sheet) -> Vec<TechniqueView> {
        self.character
            .techniques
            .iter()
            .zip(&sheet.techniques)
            .map(|(t, r)| {
                let tr = self
                    .translated_kind("technique", &t.def.id)
                    .map(|n| format!("{n} ({})", self.skill_name(&t.def.skill)));
                let own = self
                    .technique_def(&t.def.id)
                    .and_then(|d| d.description.as_ref())
                    .or(t.def.description.as_ref());
                let default_explanation = self.text_for("technique", &t.def.id, own);
                let sk = self.skill_name(&t.def.skill);
                TechniqueView {
                    index: r.index,
                    id: t.def.id.clone(),
                    original_name: tr.as_ref().map(|_| t.def.name.clone()),
                    name: tr.unwrap_or_else(|| t.def.name.clone()),
                    skill: t.def.skill.clone(),
                    difficulty: t.def.difficulty.key().to_string(),
                    modifier: t.def.modifier,
                    max: t.def.max,
                    points: t.points,
                    level: r.level,
                    relative: r.level.zip(r.max_level).map(|(l, cap)| {
                        let d = l - (cap - t.def.max);
                        format!(
                            "{sk}{}",
                            if d >= 0 {
                                format!("+{d}")
                            } else {
                                d.to_string()
                            }
                        )
                    }),
                    max_level: r.max_level,
                    explain: r.explain.clone(),
                    page: t.def.page.clone(),
                    source: t.source.clone(),
                    explanation: gurps_core::non_blank(t.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: t.has_custom_explanation(),
                }
            })
            .collect()
    }

    fn equipment_views(&self) -> Vec<EquipmentView> {
        let mut out = Vec::new();
        self.walk_equipment(&self.character.equipment, &mut Vec::new(), true, &mut out);
        out
    }

    fn walk_equipment(
        &self,
        list: &[Equipment],
        path: &mut Vec<usize>,
        parent_carried: bool,
        out: &mut Vec<EquipmentView>,
    ) {
        for (i, e) in list.iter().enumerate() {
            path.push(i);
            let tr = self
                .translated_kind("equipment", &e.def.id)
                .map(str::to_owned);
            let own = self
                .equipment_library
                .iter()
                .find(|(_, d)| !e.def.id.is_empty() && d.id == e.def.id)
                .and_then(|(_, d)| d.description.as_ref())
                .or(e.def.description.as_ref());
            let carried = parent_carried && e.carried;
            out.push(EquipmentView {
                path: path.clone(),
                depth: path.len() - 1,
                id: e.def.id.clone(),
                original_name: tr.as_ref().map(|_| e.def.name.clone()),
                name: tr.unwrap_or_else(|| e.def.name.clone()),
                quantity: e.quantity,
                weight: e.def.weight,
                cost: e.def.cost,
                total_weight: e.total_weight(),
                total_cost: e.total_cost(),
                carried: e.carried,
                effectively_carried: carried,
                container: e.def.container,
                stats: e.def.stats.clone(),
                page: e.def.page.clone(),
                source: e.source.clone(),
                explanation: gurps_core::non_blank(e.explanation.clone())
                    .or_else(|| self.text_for("equipment", &e.def.id, own)),
                default_explanation: self.text_for("equipment", &e.def.id, own),
                custom_explanation: !gurps_core::is_blank(&e.explanation),
            });
            self.walk_equipment(&e.contents, path, carried, out);
            path.pop();
        }
    }

    pub fn equipment_library_view(&self) -> Vec<EquipmentLibEntry> {
        self.equipment_library
            .iter()
            .map(|(plugin, def)| EquipmentLibEntry {
                plugin: plugin.clone(),
                explanation: self.text_for("equipment", &def.id, def.description.as_ref()),
                translated_name: self
                    .translated_kind("equipment", &def.id)
                    .map(str::to_owned),
                def: def.clone(),
            })
            .collect()
    }

    /// Adds a library item at the top level or into the container at `parent`.
    pub fn add_equipment(
        &mut self,
        id: &str,
        quantity: Option<u32>,
        parent: Option<Vec<usize>>,
    ) -> Result<SheetView, String> {
        let (plugin, def) = self
            .equipment_library
            .iter()
            .find(|(_, d)| d.id == id)
            .ok_or_else(|| format!("equipment {id} not in library"))?;
        let mut def = def.clone();
        def.description = None;
        let mut e = Equipment::new(def, quantity.unwrap_or(1).max(1));
        e.source = Some(plugin.clone());
        let linked: Vec<Weapon> = self
            .weapon_library
            .iter()
            .filter(|(_, w)| w.equipment.as_deref() == Some(id))
            .filter(|(_, w)| !self.character.weapons.iter().any(|x| x.def.id == w.id))
            .map(|(p, w)| Self::weapon_snapshot(p, w))
            .collect();
        match parent.filter(|p| !p.is_empty()) {
            None => self.character.equipment.push(e),
            Some(p) => {
                let c = Equipment::at_path_mut(&mut self.character.equipment, &p)
                    .ok_or("no such container")?;
                if !c.def.container {
                    return Err(format!("{} is not a container", c.def.name));
                }
                c.contents.push(e);
            }
        }
        self.character.weapons.extend(linked);
        Ok(self.view())
    }

    fn equipment_mut(&mut self, path: &[usize]) -> Result<&mut Equipment, String> {
        Equipment::at_path_mut(&mut self.character.equipment, path)
            .ok_or_else(|| "no such item".into())
    }

    pub fn set_equipment_quantity(
        &mut self,
        path: &[usize],
        quantity: u32,
    ) -> Result<SheetView, String> {
        self.equipment_mut(path)?.quantity = quantity;
        Ok(self.view())
    }

    pub fn set_equipment_carried(
        &mut self,
        path: &[usize],
        carried: bool,
    ) -> Result<SheetView, String> {
        self.equipment_mut(path)?.carried = carried;
        Ok(self.view())
    }

    pub fn set_equipment_explanation(
        &mut self,
        path: &[usize],
        text: Option<String>,
    ) -> Result<SheetView, String> {
        self.equipment_mut(path)?.explanation = gurps_core::non_blank(text);
        Ok(self.view())
    }

    pub fn remove_equipment(&mut self, path: &[usize]) -> Result<SheetView, String> {
        let i = *path.last().ok_or("empty path")?;
        let list = Equipment::parent_list_mut(&mut self.character.equipment, path)
            .filter(|l| i < l.len())
            .ok_or("no such item")?;
        list.remove(i);
        self.prune_weapons();
        Ok(self.view())
    }

    fn weapon_snapshot(plugin: &str, d: &WeaponDef) -> Weapon {
        let mut d = d.clone();
        d.description = None;
        let mut w = Weapon::new(d);
        w.source = Some(plugin.to_owned());
        w
    }

    /// Drops linked weapons whose equipment is no longer owned (anywhere, carried or not).
    fn prune_weapons(&mut self) {
        fn owned(list: &[Equipment], id: &str) -> bool {
            list.iter()
                .any(|e| e.def.id == id || owned(&e.contents, id))
        }
        let eq = &self.character.equipment;
        self.character
            .weapons
            .retain(|w| w.def.equipment.as_deref().is_none_or(|id| owned(eq, id)));
    }

    fn weapon_views(&self, sheet: &Sheet) -> Vec<WeaponView> {
        sheet
            .weapons
            .iter()
            .map(|r| {
                let w = &self.character.weapons[r.index];
                let tr = self.translated_kind("weapon", &w.def.id).map(str::to_owned);
                let own = self
                    .weapon_library
                    .iter()
                    .find(|(_, d)| d.id == w.def.id)
                    .and_then(|(_, d)| d.description.as_ref())
                    .or(w.def.description.as_ref());
                let default_explanation = self.text_for("weapon", &w.def.id, own);
                WeaponView {
                    index: r.index,
                    id: r.id.clone(),
                    original_name: tr.as_ref().map(|_| r.name.clone()),
                    name: tr.unwrap_or_else(|| r.name.clone()),
                    equipment: r.equipment.clone(),
                    available: r.available,
                    ready: r.ready,
                    attacks: r
                        .attacks
                        .iter()
                        .map(|a| {
                            let t = a.skill_id.as_deref().and_then(|id| {
                                self.translated_kind(
                                    if a.via_technique {
                                        "technique"
                                    } else {
                                        "skill"
                                    },
                                    id,
                                )
                            });
                            let mut row = a.clone();
                            let original_skill =
                                t.map(|t| std::mem::replace(&mut row.skill, t.to_owned()));
                            AttackView {
                                row,
                                original_skill,
                            }
                        })
                        .collect(),
                    page: w.def.page.clone(),
                    source: w.source.clone(),
                    explanation: gurps_core::non_blank(w.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: !gurps_core::is_blank(&w.explanation),
                }
            })
            .collect()
    }

    pub fn weapon_library_view(&self) -> Vec<WeaponLibEntry> {
        self.weapon_library
            .iter()
            .map(|(plugin, def)| WeaponLibEntry {
                plugin: plugin.clone(),
                explanation: self.text_for("weapon", &def.id, def.description.as_ref()),
                translated_name: self.translated_kind("weapon", &def.id).map(str::to_owned),
                equipment_name: def.equipment.as_ref().map(|e| {
                    self.translated_kind("equipment", e)
                        .map(str::to_owned)
                        .or_else(|| {
                            self.equipment_library
                                .iter()
                                .find(|(_, d)| &d.id == e)
                                .map(|(_, d)| d.name.clone())
                        })
                        .unwrap_or_else(|| e.clone())
                }),
                skill_name: self.skill_name(&def.skill),
                def: def.clone(),
            })
            .collect()
    }

    /// Adds a library weapon (once per id). Linked weapons show only while the item is owned.
    pub fn add_weapon(&mut self, id: &str) -> Result<SheetView, String> {
        if self.character.weapons.iter().any(|w| w.def.id == id) {
            return Err(format!("weapon {id} is already on the sheet"));
        }
        let (plugin, def) = self
            .weapon_library
            .iter()
            .find(|(_, d)| d.id == id)
            .ok_or_else(|| format!("weapon {id} not in library"))?;
        let w = Self::weapon_snapshot(plugin, def);
        self.character.weapons.push(w);
        Ok(self.view())
    }

    pub fn remove_weapon(&mut self, index: usize) -> Result<SheetView, String> {
        if index >= self.character.weapons.len() {
            return Err("no such weapon".into());
        }
        self.character.weapons.remove(index);
        Ok(self.view())
    }

    pub fn set_weapon_explanation(
        &mut self,
        index: usize,
        text: Option<String>,
    ) -> Result<SheetView, String> {
        self.character
            .weapons
            .get_mut(index)
            .ok_or("no such weapon")?
            .explanation = gurps_core::non_blank(text);
        Ok(self.view())
    }

    fn modifier_views(&self, t: &Trait) -> Vec<ModifierView> {
        t.modifiers
            .iter()
            .enumerate()
            .map(|(index, m)| {
                let tr = self.translated_kind("modifier", &m.def.id);
                let own = self
                    .modifier_library
                    .iter()
                    .find(|(_, d)| d.id == m.def.id)
                    .and_then(|(_, d)| d.description.as_ref())
                    .or(m.def.description.as_ref());
                let default_explanation = self.text_for("modifier", &m.def.id, own);
                ModifierView {
                    index,
                    id: m.def.id.clone(),
                    original_name: tr.map(|_| m.def.name.clone()),
                    name: tr.map(str::to_owned).unwrap_or_else(|| m.def.name.clone()),
                    cost_percent: m.def.cost_percent,
                    page: m.def.page.clone(),
                    source: m.source.clone(),
                    explanation: gurps_core::non_blank(m.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: m.has_custom_explanation(),
                }
            })
            .collect()
    }

    pub fn modifier_library_view(&self) -> Vec<ModifierLibEntry> {
        self.modifier_library
            .iter()
            .map(|(plugin, def)| ModifierLibEntry {
                plugin: plugin.clone(),
                explanation: self.text_for("modifier", &def.id, def.description.as_ref()),
                translated_name: self.translated_kind("modifier", &def.id).map(str::to_owned),
                def: def.clone(),
            })
            .collect()
    }

    fn trait_mut(&mut self, index: usize) -> Result<&mut Trait, String> {
        self.character
            .traits
            .get_mut(index)
            .ok_or_else(|| "no such trait".into())
    }

    pub fn add_trait_modifier(
        &mut self,
        trait_index: usize,
        id: &str,
    ) -> Result<SheetView, String> {
        let (plugin, def) = self
            .modifier_library
            .iter()
            .find(|(_, d)| d.id == id)
            .ok_or_else(|| format!("modifier {id} not in library"))?;
        let mut def = def.clone();
        def.description = None;
        let mut m = TraitModifier::new(def);
        m.source = Some(plugin.clone());
        self.trait_mut(trait_index)?.modifiers.push(m);
        Ok(self.view())
    }

    pub fn remove_trait_modifier(
        &mut self,
        trait_index: usize,
        index: usize,
    ) -> Result<SheetView, String> {
        let t = self.trait_mut(trait_index)?;
        if index >= t.modifiers.len() {
            return Err("no such modifier".into());
        }
        t.modifiers.remove(index);
        Ok(self.view())
    }

    pub fn set_modifier_explanation(
        &mut self,
        trait_index: usize,
        index: usize,
        text: Option<String>,
    ) -> Result<SheetView, String> {
        let m = self
            .trait_mut(trait_index)?
            .modifiers
            .get_mut(index)
            .ok_or("no such modifier")?;
        m.explanation = gurps_core::non_blank(text);
        Ok(self.view())
    }

    fn spell_views(&self, sheet: &Sheet) -> Vec<SpellView> {
        self.character
            .spells
            .iter()
            .zip(&sheet.spells)
            .map(|(s, r)| {
                let tr = self.translated_kind("spell", &s.def.id);
                let own = self
                    .spell_library
                    .iter()
                    .find(|(_, d)| d.id == s.def.id)
                    .and_then(|(_, d)| d.description.as_ref())
                    .or(s.def.description.as_ref());
                let default_explanation = self.text_for("spell", &s.def.id, own);
                SpellView {
                    index: r.index,
                    id: s.def.id.clone(),
                    original_name: tr.map(|_| s.def.name.clone()),
                    name: tr.map(str::to_owned).unwrap_or_else(|| s.def.name.clone()),
                    college: s.def.college.clone(),
                    attribute: s.def.attribute.key().to_string(),
                    difficulty: s.def.difficulty.key().to_string(),
                    points: s.points,
                    level: r.level,
                    relative: r.relative.clone(),
                    bonus: 0,
                    prerequisites: s.def.prerequisites.clone(),
                    explain: r.explain.clone(),
                    page: s.def.page.clone(),
                    source: s.source.clone(),
                    explanation: gurps_core::non_blank(s.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: s.has_custom_explanation(),
                }
            })
            .collect()
    }

    pub fn spell_library_view(&self) -> Vec<SpellLibEntry> {
        self.spell_library
            .iter()
            .map(|(plugin, def)| SpellLibEntry {
                plugin: plugin.clone(),
                explanation: self.text_for("spell", &def.id, def.description.as_ref()),
                translated_name: self.translated_kind("spell", &def.id).map(str::to_owned),
                def: def.clone(),
            })
            .collect()
    }

    pub fn add_spell(&mut self, id: &str, points: Option<i32>) -> Result<SheetView, String> {
        let (plugin, def) = self
            .spell_library
            .iter()
            .find(|(_, d)| d.id == id)
            .ok_or_else(|| format!("spell {id} not in library"))?;
        let mut def = def.clone();
        def.description = None;
        let mut s = Spell::new(def, points.unwrap_or(1).max(0));
        s.source = Some(plugin.clone());
        self.character.spells.push(s);
        Ok(self.view())
    }

    fn spell_mut(&mut self, index: usize) -> Result<&mut Spell, String> {
        self.character
            .spells
            .get_mut(index)
            .ok_or_else(|| "no such spell".into())
    }

    pub fn set_spell_points(&mut self, index: usize, points: i32) -> Result<SheetView, String> {
        self.spell_mut(index)?.points = points.max(0);
        Ok(self.view())
    }

    pub fn set_spell_explanation(
        &mut self,
        index: usize,
        text: Option<String>,
    ) -> Result<SheetView, String> {
        self.spell_mut(index)?.explanation = gurps_core::non_blank(text);
        Ok(self.view())
    }

    pub fn remove_spell(&mut self, index: usize) -> Result<SheetView, String> {
        if index >= self.character.spells.len() {
            return Err("no such spell".into());
        }
        self.character.spells.remove(index);
        Ok(self.view())
    }

    pub fn skill_library_view(&self) -> SkillLibrary {
        SkillLibrary {
            skills: self
                .skill_library
                .iter()
                .map(|(plugin, def)| SkillLibEntry {
                    plugin: plugin.clone(),
                    explanation: self.text_for("skill", &def.id, def.description.as_ref()),
                    translated_name: self.translated_kind("skill", &def.id).map(str::to_owned),
                    def: def.clone(),
                })
                .collect(),
            techniques: self
                .technique_library
                .iter()
                .map(|(plugin, def)| TechniqueLibEntry {
                    plugin: plugin.clone(),
                    explanation: self.text_for("technique", &def.id, def.description.as_ref()),
                    translated_name: self
                        .translated_kind("technique", &def.id)
                        .map(str::to_owned),
                    skill_name: self.skill_name(&def.skill),
                    def: def.clone(),
                })
                .collect(),
        }
    }

    pub fn add_skill(
        &mut self,
        id: &str,
        specialization: Option<String>,
        points: Option<i32>,
    ) -> Result<SheetView, String> {
        let (plugin, def) = self
            .skill_library
            .iter()
            .find(|(_, d)| d.id == id)
            .ok_or_else(|| format!("skill {id} not in library"))?;
        let mut def = def.clone();
        def.description = None;
        let mut s = Skill::new(def, points.unwrap_or(1).max(0));
        s.specialization = gurps_core::non_blank(specialization);
        s.source = Some(plugin.clone());
        self.character.skills.push(s);
        Ok(self.view())
    }

    fn skill_mut(&mut self, index: usize) -> Result<&mut Skill, String> {
        self.character
            .skills
            .get_mut(index)
            .ok_or_else(|| "no such skill".into())
    }

    pub fn set_skill_points(&mut self, index: usize, points: i32) -> Result<SheetView, String> {
        self.skill_mut(index)?.points = points.max(0);
        Ok(self.view())
    }

    pub fn set_skill_specialization(
        &mut self,
        index: usize,
        specialization: Option<String>,
    ) -> Result<SheetView, String> {
        self.skill_mut(index)?.specialization = gurps_core::non_blank(specialization);
        Ok(self.view())
    }

    pub fn set_skill_explanation(
        &mut self,
        index: usize,
        text: Option<String>,
    ) -> Result<SheetView, String> {
        self.skill_mut(index)?.explanation = gurps_core::non_blank(text);
        Ok(self.view())
    }

    pub fn remove_skill(&mut self, index: usize) -> Result<SheetView, String> {
        if index >= self.character.skills.len() {
            return Err("no such skill".into());
        }
        self.character.skills.remove(index);
        Ok(self.view())
    }

    pub fn add_technique(&mut self, id: &str, points: Option<i32>) -> Result<SheetView, String> {
        let (plugin, def) = self
            .technique_library
            .iter()
            .find(|(_, d)| d.id == id)
            .ok_or_else(|| format!("technique {id} not in library"))?;
        let mut def = def.clone();
        def.description = None;
        let mut t = Technique::new(def, points.unwrap_or(0).max(0));
        t.source = Some(plugin.clone());
        self.character.techniques.push(t);
        Ok(self.view())
    }

    fn technique_mut(&mut self, index: usize) -> Result<&mut Technique, String> {
        self.character
            .techniques
            .get_mut(index)
            .ok_or_else(|| "no such technique".into())
    }

    pub fn set_technique_points(&mut self, index: usize, points: i32) -> Result<SheetView, String> {
        self.technique_mut(index)?.points = points.max(0);
        Ok(self.view())
    }

    pub fn set_technique_explanation(
        &mut self,
        index: usize,
        text: Option<String>,
    ) -> Result<SheetView, String> {
        self.technique_mut(index)?.explanation = gurps_core::non_blank(text);
        Ok(self.view())
    }

    pub fn remove_technique(&mut self, index: usize) -> Result<SheetView, String> {
        if index >= self.character.techniques.len() {
            return Err("no such technique".into());
        }
        self.character.techniques.remove(index);
        Ok(self.view())
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

    /// 3d6 success roll; `seed` makes it deterministic (tests, replays).
    pub fn roll(&mut self, target: i32, modifier: i32, seed: Option<u64>) -> RollResult {
        match seed {
            Some(s) => success_roll(target, modifier, &mut SplitMix(s)),
            None => success_roll(target, modifier, &mut self.rng),
        }
    }

    pub fn roll_damage(
        &mut self,
        expr: &str,
        crushing: bool,
        seed: Option<u64>,
    ) -> Result<gurps_dice::DamageRoll, String> {
        let r = match seed {
            Some(s) => gurps_dice::damage_roll(expr, crushing, &mut SplitMix(s)),
            None => gurps_dice::damage_roll(expr, crushing, &mut self.rng),
        };
        r.ok_or_else(|| format!("cannot parse damage {expr}"))
    }
}

fn pct(v: i32) -> String {
    if v >= 0 {
        format!("+{v}%")
    } else {
        format!("{v}%")
    }
}

/// Строки пояснения цены черты с модификаторами (пусто без модификаторов).
fn trait_cost_explain(t: &Trait, r: gurps_core::NegativeRounding) -> Vec<String> {
    use gurps_core::modifier::MIN_TOTAL_PERCENT;
    if t.modifiers.is_empty() {
        return Vec::new();
    }
    let base = t.base_cost();
    let mut out = vec![format!("база {base}")];
    for m in &t.modifiers {
        out.push(format!("{} {}", pct(m.def.cost_percent), m.def.name));
    }
    let total = t.modifier_percent();
    if total < MIN_TOTAL_PERCENT {
        out.push(format!(
            "итого {}, не ниже {}",
            pct(total),
            pct(MIN_TOTAL_PERCENT)
        ));
    } else {
        out.push(format!("итого {}", pct(total)));
    }
    let eff = 100 + total.max(MIN_TOTAL_PERCENT);
    let exact = base as f64 * eff as f64 / 100.0;
    out.push(format!(
        "{base} × {eff}% = {} -> {}",
        gurps_library::sheet::fmt_num(exact),
        t.cost_with(r)
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Состояние на тестовой БД (testkit: схема и сиды); без `surreal` тест пропускается.
    struct T {
        s: AppState,
        _srv: gurps_db::testkit::Server,
    }

    impl std::ops::Deref for T {
        type Target = AppState;
        fn deref(&self) -> &AppState {
            &self.s
        }
    }

    impl std::ops::DerefMut for T {
        fn deref_mut(&mut self) -> &mut AppState {
            &mut self.s
        }
    }

    fn state() -> Option<T> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (srv, _db) = rt.block_on(gurps_db::testkit::start())?;
        let s = AppState::with_source(Some(gurps_library::LibrarySource::Db(srv.cfg.clone())));
        Some(T { s, _srv: srv })
    }

    #[test]
    fn acceptance_flow() {
        let Some(mut s) = state() else {
            return;
        };
        assert!(s.plugin_errors.is_empty());
        s.new_character(Some("Hero".into()));
        s.set_attribute("ST", 2).unwrap();
        let v = s.add_trait("reflexes-a", None).unwrap();
        assert!(
            v.issues.iter().all(|i| i.severity != "error"),
            "{:?}",
            v.issues
        );
        let st = v.sheet.row(Stat::ST).unwrap();
        assert_eq!((st.total, st.points, st.level), (12.0, 20, 2));
        assert_eq!(st.value.contributions[0].amount, 2.0);
        assert_eq!(st.label, "Сила", "подпись из перевода БД");
        // reflexes_a: +1 Per (заглушка сидов)
        let per = v.sheet.row(Stat::Per).unwrap();
        assert_eq!((per.value.base, per.total), (10.0, 11.0));
        assert_eq!(per.value.contributions[0].amount, 1.0);
        assert!(
            per.explain.iter().any(|e| e.contains("черта")),
            "{:?}",
            per.explain
        );
        assert_eq!(v.sheet.row(Stat::HP).unwrap().total, 12.0);
        assert_eq!(v.sheet.points.total, 21);
        let v = s.remove_trait(0).unwrap();
        assert_eq!(v.sheet.points.total, 20);
        assert!(s.remove_trait(5).is_err());
        assert!(s.set_attribute("BasicLift", 1).is_err());
        assert!(s.add_trait("nope", None).is_err());
    }

    #[test]
    fn undo_redo_history() {
        let Some(mut s) = state() else {
            return;
        };
        s.new_character(Some("Hero".into()));
        assert!(!s.history_view().can_undo && !s.history_view().dirty);
        s.record("изменение ST", |s| s.set_attribute("ST", 2))
            .unwrap();
        s.record("добавление черты", |s| {
            s.add_trait("reflexes-a", None)
        })
        .unwrap();
        let h = s.history_view();
        assert_eq!(h.undo_depth, 2);
        assert!(h.undo_label.as_deref().unwrap().contains("reflexes_a"));
        assert!(h.dirty);
        // failed edit: not recorded, character untouched
        assert!(s.record("x", |s| s.remove_trait(9)).is_err());
        // no-op edit: not recorded
        s.record("изменение ST", |s| s.set_attribute("ST", 2))
            .unwrap();
        assert_eq!(s.history_view().undo_depth, 2);
        let v = s.undo().unwrap();
        assert!(v.traits.is_empty());
        assert!(v
            .history
            .redo_label
            .as_deref()
            .unwrap()
            .contains("reflexes_a"));
        s.undo().unwrap();
        assert!(!s.history_view().dirty, "back at the new-character state");
        assert!(s.undo().is_err());
        let v = s.redo().unwrap();
        assert_eq!(v.sheet.points.attributes, 20);
        // a new edit clears redo
        s.record("переименование", |s| {
            Ok(s.set_name("B".into()))
        })
        .unwrap();
        assert!(!s.history_view().can_redo);
        assert!(s.redo().is_err());
        // bounded
        for i in 0..250 {
            s.record("изменение DX", |s| {
                s.set_attribute("DX", i % 7 + 1)
            })
            .unwrap();
        }
        assert_eq!(s.history_view().undo_depth, HISTORY_LIMIT);
        // session values are undoable and separate from build points
        let pts = s.view().sheet.points.total;
        let v = s
            .record("текущие HP", |s| s.set_current("HP", Some(4)))
            .unwrap();
        assert_eq!(
            (v.session.hp, v.session.hp_max, v.sheet.points.total),
            (4, 12, pts)
        );
        assert_eq!(s.set_current("HP", Some(-500)).unwrap().session.hp, -60);
        assert_eq!(s.set_current("FP", Some(99)).unwrap().session.fp, 10);
        assert!(s.character.session.fp.is_none(), "max is stored as None");
        s.undo().unwrap();
        assert_eq!(s.view().session.hp, 12);
        assert!(s.set_current("XP", None).is_err());
        // load/new clear history; save resets dirty
        let dir = std::env::temp_dir().join("gurps-undo-test.json");
        s.save(Some(dir.display().to_string())).unwrap();
        assert!(!s.history_view().dirty && s.history_view().can_undo);
        s.load(&dir.display().to_string()).unwrap();
        assert!(!s.history_view().can_undo && !s.history_view().can_redo);
        s.new_character(None);
        assert_eq!(s.history_view().undo_depth, 0);
    }

    #[test]
    fn notes_are_one_undo_step_and_skipped_when_empty() {
        let Some(mut s) = state() else {
            return;
        };
        s.new_character(None);
        assert!(!serde_json::to_string(&s.character)
            .unwrap()
            .contains("notes"));
        let v = s
            .record("заметки", |s| {
                Ok(s.set_notes("# Долги\n- гному 5 $".into()))
            })
            .unwrap();
        assert_eq!(v.notes, "# Долги\n- гному 5 $");
        let h = s.history_view();
        assert_eq!(h.undo_label.as_deref(), Some("заметки"));
        assert!(h.dirty);
        let json = serde_json::to_string(&s.character).unwrap();
        assert!(json.contains("\"notes\""));
        // old files without the field still load
        let back: gurps_core::Character =
            serde_json::from_str(&json.replace(",\"notes\":\"# Долги\\n- гному 5 $\"", ""))
                .unwrap();
        assert_eq!(back.notes, "");
        assert_eq!(s.undo().unwrap().notes, "");
        assert_eq!(s.set_notes("  \n".into()).notes, "");
    }

    fn li(kind: gurps_core::LedgerKind, amount: i32, note: &str) -> LedgerInput {
        LedgerInput {
            date: "2026-10-23".into(),
            amount,
            kind,
            note: note.into(),
            skill: None,
        }
    }

    #[test]
    fn point_pool_ledger() {
        use gurps_core::LedgerKind::*;
        let Some(mut s) = state() else {
            return;
        };
        s.new_character(None);
        let p = s.view().pool;
        assert_eq!(
            (p.entries.len(), p.starting, p.unspent, p.disadvantage_limit),
            (1, 150, 150, 75)
        );
        s.record("x", |s| s.set_attribute("DX", 1)).unwrap(); // 20 pts
        let v = s
            .record("запись", |s| {
                s.add_ledger_entry(li(SessionAward, 3, ""))
            })
            .unwrap();
        assert_eq!(v.pool.entries[1].id, 2);
        s.add_ledger_entry(li(GmBonus, 1, "подарок от Мастера"))
            .unwrap();
        s.add_ledger_entry(li(Influence, 2, "")).unwrap();
        let p = s.view().pool;
        assert_eq!(
            (p.earned, p.spent_outside, p.character_cost, p.unspent),
            (154, 2, 20, 132)
        );
        assert_eq!(s.view().budget, 152);
        assert_eq!(
            s.character.points_budget, 152,
            "legacy field mirrors available points"
        );
        assert!(s.add_ledger_entry(li(Other, 1, " ")).is_err());
        // edit and delete
        let v = s.update_ledger_entry(3, li(GmBonus, 2, "роль")).unwrap();
        assert_eq!(v.pool.entries[2].amount, 2);
        assert_eq!(v.pool.entries[2].note, "роль");
        assert!(s.update_ledger_entry(99, li(GmBonus, 2, "")).is_err());
        s.record("удаление", |s| s.remove_ledger_entry(4)).unwrap();
        assert_eq!(s.view().pool.spent_outside, 0);
        s.undo().unwrap();
        assert_eq!(s.view().pool.spent_outside, 2);
        // negative unspent allowed
        s.add_ledger_entry(li(BuyOff, 200, "")).unwrap();
        assert!(s.view().pool.unspent < 0);
        // disadvantage limit
        assert_eq!(
            s.set_disadvantage_limit(Some(40))
                .unwrap()
                .pool
                .disadvantage_limit,
            40
        );
        assert!(s.set_disadvantage_limit(Some(-1)).is_err());
        assert!(
            !s.set_disadvantage_limit(None)
                .unwrap()
                .pool
                .disadvantage_limit_custom
        );
    }

    #[test]
    fn replace_starting_entry_then_save_and_reload() {
        use gurps_core::LedgerKind::*;
        let Some(mut s) = state() else {
            return;
        };
        s.new_character(Some("Bug".into()));
        s.record("x", |s| s.remove_ledger_entry(1)).unwrap();
        let v = s
            .record("x", |s| s.add_ledger_entry(li(Starting, 200, "")))
            .unwrap();
        let _ = v;
        let p = s.view().pool;
        assert_eq!(
            (p.entries.len(), p.starting, p.available),
            (1, 200, 200),
            "{:?}",
            p.entries
        );
        let path = std::env::temp_dir().join(format!("gurps-bug-{}.json", std::process::id()));
        s.save(Some(path.display().to_string())).unwrap();
        assert!(!s.history_view().dirty);
        let Some(mut t) = state() else {
            return;
        };
        let v = t.load(&path.display().to_string()).unwrap();
        assert_eq!(
            (v.pool.entries.len(), v.pool.starting, v.budget),
            (1, 200, 200)
        );
        assert!(!v.history.dirty);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn relative_save_goes_to_save_dir_not_cwd() {
        let dir = std::env::temp_dir().join(format!("gurps-savedir-{}", std::process::id()));
        let Some(mut s) = state() else {
            return;
        };
        s.save_dir = Some(dir.join("GURPS Sheet"));
        s.new_character(Some("Rel".into()));
        let out = s.save(Some("character.gurps.json".into())).unwrap();
        assert!(
            dir.join("GURPS Sheet/character.gurps.json").is_file(),
            "{out}"
        );
        assert!(
            !std::path::Path::new("character.gurps.json").exists(),
            "nothing in the CWD"
        );
        assert_eq!(s.load("character.gurps.json").unwrap().sheet.name, "Rel");
        // errors are returned, not panics
        assert!(s
            .save(Some(dir.join("f\0").join("x").display().to_string() + "\0"))
            .is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn legacy_budget_migrates_on_load() {
        let Some(mut s) = state() else {
            return;
        };
        let path = std::env::temp_dir().join(format!("gurps-legacy-{}.json", std::process::id()));
        std::fs::write(&path, r#"{"format":"gurps-sheet","version":1,"character":{"name":"Old","points_budget":100}}"#).unwrap();
        let v = s.load(&path.display().to_string()).unwrap();
        assert_eq!(v.pool.entries.len(), 1);
        assert_eq!(v.pool.entries[0].kind, gurps_core::LedgerKind::Starting);
        assert_eq!((v.pool.starting, v.budget), (100, 100));
        assert!(!v.history.dirty, "migration alone is not an unsaved change");
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn custom_definitions() {
        let Some(mut s) = state() else {
            return;
        };
        s.new_character(Some("Маг".into()));
        let spell = serde_json::json!({"name": "Искра", "difficulty": "VH", "college": ["Fire"],
            "prerequisites": "Magery 1", "description": "**жжёт**"});
        let v = s.record("своё", |s| s.save_custom("spell", spell)).unwrap();
        let id = v.custom.spells[0].id.clone();
        assert_eq!(id, "custom-spell-1");
        let lib = s.spell_library_view();
        let e = lib.iter().find(|e| e.def.id == id).unwrap();
        assert_eq!(e.plugin, CUSTOM);
        assert_eq!(e.explanation.as_deref(), Some("**жжёт**"));
        s.add_spell(&id, Some(4)).unwrap();
        // editing renames the sheet copy too
        s.save_custom(
            "spell",
            serde_json::json!({"id": id, "name": "Искра!", "difficulty": "H"}),
        )
        .unwrap();
        assert_eq!(s.character.spells[0].def.name, "Искра!");
        assert!(s
            .save_custom("spell", serde_json::json!({"name": " "}))
            .is_err());
        assert!(s
            .save_custom("weapon", serde_json::json!({"name": "x"}))
            .is_err());
        s.save_custom(
            "trait",
            serde_json::json!({"name": "Везучесть в картах", "category": "advantage", "cost": 3}),
        )
        .unwrap();
        s.add_trait("custom-trait-1", None).unwrap();
        assert_eq!(s.view().sheet.points.advantages, 3);
        s.save_custom(
            "skill",
            serde_json::json!({"name": "Гадание", "attribute": "IQ", "difficulty": "A"}),
        )
        .unwrap();
        s.save_custom(
            "equipment",
            serde_json::json!({"name": "Колода", "weight": 0.1, "cost": 5}),
        )
        .unwrap();
        // survives undo/redo and plugin reload
        s.undo().unwrap();
        assert!(s
            .equipment_library_view()
            .iter()
            .all(|e| e.def.name != "Колода"));
        s.redo().unwrap();
        s.reload_plugins();
        assert!(s
            .equipment_library_view()
            .iter()
            .any(|e| e.def.name == "Колода" && e.plugin == CUSTOM));
        // remove
        s.remove_custom("spell", &id).unwrap();
        assert!(s.spell_library_view().iter().all(|e| e.def.id != id));
        assert!(s.remove_custom("spell", &id).is_err());
    }

    #[test]
    fn seeded_rolls() {
        let Some(mut s) = state() else {
            return;
        };
        let a = s.roll(12, -1, Some(42));
        assert_eq!(a, s.roll(12, -1, Some(42)));
        assert_eq!(a.effective, 11);
        assert_eq!(a.margin, 11 - a.roll);
        assert_eq!(a.outcome, gurps_dice::outcome(a.roll, 11));
        let d = s.roll_damage("2d-1", false, Some(5)).unwrap();
        assert_eq!(d, s.roll_damage("2d-1", false, Some(5)).unwrap());
        assert_eq!(d.dice.len(), 2);
        assert!(s.roll_damage("zz", false, None).is_err());
    }

    #[test]
    fn negative_rounding_option() {
        let Some(mut s) = state() else {
            return;
        };
        s.new_character(None);
        s.add_trait("temper-a", None).unwrap();
        // -1 × 90% = -0.9: вверх -> 0, от нуля -> -1; расчёт (rule_overrides) согласован
        let v = s.add_trait_modifier(0, "limit-a").unwrap();
        assert_eq!(v.traits[0].cost, 0);
        assert_eq!(v.sheet.points.disadvantages, 0);
        let v = s.set_negative_rounding("by_magnitude").unwrap();
        assert_eq!(v.traits[0].cost, -1);
        assert_eq!(v.sheet.points.disadvantages, -1);
        assert!(v.traits[0].cost_explain.last().unwrap().ends_with("-> -1"));
        let json = serde_json::to_string(&s.character).unwrap();
        assert!(json.contains("\"negative_rounding\":\"by_magnitude\""));
        assert!(s.set_negative_rounding("x").is_err());
        let v = s.set_negative_rounding("toward_positive").unwrap();
        assert_eq!(v.traits[0].cost, 0);
        assert!(!serde_json::to_string(&s.character)
            .unwrap()
            .contains("rules"));
    }

    #[test]
    fn save_load() {
        let Some(mut s) = state() else {
            return;
        };
        s.set_attribute("DX", 1).unwrap();
        let p = std::env::temp_dir().join(format!("app-{}.json", std::process::id()));
        s.save(Some(p.display().to_string())).unwrap();
        let Some(mut s2) = state() else {
            return;
        };
        let v = s2.load(&p.display().to_string()).unwrap();
        assert_eq!(v.sheet.points.attributes, 20);
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn explanation_override_and_reset() {
        let Some(mut s) = state() else {
            return;
        };
        let v = s.add_trait("reflexes-a", None).unwrap();
        let default = v.traits[0].explanation.clone();
        assert_eq!(v.traits[0].default_explanation, default);
        assert!(!v.traits[0].custom_explanation);

        let v = s
            .set_trait_explanation(0, Some("**Своё** пояснение".into()))
            .unwrap();
        assert_eq!(
            v.traits[0].explanation.as_deref(),
            Some("**Своё** пояснение")
        );
        assert!(v.traits[0].custom_explanation);

        let p = std::env::temp_dir().join(format!("app-expl-{}.json", std::process::id()));
        s.save(Some(p.display().to_string())).unwrap();
        let Some(mut s2) = state() else {
            return;
        };
        let v = s2.load(&p.display().to_string()).unwrap();
        assert_eq!(
            v.traits[0].explanation.as_deref(),
            Some("**Своё** пояснение")
        );
        std::fs::remove_file(p).unwrap();

        let v = s2.set_trait_explanation(0, Some("  ".into())).unwrap();
        assert_eq!(v.traits[0].explanation, default);
        assert!(!v.traits[0].custom_explanation);
        assert!(s2.set_trait_explanation(9, None).is_err());
    }

    #[test]
    fn localization_switch() {
        let Some(mut s) = state() else {
            return;
        };
        assert!(s.locale_info().available.contains(&"ru".to_string()));
        let v = s.add_trait("claws-a", None).unwrap();
        assert_ne!(v.traits[0].name, "Claws A");
        assert_eq!(v.traits[0].original_name.as_deref(), Some("Claws A"));
        let lib = s.library_view();
        let e = lib.iter().find(|e| e.def.id == "claws-a").unwrap();
        assert!(e.translated_name.is_some());
        assert_eq!(e.def.name, "Claws A");
        assert_eq!(v.sheet.row(Stat::ST).unwrap().label, "Сила");

        let v = s.set_locale("ru".into(), true);
        assert_eq!(v.traits[0].name, "Claws A");
        assert_eq!(v.traits[0].original_name, None);
        assert_eq!(
            v.sheet.row(Stat::ST).unwrap().label,
            "ST",
            "без переводов - код"
        );
        assert!(s.library_view().iter().all(|e| e.translated_name.is_none()));

        let v = s.set_locale("en".into(), false);
        assert_eq!(v.sheet.row(Stat::ST).unwrap().label, "Strength");
        s.set_locale("ru".into(), false);
        // user override wins over the translation
        let v = s.set_trait_explanation(0, Some("своё".into())).unwrap();
        assert_eq!(v.traits[0].explanation.as_deref(), Some("своё"));
    }

    #[test]
    fn plugin_status_and_reload() {
        let Some(mut s) = state() else {
            return;
        };
        let st = s.plugin_status();
        assert!(st.errors.is_empty(), "{:?}", st.errors);
        assert!(st.plugins.iter().any(|p| p.starts_with("basic")));
        assert!(st.traits >= 10);
        s.reload_plugins();
        assert_eq!(s.plugin_status().traits, st.traits);
        let bad = AppState::with_source(None);
        assert!(!bad.plugin_status().errors.is_empty());
    }

    #[test]
    fn equipment_flow_translated() {
        let Some(mut s) = state() else {
            return;
        };
        let lib = s.equipment_library_view();
        assert!(lib.len() >= 5);
        s.add_equipment("bag-a", None, None).unwrap();
        s.add_equipment("sword-item-a", Some(2), Some(vec![0]))
            .unwrap();
        assert!(
            s.add_equipment("knuckles-a", None, Some(vec![0, 0]))
                .is_err(),
            "sword is no container"
        );
        let v = s.add_equipment("shield-a", None, None).unwrap();
        assert_eq!(v.equipment.len(), 3);
        assert_eq!(
            v.equipment[1].original_name.as_deref(),
            Some("Sword item A")
        );
        assert_eq!(
            (v.equipment[1].depth, v.equipment[1].path.clone()),
            (1, vec![0, 0])
        );
        assert_eq!(v.equipment[0].total_weight, 3.0);
        assert_eq!(v.sheet.encumbrance.carried_weight, 4.0);
        assert_eq!(v.sheet.encumbrance.total_weight, 4.0);
        assert_eq!(v.sheet.encumbrance.level, 0);
        assert!(!v.sheet.encumbrance.name.is_empty());
        let v = s.set_equipment_carried(&[0], false).unwrap();
        assert!(!v.equipment[1].effectively_carried && v.equipment[1].carried);
        assert_eq!(v.sheet.encumbrance.carried_weight, 1.0);
        let v = s.set_equipment_quantity(&[0, 0], 5).unwrap();
        assert_eq!(v.equipment[1].total_weight, 5.0);
        let v = s
            .set_equipment_explanation(&[1], Some("своё".into()))
            .unwrap();
        assert!(v.equipment[2].custom_explanation);
        assert_eq!(v.equipment[2].explanation.as_deref(), Some("своё"));
        let v = s.remove_equipment(&[0, 0]).unwrap();
        assert_eq!(v.equipment.len(), 2);
        assert!(s.remove_equipment(&[5]).is_err());
    }

    #[test]
    fn skills_flow_translated() {
        let Some(mut s) = state() else {
            return;
        };
        s.set_attribute("DX", 2).unwrap();
        let lib = s.skill_library_view();
        assert!(lib.skills.len() >= 5);
        assert!(lib.techniques.iter().any(|t| t.def.id == "strike-a"));
        s.add_skill("unarmed-a", None, Some(1)).unwrap();
        s.add_skill("outdoor-a", Some("spec_a".into()), Some(2))
            .unwrap();
        s.add_trait("talent-a", Some(1)).unwrap();
        let v = s.add_technique("strike-a", Some(2)).unwrap();
        assert!(
            v.issues.iter().all(|i| i.severity != "error"),
            "{:?}",
            v.issues
        );
        // DX 12, E 1 оч. = DX
        assert_eq!(v.skills[0].level, Some(12));
        assert_eq!(v.skills[0].relative.as_deref(), Some("DX"));
        // Per 10, A 2 оч. = Per, +1 талант
        assert_eq!(v.skills[1].level, Some(11));
        assert_eq!(
            v.skills[1].original_name.as_deref(),
            Some("Outdoor A (spec_a)")
        );
        // приём: от навыка -2, A 2 оч. -> +2, предел = навык (max_mod 0)
        assert_eq!(v.techniques[0].level, Some(12));
        assert_eq!(v.sheet.points.skills, 5);
        let v = s.set_skill_explanation(1, Some("своё".into())).unwrap();
        assert!(v.skills[1].custom_explanation);
        let v = s.set_technique_explanation(0, Some("своё".into())).unwrap();
        assert_eq!(v.techniques[0].explanation.as_deref(), Some("своё"));
        let v = s.set_skill_specialization(1, Some(" ".into())).unwrap();
        assert!(v.issues.iter().any(|i| i.message.contains("специализация")));
        // file roundtrip keeps skills and the user's texts
        let p = std::env::temp_dir().join(format!("app-skills-{}.json", std::process::id()));
        s.save(Some(p.display().to_string())).unwrap();
        let Some(mut s0) = state() else {
            return;
        };
        let v = s0.load(&p.display().to_string()).unwrap();
        std::fs::remove_file(p).unwrap();
        assert_eq!(v.skills.len(), 2);
        assert_eq!(v.techniques[0].explanation.as_deref(), Some("своё"));
        assert!(s.remove_technique(3).is_err());
        assert!(s.add_skill("nope", None, None).is_err());
        assert!(s.set_skill_points(0, 4).unwrap().sheet.points.skills >= 4);
    }

    #[test]
    fn spells_and_modifiers_flow_translated() {
        let Some(mut s) = state() else {
            return;
        };
        assert_eq!(s.plugin_status().spells, 2);
        assert_eq!(s.plugin_status().modifiers, 1);
        s.set_attribute("IQ", 4).unwrap();
        s.add_trait("magery", Some(2)).unwrap();
        let v = s.add_spell("spell-a", None).unwrap();
        let sp = &v.spells[0];
        assert_eq!(sp.original_name.as_deref(), Some("Spell A"));
        // IQ 14, H 1 оч. = 12, +2 Магия 2 (заглушка: spell_bonus за уровень)
        assert_eq!(sp.level, Some(14));
        let v = s.set_spell_points(0, 4).unwrap();
        assert_eq!(v.spells[0].level, Some(16));
        assert_eq!(v.sheet.points.spells, 4);
        let v = s.set_spell_explanation(0, Some("своё".into())).unwrap();
        assert!(v.spells[0].custom_explanation);
        let v = s.set_spell_explanation(0, None).unwrap();
        assert!(!v.spells[0].custom_explanation);

        // magery (заглушка): 0 + 1/ур. = 2; limit_a -10%: 1.8 -> 2
        let v = s.add_trait_modifier(0, "limit-a").unwrap();
        let t = &v.traits[0];
        assert_eq!((t.base_cost, t.modifier_percent, t.cost), (2, -10, 2));
        assert!(t.cost_explain.last().unwrap().ends_with("= 1.8 -> 2"));
        let v = s.set_modifier_explanation(0, 0, Some("x".into())).unwrap();
        assert!(v.traits[0].modifiers[0].custom_explanation);
        let v = s.remove_trait_modifier(0, 0).unwrap();
        assert_eq!(v.traits[0].cost, 2);
        assert!(v.traits[0].cost_explain.is_empty());
        let v = s.remove_spell(0).unwrap();
        assert!(v.spells.is_empty());
        assert_eq!(s.modifier_library_view().len(), 1);
    }

    #[test]
    fn weapons_flow_translated() {
        let Some(mut s) = state() else {
            return;
        };
        let v = s.new_character(Some("W".into()));
        let ids: Vec<_> = v.weapons.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(
            ids,
            ["bite", "kick", "punch"],
            "natural attacks on a new character"
        );
        let punch = &v.weapons[2];
        assert!(punch.ready);
        assert_eq!(punch.attacks[0].row.level, Some(10), "DX 10 по умолчанию");
        s.add_skill("sword-a", None, Some(2)).unwrap();
        let v = s.add_equipment("sword-item-a", None, None).unwrap();
        let sw = v.weapons.iter().find(|w| w.id == "sword-item-a").unwrap();
        assert!(sw.ready);
        let modes: Vec<_> = sw.attacks.iter().map(|a| a.row.mode.as_str()).collect();
        assert_eq!(modes.len(), 2, "{modes:?}");
        assert_eq!(
            sw.attacks[0].row.level,
            Some(11),
            "homebrew sword_a (E) перекрывает basic: DX 10, 2 оч. = +1"
        );
        assert!(v.sheet.defenses.parry.is_some());
        assert!(s.add_weapon("sword-item-a").is_err());
        let i = sw.index;
        let v = s.set_weapon_explanation(i, Some("своё".into())).unwrap();
        assert!(v.weapons[i].custom_explanation);
        let v = s.remove_equipment(&[0]).unwrap();
        assert_eq!(v.weapons.len(), 3, "sword row goes with the sword");
        let v = s.remove_weapon(0).unwrap();
        assert_eq!(v.weapons.len(), 2);
        assert!(s.add_weapon("bite").is_ok());
    }
}
