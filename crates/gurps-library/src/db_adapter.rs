//! ВРЕМЕННЫЙ АДАПТЕР (снимается на этапе 4): записи каталога БД -> старые типы `gurps-core`.
//!
//! Проекция «как лист видит запись БД» уже реализована в `sheet-bridge`
//! (`catalog::export::to_sheet`, её же проверяют round-trip-тесты моста); её зеркальные типы
//! повторяют serde-форму `gurps-core`, поэтому перекладка — через `serde_json::Value`.
//! Ref-коды (`ref_attribute:DX`, `ref_difficulty:A`, `ref_trait_category:…`) превращаются
//! в enum'ы `Stat`/`Difficulty`/`TraitCategory`, sid — в kebab-id. На этапе 4 расчёт и UI
//! перейдут на `gurps-calc` и ключи `(pack, sid)`, и этот модуль уйдёт.
use crate::{flatten, Library, LocEntry};
use gurps_core::{
    EquipmentDef, ModifierDef, SkillDef, SpellDef, TechniqueDef, TraitDef, WeaponDef,
};
use gurps_db::catalog::Catalog;
use serde::Deserialize;
use sheet_bridge::catalog::export::{to_sheet, PackRows};
use sheet_bridge::report::{Dir, Report};
use std::collections::BTreeMap;

#[derive(Default, Deserialize)]
struct Data {
    #[serde(default)]
    traits: Vec<TraitDef>,
    #[serde(default)]
    skills: Vec<SkillDef>,
    #[serde(default)]
    techniques: Vec<TechniqueDef>,
    #[serde(default)]
    spells: Vec<SpellDef>,
    #[serde(default)]
    equipment: Vec<EquipmentDef>,
    #[serde(default)]
    weapons: Vec<WeaponDef>,
    #[serde(default)]
    modifiers: Vec<ModifierDef>,
}

fn tag<T>(pack: &str, v: Vec<T>) -> Vec<(String, T)> {
    v.into_iter().map(|x| (pack.to_string(), x)).collect()
}

/// Каталог БД (паки по возрастанию priority) -> библиотека листа.
pub fn from_catalog(cat: &Catalog, colleges: &BTreeMap<String, String>) -> Result<Library, String> {
    let mut lib = Library::default();
    let (mut tr, mut sk, mut te, mut sp, mut eq, mut we, mut mo) =
        (vec![], vec![], vec![], vec![], vec![], vec![], vec![]);
    for p in &cat.packs {
        let Some(rows) = cat.rows.get(&p.code) else {
            continue;
        };
        let mut rep = Report::default();
        let (data, locales) = to_sheet(&PackRows { rows: rows.clone() }, &mut rep, colleges);
        let v = serde_json::to_value(&data).map_err(|e| format!("pack {}: {e}", p.code))?;
        let d: Data =
            serde_json::from_value(v).map_err(|e| format!("pack {}: разбор каталога: {e}", p.code))?;
        tr.extend(tag(&p.code, d.traits));
        sk.extend(tag(&p.code, d.skills));
        te.extend(tag(&p.code, d.techniques));
        sp.extend(tag(&p.code, d.spells));
        eq.extend(tag(&p.code, d.equipment));
        we.extend(tag(&p.code, d.weapons));
        mo.extend(tag(&p.code, d.modifiers));
        for (lang, m) in locales {
            let dst = lib.translations.langs.entry(lang).or_default();
            for (k, e) in m {
                dst.insert(
                    k,
                    LocEntry {
                        name: e.name,
                        description: e.description,
                    },
                );
            }
        }
        for ((dir, topic), what) in rep.gaps {
            if dir == Dir::Export {
                lib.warnings
                    .push(format!("{}: {topic}: {}", p.code, what.join(", ")));
            }
        }
        lib.packs.push(format!("{} (priority {})", p.code, p.priority));
    }
    lib.traits = flatten(tr, |d| &d.id);
    lib.skills = flatten(sk, |d| &d.id);
    lib.techniques = flatten(te, |d| &d.id);
    lib.spells = flatten(sp, |d| &d.id);
    lib.equipment = flatten(eq, |d| &d.id);
    lib.weapons = flatten(we, |d| &d.id);
    lib.modifiers = flatten(mo, |d| &d.id);
    Ok(lib)
}