//! Расчёт листа через `gurps-calc`: персонаж -> документ `(pack, sid)` -> `gurps_db::calc_input::load_doc`
//! (каталог и справочники правил — из БД) -> [`gurps_calc::compute`].
//! Опции правил персонажа передаются как `rule_overrides` (перекрывают `ref_rule`).

use gurps_core::{Character, Equipment, LedgerKind};
use gurps_db::gurps_calc::{self, model::Input, Computed};
use serde_json::{json, Value};

/// `ref_rule`, перекрываемые персонажем.
pub const RULE_NEGATIVE_ROUNDING: &str = "trait_cost_negative_rounding";
pub const RULE_DISADVANTAGE_LIMIT_PERCENT: &str = "disadvantage_limit_percent";
pub const RULE_QUIRK_LIMIT: &str = "quirk_limit";

/// sid каталога из id листа (`combat-reflexes` -> `combat_reflexes`).
pub fn sid(id: &str) -> String {
    id.trim().to_lowercase().replace(['-', ' '], "_")
}

/// Пак своего содержимого листа (записи без пака каталога): такие записи идут в документ
/// встроенными определениями (`inline`), а не ссылками `(pack, sid)`.
pub const LOCAL_SOURCE: &str = "custom";

/// Ссылка записи листа: каталог БД или встроенное определение.
fn catalog_pack(source: &Option<String>) -> Option<&str> {
    source
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty() && *p != LOCAL_SOURCE)
}

/// Запись документа: `{pack, sid}` или `{inline: def}` плюс поля экземпляра.
fn entry_of(
    source: &Option<String>,
    id: &str,
    inline: impl FnOnce() -> Value,
    extra: Value,
) -> Value {
    let mut o = match catalog_pack(source) {
        Some(pack) => json!({ "pack": pack, "sid": sid(id) }),
        None => json!({ "inline": inline() }),
    };
    if let (Some(o), Value::Object(x)) = (o.as_object_mut(), extra) {
        o.extend(x);
    }
    o
}

fn code<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|x| x.as_str().map(String::from))
        .unwrap_or_default()
}

/// Код атрибута БД (`ref_attribute`).
fn attr(s: gurps_core::Stat) -> String {
    use gurps_core::Stat::*;
    match s {
        BasicSpeed => "Speed".into(),
        BasicMove => "Move".into(),
        other => code(&other),
    }
}

/// Эффекты своего содержимого -> форма `features` расчёта (ссылки — ключи).
fn features(fs: &[gurps_core::Feature]) -> Vec<Value> {
    use gurps_core::Feature as F;
    let base = |kind: &str, amount: f64, per_level: bool| json!({ "kind": kind, "amount": amount, "per_level": per_level });
    let mut out = Vec::new();
    for f in fs {
        match f {
            F::AttributeBonus {
                target,
                amount,
                per_level,
            } => {
                let mut o = base("attribute_bonus", *amount, *per_level);
                o["attribute"] = attr(*target).into();
                out.push(o);
            }
            F::LiftingStBonus { amount, per_level } => {
                out.push(base("lifting_st_bonus", *amount, *per_level))
            }
            F::StrikingStBonus { amount, per_level } => {
                out.push(base("striking_st_bonus", *amount, *per_level))
            }
            F::DefenseBonus {
                amount,
                defenses,
                per_level,
            } => {
                let mut o = base("defense_bonus", *amount, *per_level);
                o["defenses"] = json!(defenses
                    .iter()
                    .map(|d| d.to_lowercase())
                    .collect::<Vec<_>>());
                out.push(o);
            }
            F::SkillBonus {
                skills,
                amount,
                per_level,
            } => {
                for s in skills {
                    let mut o = base("skill_bonus", *amount, *per_level);
                    o["skill"] = sid(s).into();
                    out.push(o);
                }
            }
            F::SpellBonus {
                amount,
                colleges,
                per_level,
            } => {
                if colleges.is_empty() {
                    out.push(base("spell_bonus", *amount, *per_level));
                }
                for c in colleges {
                    let mut o = base("spell_bonus", *amount, *per_level);
                    o["college"] = sid(c).into();
                    out.push(o);
                }
            }
        }
    }
    out
}

/// Встроенные определения (форма входа `gurps-calc`, см. `gurps_db::calc_input::load_doc`).
pub mod inline {
    use super::*;
    use gurps_core::{EquipmentDef, ModifierDef, SkillDef, SpellDef, TechniqueDef, TraitDef};

    pub fn trait_def(d: &TraitDef) -> Value {
        let ty = match (d.cost, d.cost_per_level) {
            (_, 0) => "fixed",
            (0, _) => "per_level",
            _ => "base_plus_level",
        };
        json!({ "sid": sid(&d.id), "cost": { "type": ty, "base": d.cost,
            "per_level": (d.cost_per_level != 0).then_some(d.cost_per_level) },
            "features": features(&d.features) })
    }
    pub fn modifier(d: &ModifierDef) -> Value {
        let lim = d.cost_percent < 0;
        json!({ "type": if lim { "lim" } else { "enh" }, "value_kind": "%", "amount": d.cost_percent.abs() })
    }
    pub fn skill(d: &SkillDef) -> Value {
        json!({ "sid": sid(&d.id), "attribute": attr(d.attribute), "difficulty": code(&d.difficulty) })
    }
    pub fn technique(d: &TechniqueDef) -> Value {
        json!({ "sid": sid(&d.id), "parent_skill": sid(&d.skill), "difficulty": code(&d.difficulty),
            "default_mod": d.modifier, "max_mod": d.max })
    }
    pub fn spell(d: &SpellDef) -> Value {
        json!({ "sid": sid(&d.id), "difficulty": code(&d.difficulty),
            "colleges": d.college.iter().map(|c| sid(c)).collect::<Vec<_>>() })
    }
    pub fn item(d: &EquipmentDef) -> Value {
        json!({ "sid": sid(&d.id), "weight": d.weight, "cost": d.cost })
    }
}

fn items(out: &mut Vec<Value>, list: &[Equipment], carried: bool) {
    for e in list {
        let on = carried && e.carried;
        out.push(entry_of(
            &e.source,
            &e.def.id,
            || inline::item(&e.def),
            json!({ "qty": e.quantity.max(1), "equipped": on }),
        ));
        items(out, &e.contents, on);
    }
}
/// Опции правил персонажа -> перекрытия `ref_rule`.
pub fn rule_overrides(ch: &Character) -> Vec<Value> {
    let mut v = Vec::new();
    if let Some(t) = serde_json::to_value(ch.rules.negative_rounding)
        .ok()
        .and_then(|x| x.as_str().map(String::from))
    {
        v.push(json!({ "code": RULE_NEGATIVE_ROUNDING, "text": t }));
    }
    v
}

/// Документ персонажа для [`gurps_db::calc_input::load_doc`].
pub fn document(ch: &Character) -> Result<Value, String> {
    let a = &ch.attributes;
    let attributes: Vec<Value> = [
        ("ST", a.st),
        ("DX", a.dx),
        ("IQ", a.iq),
        ("HT", a.ht),
        ("HP", a.hp),
        ("Will", a.will),
        ("Per", a.per),
        ("FP", a.fp),
        ("Speed", a.basic_speed_quarters),
        ("Move", a.basic_move),
    ]
    .iter()
    .map(|(c, l)| json!({ "code": c, "levels": l }))
    .collect();
    let traits: Vec<Value> = ch
        .traits
        .iter()
        .map(|t| {
            let mods: Vec<Value> = t
                .modifiers
                .iter()
                .map(|m| entry_of(&m.source, &m.def.id, || inline::modifier(&m.def), json!({})))
                .collect();
            let level = (t.def.cost_per_level != 0).then_some(t.levels);
            entry_of(
                &t.source,
                &t.def.id,
                || inline::trait_def(&t.def),
                json!({ "level": level, "modifiers": mods }),
            )
        })
        .collect();
    let skills: Vec<Value> = ch
        .skills
        .iter()
        .map(|s| {
            let spec = s
                .specialization
                .as_deref()
                .filter(|x| !x.trim().is_empty())
                .map(self::sid);
            entry_of(
                &s.source,
                &s.def.id,
                || inline::skill(&s.def),
                json!({ "points": s.points, "spec": spec }),
            )
        })
        .collect();
    let techniques: Vec<Value> = ch
        .techniques
        .iter()
        .map(|t| {
            entry_of(
                &t.source,
                &t.def.id,
                || inline::technique(&t.def),
                json!({ "points": t.points }),
            )
        })
        .collect();
    let spells: Vec<Value> = ch
        .spells
        .iter()
        .map(|s| {
            entry_of(
                &s.source,
                &s.def.id,
                || inline::spell(&s.def),
                json!({ "points": s.points }),
            )
        })
        .collect();
    let mut its = Vec::new();
    items(&mut its, &ch.equipment, true); // Виды point_ledger в схеме: award/spend/refund/adjust; старый лист без журнала — points_budget.
    let mut ledger: Vec<Value> = ch
        .ledger
        .iter()
        .map(|l| {
            let kind = match l.kind {
                LedgerKind::Correction => "adjust",
                _ if l.amount < 0 => "spend",
                _ => "award",
            };
            json!({ "kind": kind, "delta": l.amount })
        })
        .collect();
    if ledger.is_empty() && ch.points_budget != 0 {
        ledger.push(json!({ "kind": "award", "delta": ch.points_budget }));
    }
    Ok(json!({
        "attributes": attributes,
        "current_hp": ch.session.hp,
        "current_fp": ch.session.fp,
        "traits": traits,
        "skills": skills,
        "techniques": techniques,
        "spells": spells,
        "items": its,
        "ledger": ledger,
        "rule_overrides": rule_overrides(ch),
    }))
}

/// Вход расчёта и результат.
#[derive(Debug)]
pub struct Calc {
    pub input: Input,
    pub computed: Computed,
}

/// Расчёт через готовое подключение.
pub async fn compute(db: &gurps_db::Db, ch: &Character) -> Result<Calc, String> {
    let doc = document(ch)?;
    let input = gurps_db::calc_input::load_doc(db, &doc)
        .await
        .map_err(|e| e.to_string())?;
    let computed = gurps_calc::compute(&input);
    Ok(Calc { input, computed })
}

/// Замечание проверки листа (лимиты из `ref_rule`; кости и проверки — на стороне приложения).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Issue {
    pub severity: &'static str,
    pub message: String,
}

/// Лимиты недостатков и причуд: проценты/число — из `ref_rule` (с перекрытиями персонажа),
/// ручной лимит листа (`rules.disadvantage_limit`) важнее процента.
pub fn validate(ch: &Character, c: &Calc) -> Vec<Issue> {
    let refs = c.input.refs.with_overrides(&c.input.rule_overrides);
    let mut out = Vec::new();
    let quirks = ch
        .traits
        .iter()
        .filter(|t| t.def.category == gurps_core::TraitCategory::Quirk)
        .count();
    let quirk_pts: i64 = ch
        .traits
        .iter()
        .filter(|t| t.def.category == gurps_core::TraitCategory::Quirk)
        .map(|t| t.cost() as i64)
        .sum();
    let disadv = -(c.computed.points.disadvantages - quirk_pts);
    let starting: i64 = ch
        .ledger
        .iter()
        .filter(|l| l.kind == LedgerKind::Starting)
        .map(|l| l.amount as i64)
        .sum();
    let limit = match ch.rules.disadvantage_limit {
        Some(l) => Some(l as i64),
        None => refs
            .value(RULE_DISADVANTAGE_LIMIT_PERCENT)
            .map(|p| (starting as f64 * p / 100.0).floor() as i64),
    };
    if let Some(l) = limit {
        if disadv > l {
            out.push(Issue {
                severity: "warning",
                message: format!("Недостатки {disadv} очк. больше лимита {l}"),
            });
        }
    }
    if let Some(q) = refs.value(RULE_QUIRK_LIMIT) {
        if quirks as f64 > q {
            out.push(Issue {
                severity: "warning",
                message: format!("Причуд {quirks}, лимит {q}"),
            });
        }
    }
    for v in &c.computed.prereq_violations {
        out.push(Issue {
            severity: "warning",
            message: serde_json::to_string(v).unwrap_or_default(),
        });
    }
    out
}

/// Синхронный расчётчик для приложения: свой runtime и одно подключение к БД.
pub struct Engine {
    rt: tokio::runtime::Runtime,
    db: gurps_db::Db,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Engine")
    }
}

impl Engine {
    pub fn connect(cfg: &gurps_db::DbConfig) -> Result<Self, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("tokio: {e}"))?;
        let db = rt
            .block_on(gurps_db::Db::connect(cfg))
            .map_err(|e| e.to_string())?;
        Ok(Self { rt, db })
    }

    /// Расчёт листа (нельзя звать изнутри другого tokio runtime).
    pub fn compute(&self, ch: &Character) -> Result<Calc, String> {
        self.rt.block_on(compute(&self.db, ch))
    }
}
