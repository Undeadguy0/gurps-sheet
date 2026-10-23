mod state;

use gurps_rules::dice::RollResult;
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
fn set_attribute(state: S, key: String, level: i32) -> Result<SheetView, String> {
    lock(&state).set_attribute(&key, level)
}

#[tauri::command]
fn set_name(state: S, name: String) -> SheetView {
    lock(&state).set_name(name)
}

#[tauri::command]
fn add_trait(state: S, id: String, levels: Option<u32>) -> Result<SheetView, String> {
    lock(&state).add_trait(&id, levels)
}

#[tauri::command]
fn set_trait_levels(state: S, index: usize, levels: u32) -> Result<SheetView, String> {
    lock(&state).set_trait_levels(index, levels)
}

#[tauri::command]
fn remove_trait(state: S, index: usize) -> Result<SheetView, String> {
    lock(&state).remove_trait(index)
}

#[tauri::command]
fn set_trait_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    lock(&state).set_trait_explanation(index, text)
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
    lock(&state).add_skill(&id, specialization, points)
}

#[tauri::command]
fn set_skill_points(state: S, index: usize, points: i32) -> Result<SheetView, String> {
    lock(&state).set_skill_points(index, points)
}

#[tauri::command]
fn set_skill_specialization(
    state: S,
    index: usize,
    specialization: Option<String>,
) -> Result<SheetView, String> {
    lock(&state).set_skill_specialization(index, specialization)
}

#[tauri::command]
fn set_skill_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    lock(&state).set_skill_explanation(index, text)
}

#[tauri::command]
fn remove_skill(state: S, index: usize) -> Result<SheetView, String> {
    lock(&state).remove_skill(index)
}

#[tauri::command]
fn add_technique(state: S, id: String, points: Option<i32>) -> Result<SheetView, String> {
    lock(&state).add_technique(&id, points)
}

#[tauri::command]
fn set_technique_points(state: S, index: usize, points: i32) -> Result<SheetView, String> {
    lock(&state).set_technique_points(index, points)
}

#[tauri::command]
fn set_technique_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    lock(&state).set_technique_explanation(index, text)
}

#[tauri::command]
fn remove_technique(state: S, index: usize) -> Result<SheetView, String> {
    lock(&state).remove_technique(index)
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
    lock(&state).add_equipment(&id, quantity, parent)
}

#[tauri::command]
fn set_equipment_quantity(state: S, path: Vec<usize>, quantity: u32) -> Result<SheetView, String> {
    lock(&state).set_equipment_quantity(&path, quantity)
}

#[tauri::command]
fn set_equipment_carried(state: S, path: Vec<usize>, carried: bool) -> Result<SheetView, String> {
    lock(&state).set_equipment_carried(&path, carried)
}

#[tauri::command]
fn set_equipment_explanation(
    state: S,
    path: Vec<usize>,
    text: Option<String>,
) -> Result<SheetView, String> {
    lock(&state).set_equipment_explanation(&path, text)
}

#[tauri::command]
fn remove_equipment(state: S, path: Vec<usize>) -> Result<SheetView, String> {
    lock(&state).remove_equipment(&path)
}

#[tauri::command]
fn list_weapon_library(state: S) -> Vec<WeaponLibEntry> {
    lock(&state).weapon_library_view()
}

#[tauri::command]
fn add_weapon(state: S, id: String) -> Result<SheetView, String> {
    lock(&state).add_weapon(&id)
}

#[tauri::command]
fn remove_weapon(state: S, index: usize) -> Result<SheetView, String> {
    lock(&state).remove_weapon(index)
}

#[tauri::command]
fn set_weapon_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    lock(&state).set_weapon_explanation(index, text)
}

#[tauri::command]
fn list_spell_library(state: S) -> Vec<SpellLibEntry> {
    lock(&state).spell_library_view()
}

#[tauri::command]
fn add_spell(state: S, id: String, points: Option<i32>) -> Result<SheetView, String> {
    lock(&state).add_spell(&id, points)
}

#[tauri::command]
fn set_spell_points(state: S, index: usize, points: i32) -> Result<SheetView, String> {
    lock(&state).set_spell_points(index, points)
}

#[tauri::command]
fn set_spell_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    lock(&state).set_spell_explanation(index, text)
}

#[tauri::command]
fn remove_spell(state: S, index: usize) -> Result<SheetView, String> {
    lock(&state).remove_spell(index)
}

#[tauri::command]
fn list_modifier_library(state: S) -> Vec<ModifierLibEntry> {
    lock(&state).modifier_library_view()
}

#[tauri::command]
fn add_trait_modifier(state: S, trait_index: usize, id: String) -> Result<SheetView, String> {
    lock(&state).add_trait_modifier(trait_index, &id)
}

#[tauri::command]
fn remove_trait_modifier(state: S, trait_index: usize, index: usize) -> Result<SheetView, String> {
    lock(&state).remove_trait_modifier(trait_index, index)
}

#[tauri::command]
fn set_modifier_explanation(
    state: S,
    trait_index: usize,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    lock(&state).set_modifier_explanation(trait_index, index, text)
}

#[tauri::command]
fn roll(target: i32, modifier: Option<i32>) -> RollResult {
    AppState::roll(target, modifier.unwrap_or(0))
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
            set_name,
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
            roll
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
