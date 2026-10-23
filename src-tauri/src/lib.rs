mod state;

pub use state::{AppState, SheetView};
use gurps_rules::dice::RollResult;
use state::LibraryEntry;
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
fn get_sheet(state: S) -> SheetView {
    lock(&state).view()
}

#[tauri::command]
fn list_library(state: S) -> Vec<LibraryEntry> {
    lock(&state).library.clone()
}

#[tauri::command]
fn roll(target: i32, modifier: Option<i32>) -> RollResult {
    AppState::roll(target, modifier.unwrap_or(0))
}

/// Plugin directory: `$GURPS_PLUGINS`, else `./plugins` or `../plugins` (dev).
fn plugin_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("GURPS_PLUGINS") {
        return Some(p.into());
    }
    ["plugins", "../plugins"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.is_dir())
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
            get_sheet,
            list_library,
            roll
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
