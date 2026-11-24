//! UI-independent application state; all Tauri commands delegate here so the
//! logic can be unit-tested without a webview.
//!
//! The character stores catalog references `(pack, sid)` and inline definitions for
//! its own content; views resolve them through the loaded [`Library`].

use gurps_core::{
    modified_cost_with, Character, Def, Item, ItemDef, LocalizedText, Modifier, ModifierDef, Skill,
    SkillDef, SkillDefault, Spell, SpellDef, Src, Technique, TechniqueDef, Trait, TraitDef,
    ATTRIBUTES, CUSTOM_PACK,
};
use gurps_dice::{success_roll, RollResult, SplitMix};
use gurps_library::calc::{Calc, Issue};
use gurps_library::sheet::{attr_code, trait_name, Sheet};
use gurps_library::{effective, find, resolve, Library};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct TraitView {
    pub index: usize,
    /// Catalog sid (or own sid for inline content).
    pub id: String,
    /// Display name in the UI language (with level), falls back to the original.
    pub name: String,
    /// Original (catalog) display name when `name` is a translation.
    pub original_name: Option<String>,
    /// `adv` / `disadv` / `perk` / `quirk`.
    pub kind: String,
    pub level: i32,
    pub leveled: bool,
    pub min_level: u32,
    pub max_level: Option<u32>,
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
    /// Pack code (`custom` for own content).
    pub source: String,
    /// Effective explanation (user text, else catalog text), Markdown.
    pub explanation: Option<String>,
    /// Catalog text for the current locale (for "reset to default").
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
    pub source: String,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
}

/// Library entry: pack, the definition (flattened) and texts for the UI language.
#[derive(Debug, Clone, Serialize)]
pub struct LibEntry<D> {
    pub plugin: String,
    /// Same as `sid` (the UI keys entries by `id`).
    pub id: String,
    #[serde(flatten)]
    pub def: D,
    pub explanation: Option<String>,
    pub translated_name: Option<String>,
}

pub type LibraryEntry = LibEntry<TraitDef>;
pub type ModifierLibEntry = LibEntry<ModifierDef>;
pub type SpellLibEntry = LibEntry<SpellDef>;
pub type SkillLibEntry = LibEntry<SkillDef>;
pub type EquipmentLibEntry = LibEntry<ItemDef>;

#[derive(Debug, Clone, Serialize)]
pub struct SpellView {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub original_name: Option<String>,
    /// College names.
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
    pub source: String,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
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
    pub source: String,
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
    /// Parent skill sid.
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
    pub source: String,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TechniqueLibEntry {
    #[serde(flatten)]
    pub entry: LibEntry<TechniqueDef>,
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
    /// Short summary: DR, DB, attack modes.
    pub stats: Option<String>,
    pub page: Option<String>,
    pub source: String,
    pub explanation: Option<String>,
    pub default_explanation: Option<String>,
    pub custom_explanation: bool,
}

/// One attack row; `skill` is translated, `original_skill` keeps the catalog name.
#[derive(Debug, Clone, Serialize)]
pub struct AttackView {
    #[serde(flatten)]
    pub row: gurps_library::sheet::AttackRow,
    pub original_skill: Option<String>,
}

/// A weapon: an owned item with attack modes, or a natural attack (punch, kick, ...).
#[derive(Debug, Clone, Serialize)]
pub struct WeaponView {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub original_name: Option<String>,
    /// Item sid (None for natural attacks).
    pub equipment: Option<String>,
    pub available: bool,
    pub ready: bool,
    pub attacks: Vec<AttackView>,
    pub explanation: Option<String>,
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
    /// Natural attacks.
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
pub const CUSTOM: &str = CUSTOM_PACK;

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
    /// Catalog (all packs) plus the character's own definitions under pack `custom`.
    pub lib: Library,
    pub path: Option<PathBuf>,
    /// Where relative file names are resolved (the user's documents folder in the app).
    /// Never the process CWD: under `tauri dev` that is src-tauri/, and writing there
    /// makes the dev watcher rebuild and restart the app.
    pub save_dir: Option<PathBuf>,
    pub plugin_errors: Vec<String>,
    /// Where the catalog comes from (shared SurrealDB; tests use the testkit DB).
    pub source: Option<gurps_library::LibrarySource>,
    /// UI locale used to resolve localized catalog text (default "ru").
    pub locale: String,
    /// Show catalog originals only (no translations).
    pub original_only: bool,
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
            lib: Library::default(),
            path: None,
            save_dir: None,
            plugin_errors: Vec::new(),
            source: None,
            locale: DEFAULT_LOCALE.into(),
            original_only: false,
            undo_stack: Default::default(),
            redo_stack: Vec::new(),
            saved: Character::default(),
            rng: SplitMix::from_time(),
            engine: None,
        }
    }
}

fn walk_mut<'a>(list: &'a mut [Item], path: &[usize]) -> Option<&'a mut Item> {
    let (first, rest) = path.split_first()?;
    let it = list.get_mut(*first)?;
    if rest.is_empty() {
        Some(it)
    } else {
        walk_mut(&mut it.contents, rest)
    }
}

/// The list that holds the item at `path`.
fn parent_list_mut<'a>(list: &'a mut Vec<Item>, path: &[usize]) -> Option<&'a mut Vec<Item>> {
    match path.split_last()? {
        (_, []) => Some(list),
        (_, parent) => walk_mut(list, parent).map(|p| &mut p.contents),
    }
}

/// Library entry -> reference for the sheet (own content is copied inline).
fn src_of<D: Def + Clone>(pack: &str, d: &D) -> Src<D> {
    if pack == CUSTOM {
        Src::Inline { inline: d.clone() }
    } else {
        Src::catalog(pack, d.sid())
    }
}

fn signed(m: i64) -> String {
    if m >= 0 {
        format!("+{m}")
    } else {
        m.to_string()
    }
}

impl AppState {
    pub fn with_source(source: Option<gurps_library::LibrarySource>) -> Self {
        let mut s = Self {
            source,
            ..Self::default()
        };
        s.reload_plugins();
        s.saved = s.character.clone();
        s
    }

    /// (Re)reads the catalog from `source`; nothing is cached between runs.
    pub fn reload_plugins(&mut self) {
        self.lib = Library::default();
        self.plugin_errors.clear();
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
        };
        match Library::load(&src) {
            Ok(lib) => {
                for w in &lib.warnings {
                    eprintln!("gurps-sheet: catalog: {w}");
                }
                self.lib = lib;
            }
            Err(e) => self.plugin_errors.push(e),
        }
        self.sync_custom();
    }

    pub fn plugin_status(&self) -> PluginStatus {
        let n = |c: usize| c;
        PluginStatus {
            dir: self.source.as_ref().map(|s| s.describe()),
            plugins: self.lib.packs.clone(),
            errors: self.plugin_errors.clone(),
            traits: n(effective(&self.lib.traits).len()),
            skills: effective(&self.lib.skills).len() + effective(&self.lib.techniques).len(),
            equipment: effective(&self.lib.items).len(),
            weapons: self.lib.natural.len(),
            spells: effective(&self.lib.spells).len(),
            modifiers: effective(&self.lib.modifiers).len(),
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
            Ok(c) => gurps_library::sheet::build(&self.character, &self.lib, c, &|k, id| {
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
        let issues = gurps_library::sheet::issues(&self.character, &self.lib, &calc, &sheet);
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
            traits: self.trait_views(),
            sheet,
            issues,
            path: self.path.as_ref().map(|p| p.display().to_string()),
            budget: self.character.available_points(),
        }
    }

    pub fn new_character(&mut self, name: Option<String>) -> SheetView {
        self.character = Character::new(name.unwrap_or_else(|| "New character".into()));
        self.path = None;
        self.reset_history();
        self.view()
    }

    fn reset_history(&mut self) {
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
    /// current character: the added, removed or edited trait/skill/spell/item.
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
            ($field:ident, $kind:expr, $list:ident) => {
                if let Some((from_before, i)) = diff(&before.$field, &now.$field) {
                    let item = if from_before {
                        &before.$field[i]
                    } else {
                        &now.$field[i]
                    };
                    return Some(self.name_of($kind, &self.lib.$list, &item.src));
                }
            };
        }
        check!(traits, "trait", traits);
        check!(skills, "skill", skills);
        check!(techniques, "technique", techniques);
        check!(spells, "spell", spells);
        check!(items, "equipment", items);
        None
    }

    /// Name in the UI language: translation, else definition name, else sid.
    fn name_of<D: Def>(&self, kind: &str, list: &[(String, D)], src: &Src<D>) -> String {
        self.translated_kind(kind, src.sid())
            .map(str::to_owned)
            .or_else(|| resolve(list, src).map(|d| d.name().to_owned()))
            .unwrap_or_else(|| src.sid().to_owned())
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
            disadvantage_limit: ch.rules.disadvantage_limit.unwrap_or(t.starting / 2),
            disadvantage_limit_custom: ch.rules.disadvantage_limit.is_some(),
            disadvantages: -b.disadvantages,
            quirks: ch
                .traits
                .iter()
                .filter(|x| resolve(&self.lib.traits, &x.src).is_some_and(|d| d.kind == "quirk"))
                .count(),
            quirk_points: b.quirks,
        }
    }

    pub fn add_ledger_entry(&mut self, e: LedgerInput) -> Result<SheetView, String> {
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
        Ok(self.view())
    }

    pub fn update_ledger_entry(&mut self, id: u32, e: LedgerInput) -> Result<SheetView, String> {
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
        Ok(self.view())
    }

    pub fn remove_ledger_entry(&mut self, id: u32) -> Result<SheetView, String> {
        let n = self.character.ledger.len();
        self.character.ledger.retain(|x| x.id != id);
        if self.character.ledger.len() == n {
            return Err("no such ledger entry".into());
        }
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

    /// Puts the character's own definitions into the libraries (pack `custom`, last, so
    /// they win over catalog entries with the same sid).
    pub fn sync_custom(&mut self) {
        let c = self.character.custom.clone();
        fn merge<D: Clone>(lib: &mut Vec<(String, D)>, custom: &[D]) {
            lib.retain(|(p, _)| p != CUSTOM);
            lib.extend(custom.iter().map(|d| (CUSTOM.to_string(), d.clone())));
        }
        merge(&mut self.lib.traits, &c.traits);
        merge(&mut self.lib.skills, &c.skills);
        merge(&mut self.lib.spells, &c.spells);
        merge(&mut self.lib.items, &c.items);
    }

    fn new_custom_id(&self, kind: &str) -> String {
        let c = &self.character.custom;
        let used: Vec<&str> = c
            .traits
            .iter()
            .map(|d| d.sid.as_str())
            .chain(c.skills.iter().map(|d| d.sid.as_str()))
            .chain(c.spells.iter().map(|d| d.sid.as_str()))
            .chain(c.items.iter().map(|d| d.sid.as_str()))
            .collect();
        (1..)
            .map(|n| format!("custom_{kind}_{n}"))
            .find(|id| !used.contains(&id.as_str()))
            .unwrap()
    }

    /// Adds or replaces (by sid) a user-made definition. `kind`: trait | skill | spell | equipment.
    /// An empty sid creates a new entry. Inline copies on the sheet get the new definition.
    pub fn save_custom(&mut self, kind: &str, def: serde_json::Value) -> Result<SheetView, String> {
        fn parse<D: serde::de::DeserializeOwned>(v: serde_json::Value) -> Result<D, String> {
            serde_json::from_value(v).map_err(|e| format!("invalid definition: {e}"))
        }
        fn upsert<D: Def + Clone>(list: &mut Vec<D>, d: D) {
            match list.iter().position(|x| x.sid() == d.sid()) {
                Some(i) => list[i] = d,
                None => list.push(d),
            }
        }
        fn refresh<D: Def + Clone>(src: &mut Src<D>, d: &D) {
            if let Src::Inline { inline } = src {
                if inline.sid() == d.sid() {
                    *inline = d.clone();
                }
            }
        }
        if !["trait", "skill", "spell", "equipment"].contains(&kind) {
            return Err(format!("unknown custom kind '{kind}'"));
        }
        let mut v = def;
        let blank_id = v
            .get("sid")
            .and_then(|x| x.as_str())
            .is_none_or(|x| x.trim().is_empty());
        if blank_id {
            v["sid"] = self.new_custom_id(kind).into();
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
                ch.traits.iter_mut().for_each(|t| refresh(&mut t.src, &d));
                upsert(&mut ch.custom.traits, d);
            }
            "skill" => {
                let d: SkillDef = parse(v)?;
                ch.skills.iter_mut().for_each(|t| refresh(&mut t.src, &d));
                upsert(&mut ch.custom.skills, d);
            }
            "spell" => {
                let d: SpellDef = parse(v)?;
                ch.spells.iter_mut().for_each(|t| refresh(&mut t.src, &d));
                upsert(&mut ch.custom.spells, d);
            }
            _ => {
                let d: ItemDef = parse(v)?;
                fn all(list: &mut [Item], d: &ItemDef) {
                    for i in list {
                        refresh(&mut i.src, d);
                        all(&mut i.contents, d);
                    }
                }
                all(&mut ch.items, &d);
                upsert(&mut ch.custom.items, d);
            }
        }
        self.sync_custom();
        Ok(self.view())
    }

    /// Removes a definition from «Своё» (inline copies already on the sheet stay).
    pub fn remove_custom(&mut self, kind: &str, id: &str) -> Result<SheetView, String> {
        fn drop<D: Def>(list: &mut Vec<D>, id: &str) -> usize {
            let n = list.len();
            list.retain(|d| d.sid() != id);
            n - list.len()
        }
        let c = &mut self.character.custom;
        let n = match kind {
            "trait" => drop(&mut c.traits, id),
            "skill" => drop(&mut c.skills, id),
            "spell" => drop(&mut c.spells, id),
            "equipment" => drop(&mut c.items, id),
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
        let max = |k: &str| sheet.row(k).map(|r| r.total.floor() as i32).unwrap_or(10);
        let (hp_max, fp_max) = (max("HP"), max("FP"));
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
        Ok(self.view())
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

    /// Buys levels of an attribute by sheet key (`ST`, `BasicSpeed`, ...) or DB code (`Speed`).
    pub fn set_attribute(&mut self, key: &str, level: i32) -> Result<SheetView, String> {
        let code = attr_code(key);
        if !ATTRIBUTES.contains(&code) {
            return Err(format!("{key} cannot be bought directly"));
        }
        self.character.set_attribute(code, level);
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

    pub fn add_trait(&mut self, id: &str, levels: Option<i32>) -> Result<SheetView, String> {
        let (pack, d) =
            find(&self.lib.traits, id).ok_or_else(|| format!("trait {id} not in library"))?;
        let level = d
            .is_leveled()
            .then(|| levels.unwrap_or((d.min_level() as i32).max(1)));
        let t = Trait {
            src: src_of(pack, d),
            level,
            modifiers: Vec::new(),
            explanation: None,
        };
        self.character.traits.push(t);
        Ok(self.view())
    }

    pub fn set_trait_levels(&mut self, index: usize, levels: i32) -> Result<SheetView, String> {
        let t = self
            .character
            .traits
            .get_mut(index)
            .ok_or("no such trait")?;
        t.level = Some(levels);
        Ok(self.view())
    }

    pub fn remove_trait(&mut self, index: usize) -> Result<SheetView, String> {
        if index >= self.character.traits.len() {
            return Err("no such trait".into());
        }
        self.character.traits.remove(index);
        Ok(self.view())
    }

    /// Locale used for the catalog's own text.
    fn data_locale(&self) -> &str {
        if self.original_only {
            "en"
        } else {
            &self.locale
        }
    }

    fn translated_kind(&self, kind: &str, id: &str) -> Option<&str> {
        if self.original_only {
            return None;
        }
        self.lib.translations.name(&self.locale, kind, id)
    }

    /// Translated description, else the definition's own text.
    fn text_for(&self, kind: &str, id: &str, own: Option<&LocalizedText>) -> Option<String> {
        (!self.original_only)
            .then(|| self.lib.translations.description(&self.locale, kind, id))
            .flatten()
            .or_else(|| own.and_then(|d| d.get(self.data_locale())))
            .map(str::to_owned)
    }

    /// Catalog text of an entry for the UI language.
    fn default_text<D: Def>(
        &self,
        kind: &str,
        list: &[(String, D)],
        src: &Src<D>,
    ) -> Option<String> {
        self.text_for(
            kind,
            src.sid(),
            resolve(list, src).and_then(|d| d.description()),
        )
    }

    /// Skill name in the UI language (no specialization).
    fn skill_name(&self, id: &str) -> String {
        self.translated_kind("skill", id)
            .map(str::to_owned)
            .or_else(|| find(&self.lib.skills, id).map(|(_, d)| d.name.clone()))
            .or_else(|| {
                self.character
                    .skills
                    .iter()
                    .find(|s| s.src.sid() == id)
                    .and_then(|s| resolve(&self.lib.skills, &s.src))
                    .map(|d| d.name.clone())
            })
            .unwrap_or_else(|| id.to_owned())
    }

    fn default_line(&self, d: &SkillDefault) -> String {
        match (&d.attribute, &d.skill) {
            (Some(a), _) => format!("{a}{}", signed(d.modifier)),
            (None, Some(s)) => format!("{}{}", self.skill_name(s), signed(d.modifier)),
            _ => signed(d.modifier),
        }
    }

    fn lib_entry<D: Def + Clone>(&self, kind: &str, pack: &str, d: &D) -> LibEntry<D> {
        LibEntry {
            plugin: pack.to_owned(),
            id: d.sid().to_owned(),
            explanation: self.text_for(kind, d.sid(), d.description()),
            translated_name: self.translated_kind(kind, d.sid()).map(str::to_owned),
            def: d.clone(),
        }
    }

    fn lib_view<D: Def + Clone>(&self, kind: &str, list: &[(String, D)]) -> Vec<LibEntry<D>> {
        effective(list)
            .iter()
            .map(|(p, d)| self.lib_entry(kind, p, d))
            .collect()
    }

    fn trait_views(&self) -> Vec<TraitView> {
        let r = self.character.rules.negative_rounding;
        self.character
            .traits
            .iter()
            .enumerate()
            .map(|(index, t)| {
                let d = resolve(&self.lib.traits, &t.src);
                let sid = t.src.sid();
                let original = d.map_or_else(|| sid.to_owned(), |d| trait_name(d, t.level));
                let tr = self.translated_kind("trait", sid).map(|n| match t.level {
                    Some(l) if d.is_some_and(|d| d.is_leveled()) => format!("{n} {l}"),
                    _ => n.to_owned(),
                });
                let (base, pct, cost, explain) = self.trait_cost(t, r);
                let default_explanation = self.default_text("trait", &self.lib.traits, &t.src);
                TraitView {
                    index,
                    id: sid.to_owned(),
                    original_name: tr.as_ref().map(|_| original.clone()),
                    name: tr.unwrap_or(original),
                    kind: d.map(|d| d.kind.clone()).unwrap_or_default(),
                    level: t.level.unwrap_or(0),
                    leveled: d.is_some_and(|d| d.is_leveled()),
                    min_level: d.map_or(0, |d| d.min_level()),
                    max_level: d.and_then(|d| d.cost.max_level),
                    cost,
                    base_cost: base,
                    modifier_percent: pct,
                    modifiers: self.modifier_views(t),
                    cost_explain: explain,
                    page: d.and_then(|d| d.page.clone()),
                    source: t.src.pack().to_owned(),
                    explanation: gurps_core::non_blank(t.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: !gurps_core::is_blank(&t.explanation),
                }
            })
            .collect()
    }

    /// (base cost, modifier %, final cost, explanation lines) of a trait.
    fn trait_cost(
        &self,
        t: &Trait,
        r: gurps_core::NegativeRounding,
    ) -> (i32, i32, i32, Vec<String>) {
        use gurps_core::modifier::MIN_TOTAL_PERCENT;
        let base = resolve(&self.lib.traits, &t.src)
            .map_or(0, |d| d.base_cost(t.level.unwrap_or(0))) as i32;
        let mods: Vec<(i32, String)> = t
            .modifiers
            .iter()
            .map(|m| {
                let d = resolve(&self.lib.modifiers, &m.src);
                (
                    d.map_or(0, |d| d.percent()),
                    d.map_or_else(|| m.src.sid().to_owned(), |d| d.name.clone()),
                )
            })
            .collect();
        if mods.is_empty() {
            return (base, 0, base, Vec::new());
        }
        let total: i32 = mods.iter().map(|(p, _)| p).sum();
        let cost = modified_cost_with(base, total, r);
        let mut out = vec![format!("база {base}")];
        for (p, n) in &mods {
            out.push(format!("{} {n}", pct(*p)));
        }
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
            "{base} × {eff}% = {} -> {cost}",
            gurps_library::sheet::fmt_num(exact)
        ));
        (base, total, cost, out)
    }

    fn skill_views(&self, sheet: &Sheet) -> Vec<SkillView> {
        self.character
            .skills
            .iter()
            .zip(&sheet.skills)
            .map(|(s, r)| {
                let d = resolve(&self.lib.skills, &s.src);
                let sid = s.src.sid();
                let with_spec = |n: &str| match gurps_core::non_blank(s.spec.clone()) {
                    Some(sp) => format!("{n} ({})", sp.trim()),
                    None => n.to_owned(),
                };
                let original = with_spec(d.map_or(sid, |d| d.name.as_str()));
                let tr = self.translated_kind("skill", sid).map(with_spec);
                let default_explanation = self.default_text("skill", &self.lib.skills, &s.src);
                SkillView {
                    index: r.index,
                    id: sid.to_owned(),
                    original_name: tr.as_ref().map(|_| original.clone()),
                    name: tr.unwrap_or(original),
                    attribute: d.map(|d| d.attribute.clone()).unwrap_or_default(),
                    difficulty: d.map(|d| d.difficulty.clone()).unwrap_or_default(),
                    points: s.points,
                    level: r.level,
                    relative: r.relative.clone(),
                    from_default: r.from_default,
                    bonus: 0,
                    explain: r.explain.clone(),
                    defaults: d
                        .map(|d| d.defaults.iter().map(|x| self.default_line(x)).collect())
                        .unwrap_or_default(),
                    specialization: s.spec.clone(),
                    specialization_required: d.is_some_and(|d| d.spec_required),
                    specializations: d.map(|d| d.specializations.clone()).unwrap_or_default(),
                    page: d.and_then(|d| d.page.clone()),
                    source: s.src.pack().to_owned(),
                    explanation: gurps_core::non_blank(s.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: !gurps_core::is_blank(&s.explanation),
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
                let d = resolve(&self.lib.techniques, &t.src);
                let sid = t.src.sid();
                let parent = d.map(|d| d.parent_skill.clone()).unwrap_or_default();
                let sk = self.skill_name(&parent);
                let original = d.map_or_else(|| sid.to_owned(), |d| d.name.clone());
                let tr = self
                    .translated_kind("technique", sid)
                    .map(|n| format!("{n} ({sk})"));
                let max = d.and_then(|d| d.max_mod).unwrap_or(0) as i32;
                let default_explanation =
                    self.default_text("technique", &self.lib.techniques, &t.src);
                TechniqueView {
                    index: r.index,
                    id: sid.to_owned(),
                    original_name: tr.as_ref().map(|_| original.clone()),
                    name: tr.unwrap_or(original),
                    skill: parent,
                    difficulty: d.map(|d| d.difficulty.clone()).unwrap_or_default(),
                    modifier: d.map_or(0, |d| d.default_mod) as i32,
                    max,
                    points: t.points,
                    level: r.level,
                    relative: r
                        .level
                        .zip(r.max_level)
                        .map(|(l, cap)| format!("{sk}{}", signed((l - (cap - max)) as i64))),
                    max_level: r.max_level,
                    explain: r.explain.clone(),
                    page: d.and_then(|d| d.page.clone()),
                    source: t.src.pack().to_owned(),
                    explanation: gurps_core::non_blank(t.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: !gurps_core::is_blank(&t.explanation),
                }
            })
            .collect()
    }

    fn item_totals(&self, it: &Item) -> (f64, f64) {
        let d = resolve(&self.lib.items, &it.src);
        let q = it.qty as f64;
        let mut w = d.map_or(0.0, |d| d.weight) * q;
        let mut c = d.map_or(0.0, |d| d.cost) * q;
        for x in &it.contents {
            let (a, b) = self.item_totals(x);
            w += a;
            c += b;
        }
        (w, c)
    }

    fn item_stats(d: &ItemDef) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(dr) = d.armor.iter().filter_map(|l| l.dr).max() {
            parts.push(format!("DR {dr}"));
        }
        if let Some(s) = &d.shield {
            parts.push(format!("DB {}", s.db));
        }
        for m in &d.attack_modes {
            let dmg = &m.damage;
            let base = match (dmg.base.as_deref(), dmg.dice) {
                (Some(b), _) => b.to_owned(),
                (None, Some(n)) => format!("{n}d"),
                _ => String::new(),
            };
            let add = if dmg.add == 0 {
                String::new()
            } else {
                signed(dmg.add)
            };
            let ty = dmg.kind.clone().unwrap_or_default();
            parts.push(
                format!("{} {base}{add} {ty}", m.name.clone().unwrap_or_default())
                    .trim()
                    .to_owned(),
            );
        }
        (!parts.is_empty()).then(|| parts.join("; "))
    }

    fn equipment_views(&self) -> Vec<EquipmentView> {
        let mut out = Vec::new();
        self.walk_items(&self.character.items, &mut Vec::new(), true, &mut out);
        out
    }

    fn walk_items(
        &self,
        list: &[Item],
        path: &mut Vec<usize>,
        parent_carried: bool,
        out: &mut Vec<EquipmentView>,
    ) {
        for (i, e) in list.iter().enumerate() {
            path.push(i);
            let d = resolve(&self.lib.items, &e.src);
            let sid = e.src.sid();
            let original = d.map_or_else(|| sid.to_owned(), |d| d.name.clone());
            let tr = self.translated_kind("equipment", sid).map(str::to_owned);
            let carried = parent_carried && e.equipped;
            let (tw, tc) = self.item_totals(e);
            let default_explanation = self.default_text("equipment", &self.lib.items, &e.src);
            out.push(EquipmentView {
                path: path.clone(),
                depth: path.len() - 1,
                id: sid.to_owned(),
                original_name: tr.as_ref().map(|_| original.clone()),
                name: tr.unwrap_or(original),
                quantity: e.qty,
                weight: d.map_or(0.0, |d| d.weight),
                cost: d.map_or(0.0, |d| d.cost),
                total_weight: tw,
                total_cost: tc,
                carried: e.equipped,
                effectively_carried: carried,
                container: d.is_some_and(|d| d.container),
                stats: d.and_then(Self::item_stats),
                page: d.and_then(|d| d.page.clone()),
                source: e.src.pack().to_owned(),
                explanation: gurps_core::non_blank(e.explanation.clone())
                    .or_else(|| default_explanation.clone()),
                default_explanation,
                custom_explanation: !gurps_core::is_blank(&e.explanation),
            });
            self.walk_items(&e.contents, path, carried, out);
            path.pop();
        }
    }

    pub fn equipment_library_view(&self) -> Vec<EquipmentLibEntry> {
        self.lib_view("equipment", &self.lib.items)
    }

    /// Adds a library item at the top level or into the container at `parent`.
    pub fn add_equipment(
        &mut self,
        id: &str,
        quantity: Option<u32>,
        parent: Option<Vec<usize>>,
    ) -> Result<SheetView, String> {
        let (pack, d) =
            find(&self.lib.items, id).ok_or_else(|| format!("equipment {id} not in library"))?;
        let mut e = Item::new(src_of(pack, d));
        e.qty = quantity.unwrap_or(1).max(1);
        match parent.filter(|p| !p.is_empty()) {
            None => self.character.items.push(e),
            Some(p) => {
                let c = walk_mut(&mut self.character.items, &p).ok_or("no such container")?;
                let is_container = resolve(&self.lib.items, &c.src).is_some_and(|d| d.container);
                if !is_container {
                    return Err(format!("{} is not a container", c.src.sid()));
                }
                c.contents.push(e);
            }
        }
        Ok(self.view())
    }

    fn item_mut(&mut self, path: &[usize]) -> Result<&mut Item, String> {
        walk_mut(&mut self.character.items, path).ok_or_else(|| "no such item".into())
    }

    pub fn set_equipment_quantity(
        &mut self,
        path: &[usize],
        quantity: u32,
    ) -> Result<SheetView, String> {
        self.item_mut(path)?.qty = quantity.max(1);
        Ok(self.view())
    }

    pub fn set_equipment_carried(
        &mut self,
        path: &[usize],
        carried: bool,
    ) -> Result<SheetView, String> {
        self.item_mut(path)?.equipped = carried;
        Ok(self.view())
    }

    pub fn set_equipment_explanation(
        &mut self,
        path: &[usize],
        text: Option<String>,
    ) -> Result<SheetView, String> {
        self.item_mut(path)?.explanation = gurps_core::non_blank(text);
        Ok(self.view())
    }

    pub fn remove_equipment(&mut self, path: &[usize]) -> Result<SheetView, String> {
        let i = *path.last().ok_or("empty path")?;
        let list = parent_list_mut(&mut self.character.items, path)
            .filter(|l| i < l.len())
            .ok_or("no such item")?;
        list.remove(i);
        Ok(self.view())
    }

    fn weapon_views(&self, sheet: &Sheet) -> Vec<WeaponView> {
        let items = gurps_library::sheet::all_items(&self.character);
        sheet
            .weapons
            .iter()
            .map(|r| {
                let kind = if r.equipment.is_some() {
                    "equipment"
                } else {
                    "weapon"
                };
                let tr = self.translated_kind(kind, &r.id).map(str::to_owned);
                let item = r
                    .equipment
                    .as_deref()
                    .and_then(|sid| items.iter().find(|i| i.src.sid() == sid));
                let explanation = match item {
                    Some(i) => gurps_core::non_blank(i.explanation.clone())
                        .or_else(|| self.default_text("equipment", &self.lib.items, &i.src)),
                    None => (!self.original_only)
                        .then(|| {
                            self.lib
                                .translations
                                .description(&self.locale, "weapon", &r.id)
                        })
                        .flatten()
                        .map(str::to_owned),
                };
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
                            if row.skill_id.is_some() && t.is_none() {
                                row.skill = self.skill_name(&row.skill);
                            }
                            let original_skill =
                                t.map(|t| std::mem::replace(&mut row.skill, t.to_owned()));
                            AttackView {
                                row,
                                original_skill,
                            }
                        })
                        .collect(),
                    explanation,
                }
            })
            .collect()
    }

    fn modifier_views(&self, t: &Trait) -> Vec<ModifierView> {
        t.modifiers
            .iter()
            .enumerate()
            .map(|(index, m)| {
                let d = resolve(&self.lib.modifiers, &m.src);
                let sid = m.src.sid();
                let original = d.map_or_else(|| sid.to_owned(), |d| d.name.clone());
                let tr = self.translated_kind("modifier", sid);
                let default_explanation =
                    self.default_text("modifier", &self.lib.modifiers, &m.src);
                ModifierView {
                    index,
                    id: sid.to_owned(),
                    original_name: tr.map(|_| original.clone()),
                    name: tr.map(str::to_owned).unwrap_or(original),
                    cost_percent: d.map_or(0, |d| d.percent()),
                    page: d.and_then(|d| d.page.clone()),
                    source: m.src.pack().to_owned(),
                    explanation: gurps_core::non_blank(m.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: !gurps_core::is_blank(&m.explanation),
                }
            })
            .collect()
    }

    pub fn modifier_library_view(&self) -> Vec<ModifierLibEntry> {
        self.lib_view("modifier", &self.lib.modifiers)
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
        let (pack, d) =
            find(&self.lib.modifiers, id).ok_or_else(|| format!("modifier {id} not in library"))?;
        let m = Modifier {
            src: src_of(pack, d),
            explanation: None,
        };
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
                let d = resolve(&self.lib.spells, &s.src);
                let sid = s.src.sid();
                let original = d.map_or_else(|| sid.to_owned(), |d| d.name.clone());
                let tr = self.translated_kind("spell", sid);
                let default_explanation = self.default_text("spell", &self.lib.spells, &s.src);
                SpellView {
                    index: r.index,
                    id: sid.to_owned(),
                    original_name: tr.map(|_| original.clone()),
                    name: tr.map(str::to_owned).unwrap_or(original),
                    college: d
                        .map(|d| {
                            d.colleges
                                .iter()
                                .map(|c| {
                                    self.lib
                                        .colleges
                                        .get(c)
                                        .cloned()
                                        .unwrap_or_else(|| c.clone())
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                    attribute: "IQ".into(),
                    difficulty: d.map(|d| d.difficulty.clone()).unwrap_or_default(),
                    points: s.points,
                    level: r.level,
                    relative: r.relative.clone(),
                    bonus: 0,
                    prerequisites: d
                        .filter(|d| !d.prereqs.is_null())
                        .map(|d| d.prereqs.to_string()),
                    explain: r.explain.clone(),
                    page: d.and_then(|d| d.page.clone()),
                    source: s.src.pack().to_owned(),
                    explanation: gurps_core::non_blank(s.explanation.clone())
                        .or_else(|| default_explanation.clone()),
                    default_explanation,
                    custom_explanation: !gurps_core::is_blank(&s.explanation),
                }
            })
            .collect()
    }

    pub fn spell_library_view(&self) -> Vec<SpellLibEntry> {
        self.lib_view("spell", &self.lib.spells)
    }

    pub fn add_spell(&mut self, id: &str, points: Option<i32>) -> Result<SheetView, String> {
        let (pack, d) =
            find(&self.lib.spells, id).ok_or_else(|| format!("spell {id} not in library"))?;
        let s = Spell {
            src: src_of(pack, d),
            points: points.unwrap_or(1).max(0),
            explanation: None,
        };
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
            skills: self.lib_view("skill", &self.lib.skills),
            techniques: effective(&self.lib.techniques)
                .iter()
                .map(|(p, d)| TechniqueLibEntry {
                    entry: self.lib_entry("technique", p, d),
                    skill_name: self.skill_name(&d.parent_skill),
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
        let (pack, d) =
            find(&self.lib.skills, id).ok_or_else(|| format!("skill {id} not in library"))?;
        let s = Skill {
            src: src_of(pack, d),
            points: points.unwrap_or(1).max(0),
            spec: gurps_core::non_blank(specialization),
            explanation: None,
        };
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
        self.skill_mut(index)?.spec = gurps_core::non_blank(specialization);
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
        let (pack, d) = find(&self.lib.techniques, id)
            .ok_or_else(|| format!("technique {id} not in library"))?;
        let t = Technique {
            src: src_of(pack, d),
            points: points.unwrap_or(0).max(0),
            explanation: None,
        };
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

    /// Trait library with names/explanations resolved for the current language.
    pub fn library_view(&self) -> Vec<LibraryEntry> {
        self.lib_view("trait", &self.lib.traits)
    }

    pub fn locale_info(&self) -> LocaleInfo {
        let mut available = self.lib.translations.languages();
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

    /// Sets (or with `None`/blank text, resets to the catalog default) a trait's explanation.
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
        let v = s.add_trait("reflexes_a", None).unwrap();
        assert!(
            v.issues.iter().all(|i| i.severity != "error"),
            "{:?}",
            v.issues
        );
        let st = v.sheet.row("ST").unwrap();
        assert_eq!((st.total, st.points, st.level), (12.0, 20, 2));
        assert_eq!(st.value.contributions[0].amount, 2.0);
        assert_eq!(st.label, "Сила", "подпись из перевода БД");
        // reflexes_a: +1 Per (заглушка сидов)
        let per = v.sheet.row("Per").unwrap();
        assert_eq!((per.value.base, per.total), (10.0, 11.0));
        assert_eq!(per.value.contributions[0].amount, 1.0);
        assert!(
            per.explain.iter().any(|e| e.contains("черта")),
            "{:?}",
            per.explain
        );
        assert_eq!(v.sheet.row("HP").unwrap().total, 12.0);
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
            s.add_trait("reflexes_a", None)
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
    fn v1_files_are_rejected() {
        let Some(mut s) = state() else {
            return;
        };
        let path = std::env::temp_dir().join(format!("gurps-v1-{}.json", std::process::id()));
        std::fs::write(
            &path,
            r#"{"format":"gurps-sheet","version":1,"character":{"name":"Old"}}"#,
        )
        .unwrap();
        assert!(s.load(&path.display().to_string()).is_err());
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn custom_definitions() {
        let Some(mut s) = state() else {
            return;
        };
        s.new_character(Some("Маг".into()));
        let spell = serde_json::json!({"name": "Искра", "difficulty": "VH", "colleges": ["fire"],
            "description": "**жжёт**"});
        let v = s.record("своё", |s| s.save_custom("spell", spell)).unwrap();
        let id = v.custom.spells[0].sid.clone();
        assert_eq!(id, "custom_spell_1");
        let lib = s.spell_library_view();
        let e = lib.iter().find(|e| e.id == id).unwrap();
        assert_eq!(e.plugin, CUSTOM);
        assert_eq!(e.explanation.as_deref(), Some("**жжёт**"));
        s.add_spell(&id, Some(4)).unwrap();
        // editing renames the sheet copy too
        s.save_custom(
            "spell",
            serde_json::json!({"sid": id, "name": "Искра!", "difficulty": "H"}),
        )
        .unwrap();
        assert_eq!(s.character.spells[0].src.inline().unwrap().name, "Искра!");
        assert!(s
            .save_custom("spell", serde_json::json!({"name": " "}))
            .is_err());
        assert!(s
            .save_custom("weapon", serde_json::json!({"name": "x"}))
            .is_err());
        s.save_custom(
            "trait",
            serde_json::json!({"name": "Везучесть в картах", "kind": "adv", "cost": {"type": "fixed", "base": 3}}),
        )
        .unwrap();
        s.add_trait("custom_trait_1", None).unwrap();
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
        assert!(s.spell_library_view().iter().all(|e| e.id != id));
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
        s.add_trait("temper_a", None).unwrap();
        // -1 × 90% = -0.9: вверх -> 0, от нуля -> -1; расчёт (rule_overrides) согласован
        let v = s.add_trait_modifier(0, "limit_a").unwrap();
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
        let v = s.add_trait("reflexes_a", None).unwrap();
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
        assert!(s.locale_info().available.iter().any(|l| l == "ru"));
        let v = s.add_trait("claws_a", None).unwrap();
        assert_ne!(v.traits[0].name, "Claws A");
        assert_eq!(v.traits[0].original_name.as_deref(), Some("Claws A"));
        let lib = s.library_view();
        let e = lib.iter().find(|e| e.id == "claws_a").unwrap();
        assert!(e.translated_name.is_some());
        assert_eq!(e.def.name, "Claws A");
        assert_eq!(v.sheet.row("ST").unwrap().label, "Сила");

        let v = s.set_locale("ru".into(), true);
        assert_eq!(v.traits[0].name, "Claws A");
        assert_eq!(v.traits[0].original_name, None);
        assert_eq!(
            v.sheet.row("ST").unwrap().label,
            "ST",
            "без переводов - код"
        );
        assert!(s.library_view().iter().all(|e| e.translated_name.is_none()));

        let v = s.set_locale("en".into(), false);
        assert_eq!(v.sheet.row("ST").unwrap().label, "Strength");
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
        s.add_equipment("bag_a", None, None).unwrap();
        s.add_equipment("sword_item_a", Some(2), Some(vec![0]))
            .unwrap();
        assert!(
            s.add_equipment("knuckles_a", None, Some(vec![0, 0]))
                .is_err(),
            "sword is no container"
        );
        let v = s.add_equipment("shield_a", None, None).unwrap();
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
        assert!(lib.techniques.iter().any(|t| t.entry.id == "strike_a"));
        s.add_skill("unarmed_a", None, Some(1)).unwrap();
        s.add_skill("outdoor_a", Some("spec_a".into()), Some(2))
            .unwrap();
        s.add_trait("talent_a", Some(1)).unwrap();
        let v = s.add_technique("strike_a", Some(2)).unwrap();
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
        let v = s.add_spell("spell_a", None).unwrap();
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
        let v = s.add_trait_modifier(0, "limit_a").unwrap();
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
        let mut ids: Vec<_> = v.weapons.iter().map(|w| w.id.as_str()).collect();
        ids.sort();
        assert_eq!(
            ids,
            ["bite", "kick", "punch"],
            "natural attacks on a new character"
        );
        let punch = v.weapons.iter().find(|w| w.id == "punch").unwrap();
        assert!(punch.ready);
        assert_eq!(punch.attacks[0].row.level, Some(10), "DX 10 по умолчанию");
        s.add_skill("sword_a", None, Some(2)).unwrap();
        let v = s.add_equipment("sword_item_a", None, None).unwrap();
        let sw = v.weapons.iter().find(|w| w.id == "sword_item_a").unwrap();
        assert!(sw.ready);
        let modes: Vec<_> = sw.attacks.iter().map(|a| a.row.mode.as_str()).collect();
        assert_eq!(modes.len(), 2, "{modes:?}");
        assert_eq!(
            sw.attacks[0].row.level,
            Some(11),
            "homebrew sword_a (E) перекрывает basic: DX 10, 2 оч. = +1"
        );
        assert!(v.sheet.defenses.parry.is_some());
        assert_eq!(sw.equipment.as_deref(), Some("sword_item_a"));
        let v = s
            .set_equipment_explanation(&[0], Some("своё".into()))
            .unwrap();
        let sw = v.weapons.iter().find(|w| w.id == "sword_item_a").unwrap();
        assert_eq!(sw.explanation.as_deref(), Some("своё"));
        let v = s.remove_equipment(&[0]).unwrap();
        assert_eq!(v.weapons.len(), 3, "sword row goes with the sword");
    }
}
