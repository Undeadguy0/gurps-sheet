mod state;

use gurps_rules::dice::{DamageRoll, RollResult};
pub use state::{AppState, SheetView};
use state::{
    EquipmentLibEntry, LibraryEntry, LocaleInfo, ModifierLibEntry, PluginStatus, SkillLibrary,
    SpellLibEntry, WeaponLibEntry,
};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

type S<'a> = State<'a, Mutex<AppState>>;

fn lock<'a>(s: &'a S) -> std::sync::MutexGuard<'a, AppState> {
    s.lock().unwrap_or_else(|e| e.into_inner())
}

/// Runs an undoable character edit and returns the fresh view (with history state).
fn edit(
    state: &S,
    label: impl Into<String>,
    f: impl FnOnce(&mut AppState) -> Result<SheetView, String>,
) -> Result<SheetView, String> {
    let mut st = lock(state);
    st.record(label, f)?;
    Ok(st.view())
}

#[tauri::command]
fn new_character(state: S, name: Option<String>) -> SheetView {
    lock(&state).new_character(name)
}

#[tauri::command]
fn load(state: S, path: String) -> Result<SheetView, String> {
    lock(&state).load(&path)
}

#[tauri::command]
fn save(state: S, path: Option<String>) -> Result<String, String> {
    lock(&state).save(path)
}

#[tauri::command]
fn set_negative_rounding(state: S, mode: String) -> Result<SheetView, String> {
    edit(&state, "правило округления", |s| {
        s.set_negative_rounding(&mode)
    })
}

#[tauri::command]
fn set_attribute(state: S, key: String, level: i32) -> Result<SheetView, String> {
    edit(&state, format!("изменение {key}"), |s| {
        s.set_attribute(&key, level)
    })
}

#[tauri::command]
fn add_ledger_entry(state: S, entry: state::LedgerInput) -> Result<SheetView, String> {
    edit(&state, "запись в журнале очков", |s| {
        s.add_ledger_entry(entry)
    })
}

#[tauri::command]
fn update_ledger_entry(state: S, id: u32, entry: state::LedgerInput) -> Result<SheetView, String> {
    edit(
        &state,
        "правка записи журнала очков",
        |s| s.update_ledger_entry(id, entry),
    )
}

#[tauri::command]
fn remove_ledger_entry(state: S, id: u32) -> Result<SheetView, String> {
    edit(
        &state,
        "удаление записи журнала очков",
        |s| s.remove_ledger_entry(id),
    )
}

#[tauri::command]
fn set_disadvantage_limit(state: S, limit: Option<i32>) -> Result<SheetView, String> {
    edit(&state, "лимит недостатков", |s| {
        s.set_disadvantage_limit(limit)
    })
}

#[tauri::command]
fn save_custom(state: S, kind: String, def: serde_json::Value) -> Result<SheetView, String> {
    edit(&state, "своё определение", |s| {
        s.save_custom(&kind, def)
    })
}

#[tauri::command]
fn remove_custom(state: S, kind: String, id: String) -> Result<SheetView, String> {
    edit(
        &state,
        "удаление своего определения",
        |s| s.remove_custom(&kind, &id),
    )
}

#[tauri::command]
fn export_custom_plugin(state: S) -> Result<String, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    s.export_custom_plugin(None)
}

#[tauri::command]
fn set_notes(state: S, text: String) -> Result<SheetView, String> {
    edit(&state, "заметки", |s| Ok(s.set_notes(text)))
}

#[tauri::command]
fn set_name(state: S, name: String) -> Result<SheetView, String> {
    edit(&state, "переименование", |s| {
        Ok(s.set_name(name))
    })
}

#[tauri::command]
fn add_trait(state: S, id: String, levels: Option<u32>) -> Result<SheetView, String> {
    edit(&state, "добавление черты", |s| {
        s.add_trait(&id, levels)
    })
}

#[tauri::command]
fn set_trait_levels(state: S, index: usize, levels: u32) -> Result<SheetView, String> {
    edit(&state, "уровень черты", |s| {
        s.set_trait_levels(index, levels)
    })
}

#[tauri::command]
fn remove_trait(state: S, index: usize) -> Result<SheetView, String> {
    edit(&state, "удаление черты", |s| {
        s.remove_trait(index)
    })
}

#[tauri::command]
fn set_trait_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "пояснение черты", |s| {
        s.set_trait_explanation(index, text)
    })
}

#[tauri::command]
fn get_locales(state: S) -> LocaleInfo {
    lock(&state).locale_info()
}

#[tauri::command]
fn set_locale(state: S, locale: String, original_only: bool) -> SheetView {
    lock(&state).set_locale(locale, original_only)
}

#[tauri::command]
fn get_sheet(state: S) -> SheetView {
    lock(&state).view()
}

#[tauri::command]
fn list_library(state: S) -> Vec<LibraryEntry> {
    lock(&state).library_view()
}

#[tauri::command]
fn list_skill_library(state: S) -> SkillLibrary {
    lock(&state).skill_library_view()
}

#[tauri::command]
fn add_skill(
    state: S,
    id: String,
    specialization: Option<String>,
    points: Option<i32>,
) -> Result<SheetView, String> {
    edit(&state, "добавление навыка", |s| {
        s.add_skill(&id, specialization, points)
    })
}

#[tauri::command]
fn set_skill_points(state: S, index: usize, points: i32) -> Result<SheetView, String> {
    edit(&state, "очки навыка", |s| {
        s.set_skill_points(index, points)
    })
}

#[tauri::command]
fn set_skill_specialization(
    state: S,
    index: usize,
    specialization: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "специализация навыка", |s| {
        s.set_skill_specialization(index, specialization)
    })
}

#[tauri::command]
fn set_skill_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "пояснение навыка", |s| {
        s.set_skill_explanation(index, text)
    })
}

#[tauri::command]
fn remove_skill(state: S, index: usize) -> Result<SheetView, String> {
    edit(&state, "удаление навыка", |s| {
        s.remove_skill(index)
    })
}

#[tauri::command]
fn add_technique(state: S, id: String, points: Option<i32>) -> Result<SheetView, String> {
    edit(&state, "добавление приёма", |s| {
        s.add_technique(&id, points)
    })
}

#[tauri::command]
fn set_technique_points(state: S, index: usize, points: i32) -> Result<SheetView, String> {
    edit(&state, "очки приёма", |s| {
        s.set_technique_points(index, points)
    })
}

#[tauri::command]
fn set_technique_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "пояснение приёма", |s| {
        s.set_technique_explanation(index, text)
    })
}

#[tauri::command]
fn remove_technique(state: S, index: usize) -> Result<SheetView, String> {
    edit(&state, "удаление приёма", |s| {
        s.remove_technique(index)
    })
}

#[tauri::command]
fn list_equipment_library(state: S) -> Vec<EquipmentLibEntry> {
    lock(&state).equipment_library_view()
}

#[tauri::command]
fn add_equipment(
    state: S,
    id: String,
    quantity: Option<u32>,
    parent: Option<Vec<usize>>,
) -> Result<SheetView, String> {
    edit(&state, "добавление снаряжения", |s| {
        s.add_equipment(&id, quantity, parent)
    })
}

#[tauri::command]
fn set_equipment_quantity(state: S, path: Vec<usize>, quantity: u32) -> Result<SheetView, String> {
    edit(&state, "количество снаряжения", |s| {
        s.set_equipment_quantity(&path, quantity)
    })
}

#[tauri::command]
fn set_equipment_carried(state: S, path: Vec<usize>, carried: bool) -> Result<SheetView, String> {
    edit(&state, "при себе / оставлено", |s| {
        s.set_equipment_carried(&path, carried)
    })
}

#[tauri::command]
fn set_equipment_explanation(
    state: S,
    path: Vec<usize>,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "пояснение снаряжения", |s| {
        s.set_equipment_explanation(&path, text)
    })
}

#[tauri::command]
fn remove_equipment(state: S, path: Vec<usize>) -> Result<SheetView, String> {
    edit(&state, "удаление снаряжения", |s| {
        s.remove_equipment(&path)
    })
}

#[tauri::command]
fn list_weapon_library(state: S) -> Vec<WeaponLibEntry> {
    lock(&state).weapon_library_view()
}

#[tauri::command]
fn add_weapon(state: S, id: String) -> Result<SheetView, String> {
    edit(&state, "добавление оружия", |s| {
        s.add_weapon(&id)
    })
}

#[tauri::command]
fn remove_weapon(state: S, index: usize) -> Result<SheetView, String> {
    edit(&state, "удаление оружия", |s| {
        s.remove_weapon(index)
    })
}

#[tauri::command]
fn set_weapon_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "пояснение оружия", |s| {
        s.set_weapon_explanation(index, text)
    })
}

#[tauri::command]
fn list_spell_library(state: S) -> Vec<SpellLibEntry> {
    lock(&state).spell_library_view()
}

#[tauri::command]
fn add_spell(state: S, id: String, points: Option<i32>) -> Result<SheetView, String> {
    edit(&state, "добавление заклинания", |s| {
        s.add_spell(&id, points)
    })
}

#[tauri::command]
fn set_spell_points(state: S, index: usize, points: i32) -> Result<SheetView, String> {
    edit(&state, "очки заклинания", |s| {
        s.set_spell_points(index, points)
    })
}

#[tauri::command]
fn set_spell_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "пояснение заклинания", |s| {
        s.set_spell_explanation(index, text)
    })
}

#[tauri::command]
fn remove_spell(state: S, index: usize) -> Result<SheetView, String> {
    edit(&state, "удаление заклинания", |s| {
        s.remove_spell(index)
    })
}

#[tauri::command]
fn list_modifier_library(state: S) -> Vec<ModifierLibEntry> {
    lock(&state).modifier_library_view()
}

#[tauri::command]
fn add_trait_modifier(state: S, trait_index: usize, id: String) -> Result<SheetView, String> {
    edit(
        &state,
        "добавление модификатора",
        |s| s.add_trait_modifier(trait_index, &id),
    )
}

#[tauri::command]
fn remove_trait_modifier(state: S, trait_index: usize, index: usize) -> Result<SheetView, String> {
    edit(&state, "удаление модификатора", |s| {
        s.remove_trait_modifier(trait_index, index)
    })
}

#[tauri::command]
fn set_modifier_explanation(
    state: S,
    trait_index: usize,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "пояснение модификатора", |s| {
        s.set_modifier_explanation(trait_index, index, text)
    })
}

#[tauri::command]
fn roll(state: S, target: i32, modifier: Option<i32>, seed: Option<u64>) -> RollResult {
    lock(&state).roll(target, modifier.unwrap_or(0), seed)
}

#[tauri::command]
fn roll_damage(
    state: S,
    expr: String,
    crushing: Option<bool>,
    seed: Option<u64>,
) -> Result<DamageRoll, String> {
    lock(&state).roll_damage(&expr, crushing.unwrap_or(false), seed)
}

#[tauri::command]
fn undo(state: S) -> Result<SheetView, String> {
    lock(&state).undo()
}

#[tauri::command]
fn redo(state: S) -> Result<SheetView, String> {
    lock(&state).redo()
}

/// Current HP/FP at the table; `value: null` resets to maximum. Undoable.
#[tauri::command]
fn set_current(state: S, stat: String, value: Option<i32>) -> Result<SheetView, String> {
    let label = match (stat.as_str(), value) {
        ("HP", None) | ("FP", None) => format!("сброс {stat}"),
        _ => format!("текущие {stat}"),
    };
    edit(&state, label, |s| s.set_current(&stat, value))
}

/// Plugin directory candidates, in priority order:
/// 1. `$GURPS_PLUGINS`;
/// 2. `plugins/` next to the executable (installed/portable app);
/// 3. debug builds only: the repo's `plugins/` (from `CARGO_MANIFEST_DIR`), so
///    `cargo tauri dev` / `npm run tauri dev` always load the working-tree data
///    regardless of the current directory;
/// 4. `./plugins`, `../plugins` relative to the current directory.
fn plugin_dir_candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(p) = std::env::var("GURPS_PLUGINS") {
        v.push(PathBuf::from(p));
    }
    if let Some(d) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.join("plugins")))
    {
        v.push(d);
    }
    if cfg!(debug_assertions) {
        v.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../plugins"));
    }
    v.extend(["plugins", "../plugins"].iter().map(PathBuf::from));
    v
}

fn plugin_dir() -> Option<PathBuf> {
    let dir = plugin_dir_candidates()
        .into_iter()
        .find(|p| p.is_dir())
        .map(|p| p.canonicalize().unwrap_or(p));
    match &dir {
        Some(d) => eprintln!("gurps-sheet: loading plugins from {}", d.display()),
        None => eprintln!("gurps-sheet: no plugins directory found (set GURPS_PLUGINS)"),
    }
    dir
}

#[tauri::command]
fn plugin_status(state: S) -> PluginStatus {
    lock(&state).plugin_status()
}

#[tauri::command]
fn reload_plugins(state: S) -> PluginStatus {
    let mut st = lock(&state);
    st.reload_plugins();
    st.plugin_status()
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(AppState::with_plugins(plugin_dir())))
        .invoke_handler(tauri::generate_handler![
            new_character,
            load,
            save,
            set_attribute,
            set_negative_rounding,
            set_name,
            set_notes,
            add_ledger_entry,
            update_ledger_entry,
            remove_ledger_entry,
            set_disadvantage_limit,
            save_custom,
            remove_custom,
            export_custom_plugin,
            add_trait,
            set_trait_levels,
            remove_trait,
            set_trait_explanation,
            get_locales,
            set_locale,
            get_sheet,
            list_library,
            list_skill_library,
            add_skill,
            set_skill_points,
            set_skill_specialization,
            set_skill_explanation,
            remove_skill,
            add_technique,
            set_technique_points,
            set_technique_explanation,
            remove_technique,
            list_equipment_library,
            add_equipment,
            set_equipment_quantity,
            set_equipment_carried,
            set_equipment_explanation,
            remove_equipment,
            list_weapon_library,
            add_weapon,
            remove_weapon,
            set_weapon_explanation,
            list_spell_library,
            add_spell,
            set_spell_points,
            set_spell_explanation,
            remove_spell,
            list_modifier_library,
            add_trait_modifier,
            remove_trait_modifier,
            set_modifier_explanation,
            plugin_status,
            reload_plugins,
            roll,
            roll_damage,
            undo,
            redo,
            set_current
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
