mod state;

use gurps_dice::{DamageRoll, RollResult};
pub use state::{AppState, SheetView};
use state::{
    EquipmentLibEntry, LibraryEntry, LocaleInfo, ModifierLibEntry, PluginStatus, SkillLibrary,
    SpellLibEntry,
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
    let before = st.character.clone();
    let label = label.into();
    let out = guard(|| {
        st.record(label, f)?;
        Ok(st.view())
    });
    if out.is_err() && st.character != before {
        // A panic mid-edit must not leave a half-applied character behind.
        st.character = before;
    }
    out
}

/// Turns a panic into an `Err` for the UI instead of killing the app.
fn guard<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|p| {
        let msg = p
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown panic".into());
        Err(format!("РІРЅСѓС‚СЂРµРЅРЅСЏСЏ РѕС€РёР±РєР°: {msg}"))
    })
}

/// Default folder for relative file names: Documents/GURPS Sheet.
fn default_save_dir(app: &tauri::App) -> Option<PathBuf> {
    use tauri::Manager;
    let base = app
        .path()
        .document_dir()
        .or_else(|_| app.path().home_dir())
        .ok()?;
    Some(base.join("GURPS Sheet"))
}

#[tauri::command]
fn new_character(state: S, name: Option<String>) -> SheetView {
    lock(&state).new_character(name)
}

#[tauri::command]
fn load(state: S, path: String) -> Result<SheetView, String> {
    let mut st = lock(&state);
    guard(|| st.load(&path))
}

#[tauri::command]
fn save(state: S, path: Option<String>) -> Result<String, String> {
    let mut st = lock(&state);
    guard(|| st.save(path))
}

#[tauri::command]
fn set_negative_rounding(state: S, mode: String) -> Result<SheetView, String> {
    edit(&state, "РїСЂР°РІРёР»Рѕ РѕРєСЂСѓРіР»РµРЅРёСЏ", |s| {
        s.set_negative_rounding(&mode)
    })
}

#[tauri::command]
fn set_attribute(state: S, key: String, level: i32) -> Result<SheetView, String> {
    edit(&state, format!("РёР·РјРµРЅРµРЅРёРµ {key}"), |s| {
        s.set_attribute(&key, level)
    })
}

#[tauri::command]
fn add_ledger_entry(state: S, entry: state::LedgerInput) -> Result<SheetView, String> {
    edit(&state, "Р·Р°РїРёСЃСЊ РІ Р¶СѓСЂРЅР°Р»Рµ РѕС‡РєРѕРІ", |s| {
        s.add_ledger_entry(entry)
    })
}

#[tauri::command]
fn update_ledger_entry(state: S, id: u32, entry: state::LedgerInput) -> Result<SheetView, String> {
    edit(
        &state,
        "РїСЂР°РІРєР° Р·Р°РїРёСЃРё Р¶СѓСЂРЅР°Р»Р° РѕС‡РєРѕРІ",
        |s| s.update_ledger_entry(id, entry),
    )
}

#[tauri::command]
fn remove_ledger_entry(state: S, id: u32) -> Result<SheetView, String> {
    edit(
        &state,
        "СѓРґР°Р»РµРЅРёРµ Р·Р°РїРёСЃРё Р¶СѓСЂРЅР°Р»Р° РѕС‡РєРѕРІ",
        |s| s.remove_ledger_entry(id),
    )
}

#[tauri::command]
fn set_disadvantage_limit(state: S, limit: Option<i32>) -> Result<SheetView, String> {
    edit(&state, "Р»РёРјРёС‚ РЅРµРґРѕСЃС‚Р°С‚РєРѕРІ", |s| {
        s.set_disadvantage_limit(limit)
    })
}

#[tauri::command]
fn save_custom(state: S, kind: String, def: serde_json::Value) -> Result<SheetView, String> {
    edit(&state, "СЃРІРѕС‘ РѕРїСЂРµРґРµР»РµРЅРёРµ", |s| {
        s.save_custom(&kind, def)
    })
}

#[tauri::command]
fn remove_custom(state: S, kind: String, id: String) -> Result<SheetView, String> {
    edit(
        &state,
        "СѓРґР°Р»РµРЅРёРµ СЃРІРѕРµРіРѕ РѕРїСЂРµРґРµР»РµРЅРёСЏ",
        |s| s.remove_custom(&kind, &id),
    )
}

#[tauri::command]
fn set_notes(state: S, text: String) -> Result<SheetView, String> {
    edit(&state, "Р·Р°РјРµС‚РєРё", |s| Ok(s.set_notes(text)))
}

#[tauri::command]
fn set_name(state: S, name: String) -> Result<SheetView, String> {
    edit(&state, "РїРµСЂРµРёРјРµРЅРѕРІР°РЅРёРµ", |s| {
        Ok(s.set_name(name))
    })
}

#[tauri::command]
fn add_trait(state: S, id: String, levels: Option<i32>) -> Result<SheetView, String> {
    edit(&state, "РґРѕР±Р°РІР»РµРЅРёРµ С‡РµСЂС‚С‹", |s| {
        s.add_trait(&id, levels)
    })
}

#[tauri::command]
fn set_trait_levels(state: S, index: usize, levels: i32) -> Result<SheetView, String> {
    edit(&state, "СѓСЂРѕРІРµРЅСЊ С‡РµСЂС‚С‹", |s| {
        s.set_trait_levels(index, levels)
    })
}

#[tauri::command]
fn remove_trait(state: S, index: usize) -> Result<SheetView, String> {
    edit(&state, "СѓРґР°Р»РµРЅРёРµ С‡РµСЂС‚С‹", |s| {
        s.remove_trait(index)
    })
}

#[tauri::command]
fn set_trait_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "РїРѕСЏСЃРЅРµРЅРёРµ С‡РµСЂС‚С‹", |s| {
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
    edit(&state, "РґРѕР±Р°РІР»РµРЅРёРµ РЅР°РІС‹РєР°", |s| {
        s.add_skill(&id, specialization, points)
    })
}

#[tauri::command]
fn set_skill_points(state: S, index: usize, points: i32) -> Result<SheetView, String> {
    edit(&state, "РѕС‡РєРё РЅР°РІС‹РєР°", |s| {
        s.set_skill_points(index, points)
    })
}

#[tauri::command]
fn set_skill_specialization(
    state: S,
    index: usize,
    specialization: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "СЃРїРµС†РёР°Р»РёР·Р°С†РёСЏ РЅР°РІС‹РєР°", |s| {
        s.set_skill_specialization(index, specialization)
    })
}

#[tauri::command]
fn set_skill_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "РїРѕСЏСЃРЅРµРЅРёРµ РЅР°РІС‹РєР°", |s| {
        s.set_skill_explanation(index, text)
    })
}

#[tauri::command]
fn remove_skill(state: S, index: usize) -> Result<SheetView, String> {
    edit(&state, "СѓРґР°Р»РµРЅРёРµ РЅР°РІС‹РєР°", |s| {
        s.remove_skill(index)
    })
}

#[tauri::command]
fn add_technique(state: S, id: String, points: Option<i32>) -> Result<SheetView, String> {
    edit(&state, "РґРѕР±Р°РІР»РµРЅРёРµ РїСЂРёС‘РјР°", |s| {
        s.add_technique(&id, points)
    })
}

#[tauri::command]
fn set_technique_points(state: S, index: usize, points: i32) -> Result<SheetView, String> {
    edit(&state, "РѕС‡РєРё РїСЂРёС‘РјР°", |s| {
        s.set_technique_points(index, points)
    })
}

#[tauri::command]
fn set_technique_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "РїРѕСЏСЃРЅРµРЅРёРµ РїСЂРёС‘РјР°", |s| {
        s.set_technique_explanation(index, text)
    })
}

#[tauri::command]
fn remove_technique(state: S, index: usize) -> Result<SheetView, String> {
    edit(&state, "СѓРґР°Р»РµРЅРёРµ РїСЂРёС‘РјР°", |s| {
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
    edit(&state, "РґРѕР±Р°РІР»РµРЅРёРµ СЃРЅР°СЂСЏР¶РµРЅРёСЏ", |s| {
        s.add_equipment(&id, quantity, parent)
    })
}

#[tauri::command]
fn set_equipment_quantity(state: S, path: Vec<usize>, quantity: u32) -> Result<SheetView, String> {
    edit(&state, "РєРѕР»РёС‡РµСЃС‚РІРѕ СЃРЅР°СЂСЏР¶РµРЅРёСЏ", |s| {
        s.set_equipment_quantity(&path, quantity)
    })
}

#[tauri::command]
fn set_equipment_carried(state: S, path: Vec<usize>, carried: bool) -> Result<SheetView, String> {
    edit(&state, "РїСЂРё СЃРµР±Рµ / РѕСЃС‚Р°РІР»РµРЅРѕ", |s| {
        s.set_equipment_carried(&path, carried)
    })
}

#[tauri::command]
fn set_equipment_explanation(
    state: S,
    path: Vec<usize>,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "РїРѕСЏСЃРЅРµРЅРёРµ СЃРЅР°СЂСЏР¶РµРЅРёСЏ", |s| {
        s.set_equipment_explanation(&path, text)
    })
}

#[tauri::command]
fn remove_equipment(state: S, path: Vec<usize>) -> Result<SheetView, String> {
    edit(&state, "СѓРґР°Р»РµРЅРёРµ СЃРЅР°СЂСЏР¶РµРЅРёСЏ", |s| {
        s.remove_equipment(&path)
    })
}

#[tauri::command]
fn list_spell_library(state: S) -> Vec<SpellLibEntry> {
    lock(&state).spell_library_view()
}

#[tauri::command]
fn add_spell(state: S, id: String, points: Option<i32>) -> Result<SheetView, String> {
    edit(&state, "РґРѕР±Р°РІР»РµРЅРёРµ Р·Р°РєР»РёРЅР°РЅРёСЏ", |s| {
        s.add_spell(&id, points)
    })
}

#[tauri::command]
fn set_spell_points(state: S, index: usize, points: i32) -> Result<SheetView, String> {
    edit(&state, "РѕС‡РєРё Р·Р°РєР»РёРЅР°РЅРёСЏ", |s| {
        s.set_spell_points(index, points)
    })
}

#[tauri::command]
fn set_spell_explanation(
    state: S,
    index: usize,
    text: Option<String>,
) -> Result<SheetView, String> {
    edit(&state, "РїРѕСЏСЃРЅРµРЅРёРµ Р·Р°РєР»РёРЅР°РЅРёСЏ", |s| {
        s.set_spell_explanation(index, text)
    })
}

#[tauri::command]
fn remove_spell(state: S, index: usize) -> Result<SheetView, String> {
    edit(&state, "СѓРґР°Р»РµРЅРёРµ Р·Р°РєР»РёРЅР°РЅРёСЏ", |s| {
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
        "РґРѕР±Р°РІР»РµРЅРёРµ РјРѕРґРёС„РёРєР°С‚РѕСЂР°",
        |s| s.add_trait_modifier(trait_index, &id),
    )
}

#[tauri::command]
fn remove_trait_modifier(state: S, trait_index: usize, index: usize) -> Result<SheetView, String> {
    edit(&state, "СѓРґР°Р»РµРЅРёРµ РјРѕРґРёС„РёРєР°С‚РѕСЂР°", |s| {
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
    edit(&state, "РїРѕСЏСЃРЅРµРЅРёРµ РјРѕРґРёС„РёРєР°С‚РѕСЂР°", |s| {
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
        ("HP", None) | ("FP", None) => format!("СЃР±СЂРѕСЃ {stat}"),
        _ => format!("С‚РµРєСѓС‰РёРµ {stat}"),
    };
    edit(&state, label, |s| s.set_current(&stat, value))
}

/// Catalog source: the shared SurrealDB server (same as gurps-mcp), read-only.
/// Connection from env `SURREALDB_HOST/NAMESPACE/NAME/USER/PASSWORD`.
fn library_source() -> gurps_library::LibrarySource {
    let src = gurps_library::LibrarySource::Db(gurps_library::DbConfig::from_env());
    eprintln!("gurps-sheet: catalog from {}", src.describe());
    src
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
        .manage(Mutex::new(AppState::with_source(Some(library_source()))))
        .setup(|app| {
            use tauri::Manager;
            let dir = default_save_dir(app);
            eprintln!("gurps-sheet: relative file names resolve to {dir:?}");
            app.state::<Mutex<AppState>>()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .save_dir = dir;
            Ok(())
        })
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
