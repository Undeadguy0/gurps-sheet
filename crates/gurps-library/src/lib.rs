//! Каталог редактора (библиотека трейтов, навыков, снаряжения, …).
//!
//! Источник — общий сервер SurrealDB (тот же, что у gurps-mcp), только чтение: каталог
//! правится сидами/MCP, лист пишет лишь персонажей. Подключение — env `SURREALDB_*`
//! ([`DbConfig::from_env`]). Ключ записи библиотеки — код пака БД (`basic`, `homebrew`).
//!
//! Записи — определения `gurps_core::defs` (проекция [`project`]); персонаж ссылается на них
//! по `(pack, sid)` или хранит своё определение встроенным.
use gurps_core::{
    AttackMode, Def, ItemDef, ModifierDef, SkillDef, SpellDef, Src, TechniqueDef, TraitDef,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use gurps_db::gurps_calc;
pub use gurps_db::DbConfig;

pub mod calc;
pub mod convert;
pub mod project;
pub mod sheet;

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
}

impl LibrarySource {
    /// Строка для UI («откуда данные»), без пароля.
    pub fn describe(&self) -> String {
        match self {
            LibrarySource::Db(c) => {
                format!("SurrealDB {} ({}/{})", c.host, c.namespace, c.database)
            }
        }
    }
}

/// Каталог в памяти. Списки: (код пака, определение) всех паков по возрастанию priority;
/// действующая запись по sid — из последнего пака (как `fn::effective` в БД).
#[derive(Debug, Clone, Default)]
pub struct Library {
    /// Загруженные паки по возрастанию приоритета (для UI).
    pub packs: Vec<String>,
    pub traits: Vec<(String, TraitDef)>,
    pub skills: Vec<(String, SkillDef)>,
    pub techniques: Vec<(String, TechniqueDef)>,
    pub items: Vec<(String, ItemDef)>,
    pub spells: Vec<(String, SpellDef)>,
    pub modifiers: Vec<(String, ModifierDef)>,
    /// Естественные атаки (`attack_mode.natural`).
    pub natural: Vec<(String, AttackMode)>,
    /// Код колледжа -> имя.
    pub colleges: BTreeMap<String, String>,
    pub translations: Translations,
    /// Записи, которые не удалось разобрать.
    pub warnings: Vec<String>,
}

/// Определение по ссылке: встроенное или `(pack, sid)` из списка.
pub fn resolve<'a, D: Def>(list: &'a [(String, D)], src: &'a Src<D>) -> Option<&'a D> {
    match src {
        Src::Inline { inline } => Some(inline),
        Src::Ref { pack, sid } => list
            .iter()
            .find(|(p, d)| p == pack && d.sid() == sid)
            .map(|(_, d)| d),
    }
}

/// Действующие записи (по sid побеждает поздний пак), по sid.
pub fn effective<D: Def + Clone>(list: &[(String, D)]) -> Vec<(String, D)> {
    let mut m: BTreeMap<String, (String, D)> = BTreeMap::new();
    for (p, d) in list {
        m.insert(d.sid().to_string(), (p.clone(), d.clone()));
    }
    m.into_values().collect()
}

/// Действующая запись по sid.
pub fn find<'a, D: Def>(list: &'a [(String, D)], sid: &str) -> Option<(&'a str, &'a D)> {
    list.iter()
        .rev()
        .find(|(_, d)| d.sid() == sid)
        .map(|(p, d)| (p.as_str(), d))
}

impl Library {
    pub fn load(src: &LibrarySource) -> Result<Self, String> {
        match src {
            LibrarySource::Db(cfg) => Self::load_db(cfg),
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
                let db = gurps_db::Db::connect(&cfg)
                    .await
                    .map_err(|e| e.to_string())?;
                Self::fetch(&db).await
            })
        })
        .join()
        .map_err(|_| "загрузка каталога: поток завершился паникой".to_string())?
    }

    /// Каталог БД -> библиотека (без подписей справочников).
    pub fn from_catalog(cat: &gurps_db::catalog::Catalog) -> Self {
        fn take<D: serde::de::DeserializeOwned>(
            cat: &gurps_db::catalog::Catalog,
            table: &str,
            kind: &str,
            lib: &mut Library,
        ) -> Vec<(String, D)> {
            let mut out = Vec::new();
            for e in cat.table(table) {
                match project::def::<D>(e) {
                    Ok(d) => out.push((e.id.pack.clone(), d)),
                    Err(err) => lib.warnings.push(err),
                }
                for (lang, f) in &e.tr {
                    if lang == "en" {
                        continue;
                    }
                    let le = LocEntry {
                        name: f.get("name").cloned(),
                        description: f.get("description").cloned(),
                    };
                    if le.name.is_some() || le.description.is_some() {
                        lib.translations
                            .langs
                            .entry(lang.clone())
                            .or_default()
                            .insert(format!("{kind}-{}", e.id.sid), le);
                    }
                }
            }
            // Порядок паков каталога (по возрастанию priority), внутри — по sid.
            let rank = |p: &str| cat.packs.iter().position(|x| x.code == p);
            out.sort_by_key(|(p, _)| rank(p));
            out
        }
        let mut lib = Library::default();
        lib.traits = take(cat, "trait", "trait", &mut lib);
        lib.skills = take(cat, "skill", "skill", &mut lib);
        lib.techniques = take(cat, "technique", "technique", &mut lib);
        lib.spells = take(cat, "spell", "spell", &mut lib);
        lib.items = take(cat, "item", "equipment", &mut lib);
        lib.modifiers = take(cat, "modifier", "modifier", &mut lib);
        let modes: Vec<(String, AttackMode)> = take(cat, "attack_mode", "weapon", &mut lib);
        lib.natural = modes.into_iter().filter(|(_, m)| m.natural).collect();
        lib.packs = cat
            .packs
            .iter()
            .map(|p| format!("{} (priority {})", p.code, p.priority))
            .collect();
        lib
    }

    /// Загрузка каталога через готовое подключение.
    pub async fn fetch(db: &gurps_db::Db) -> Result<Self, String> {
        let cat = gurps_db::catalog::Catalog::load(db)
            .await
            .map_err(|e| e.to_string())?;
        let mut lib = Self::from_catalog(&cat);
        lib.colleges = gurps_db::catalog::college_names(db)
            .await
            .map_err(|e| e.to_string())?;
        // Подписи справочников для листа: `attribute-ST`, `encumbrance-light` (переводы из БД).
        for (table, kind) in [
            ("ref_attribute", "attribute"),
            ("ref_encumbrance", "encumbrance"),
        ] {
            let rows = gurps_db::catalog::refs(db, table)
                .await
                .map_err(|e| e.to_string())?;
            for r in rows {
                for (lang, fields) in &r.tr {
                    if let Some(n) = fields.get("name") {
                        lib.translations
                            .langs
                            .entry(lang.clone())
                            .or_default()
                            .entry(format!("{kind}-{}", r.code))
                            .or_default()
                            .name = Some(n.clone());
                    }
                }
            }
        }
        Ok(lib)
    }
}
