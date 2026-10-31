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

fn entry(what: &str, id: &str, pack: &Option<String>) -> Result<(String, String), String> {
    let pack = pack
        .as_deref()
        .filter(|p| !p.trim().is_empty())
        .ok_or_else(|| format!("{what} {id}: не указан пак (source)"))?;
    Ok((pack.to_string(), sid(id)))
}

fn items(out: &mut Vec<Value>, list: &[Equipment], carried: bool) -> Result<(), String> {
    for e in list {
        let (pack, sid) = entry("equipment", &e.def.id, &e.source)?;
        let on = carried && e.carried;
        out.push(json!({ "pack": pack, "sid": sid, "qty": e.quantity.max(1), "equipped": on }));
        items(out, &e.contents, on)?;
    }
    Ok(())
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
    let mut traits = Vec::new();
    for t in &ch.traits {
        let (pack, sid) = entry("trait", &t.def.id, &t.source)?;
        let mut mods = Vec::new();
        for m in &t.modifiers {
            let (mp, ms) = entry("modifier", &m.def.id, &m.source)?;
            mods.push(json!({ "pack": mp, "sid": ms }));
        }
        let level = (t.def.cost_per_level != 0).then_some(t.levels);
        traits.push(json!({ "pack": pack, "sid": sid, "level": level, "modifiers": mods }));
    }
    let mut skills = Vec::new();
    for s in &ch.skills {
        let (pack, sid) = entry("skill", &s.def.id, &s.source)?;
        let spec = s
            .specialization
            .as_deref()
            .filter(|x| !x.trim().is_empty())
            .map(self::sid);
        skills.push(json!({ "pack": pack, "sid": sid, "points": s.points, "spec": spec }));
    }
    let mut techniques = Vec::new();
    for t in &ch.techniques {
        let (pack, sid) = entry("technique", &t.def.id, &t.source)?;
        techniques.push(json!({ "pack": pack, "sid": sid, "points": t.points }));
    }
    let mut spells = Vec::new();
    for s in &ch.spells {
        let (pack, sid) = entry("spell", &s.def.id, &s.source)?;
        spells.push(json!({ "pack": pack, "sid": sid, "points": s.points }));
    }
    let mut its = Vec::new();
    items(&mut its, &ch.equipment, true)?;
    // Виды point_ledger в схеме: award/spend/refund/adjust; старый лист без журнала — points_budget.
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
