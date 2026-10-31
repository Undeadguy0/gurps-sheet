//! Каталог редактора (библиотека трейтов, навыков, снаряжения, …).
//!
//! Источник — общий сервер SurrealDB (тот же, что у gurps-mcp), только чтение: каталог
//! правится сидами/MCP, лист пишет лишь персонажей. Подключение — env `SURREALDB_*`
//! ([`DbConfig::from_env`]). Ключ записи библиотеки — код пака БД (`basic`, `homebrew`).
//!
//! Записи пока отдаются в старых типах `gurps-core` через временный адаптер
//! ([`db_adapter`]) — расчёт переводится на `gurps-calc` на этапе 4.
use gurps_core::{
    EquipmentDef, ModifierDef, SkillDef, SpellDef, TechniqueDef, TraitDef, WeaponDef,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use gurps_db::gurps_calc;
pub use gurps_db::DbConfig;

pub mod calc;
pub mod db_adapter;
#[cfg(feature = "legacy-plugins")]
mod legacy;

/// Перевод записи: имя и описание.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocEntry {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// `lang -> "kind-id" -> entry` (переводы, кроме `en` — он в самих определениях).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Translations {
    pub langs: BTreeMap<String, BTreeMap<String, LocEntry>>,
}

impl Translations {
    /// Поля `other` перекрывают имеющиеся.
    pub fn merge(&mut self, other: &Translations) {
        for (lang, m) in &other.langs {
            let dst = self.langs.entry(lang.clone()).or_default();
            for (k, e) in m {
                let d = dst.entry(k.clone()).or_default();
                if e.name.is_some() {
                    d.name.clone_from(&e.name);
                }
                if e.description.is_some() {
                    d.description.clone_from(&e.description);
                }
            }
        }
    }

    pub fn languages(&self) -> Vec<String> {
        self.langs.keys().cloned().collect()
    }

    /// Запись `kind`/`id` для `locale` (точно, затем язык из `ru-RU` -> `ru`).
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

/// Откуда брать каталог.
#[derive(Debug, Clone)]
pub enum LibrarySource {
    /// Общий сервер SurrealDB (рабочий режим).
    Db(DbConfig),
    /// ВРЕМЕННО (этап 4 удалит): старые TOML-плагины, только для тестов.
    #[cfg(feature = "legacy-plugins")]
    Plugins(std::path::PathBuf),
}

impl LibrarySource {
    /// Строка для UI («откуда данные»), без пароля.
    pub fn describe(&self) -> String {
        match self {
            LibrarySource::Db(c) => format!("SurrealDB {} ({}/{})", c.host, c.namespace, c.database),
            #[cfg(feature = "legacy-plugins")]
            LibrarySource::Plugins(p) => p.display().to_string(),
        }
    }
}

/// Каталог в памяти. Списки: (код пака, определение); при одинаковом id побеждает пак
/// с большим priority (как `fn::effective` в БД).
#[derive(Debug, Clone, Default)]
pub struct Library {
    /// Загруженные паки по возрастанию приоритета (для UI).
    pub packs: Vec<String>,
    pub traits: Vec<(String, TraitDef)>,
    pub skills: Vec<(String, SkillDef)>,
    pub techniques: Vec<(String, TechniqueDef)>,
    pub equipment: Vec<(String, EquipmentDef)>,
    pub weapons: Vec<(String, WeaponDef)>,
    pub spells: Vec<(String, SpellDef)>,
    pub modifiers: Vec<(String, ModifierDef)>,
    pub translations: Translations,
    /// Что не переносится из БД в старые типы листа (отчёт адаптера).
    pub warnings: Vec<String>,
}

impl Library {
    pub fn load(src: &LibrarySource) -> Result<Self, String> {
        match src {
            LibrarySource::Db(cfg) => Self::load_db(cfg),
            #[cfg(feature = "legacy-plugins")]
            LibrarySource::Plugins(dir) => legacy::load(dir),
        }
    }

    /// Синхронная загрузка (свой поток и runtime — можно звать откуда угодно, в т.ч. из
    /// команд Tauri).
    pub fn load_db(cfg: &DbConfig) -> Result<Self, String> {
        let cfg = cfg.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("tokio: {e}"))?;
            rt.block_on(async {
                let db = gurps_db::Db::connect(&cfg).await.map_err(|e| e.to_string())?;
                Self::fetch(&db).await
            })
        })
        .join()
        .map_err(|_| "загрузка каталога: поток завершился паникой".to_string())?
    }

    /// Загрузка каталога через готовое подключение.
    pub async fn fetch(db: &gurps_db::Db) -> Result<Self, String> {
        let cat = gurps_db::catalog::Catalog::load(db)
            .await
            .map_err(|e| e.to_string())?;
        let colleges = gurps_db::catalog::college_names(db)
            .await
            .map_err(|e| e.to_string())?;
        db_adapter::from_catalog(&cat, &colleges)
    }
}

/// Сплющить по id: поздние (более приоритетные) перекрывают ранние.
pub(crate) fn flatten<T>(items: Vec<(String, T)>, id: impl Fn(&T) -> &str) -> Vec<(String, T)> {
    let mut m: BTreeMap<String, (String, T)> = BTreeMap::new();
    for (p, t) in items {
        m.insert(id(&t).to_string(), (p, t));
    }
    m.into_values().collect()
}