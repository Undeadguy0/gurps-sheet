//! Расчёт листа через `gurps-calc`: персонаж -> документ `(pack, sid)` / `inline` ->
//! `gurps_db::calc_input::load_doc` (каталог и справочники правил — из БД) -> [`gurps_calc::compute`].
//! Опции правил персонажа передаются как `rule_overrides` (перекрывают `ref_rule`).

use gurps_core::{Character, Item, LedgerKind, ATTRIBUTES};
use gurps_db::gurps_calc::{self, model::Input, Computed};
use serde_json::{json, Value};

/// `ref_rule`, перекрываемые персонажем.
pub const RULE_NEGATIVE_ROUNDING: &str = "trait_cost_negative_rounding";
pub const RULE_DISADVANTAGE_LIMIT_PERCENT: &str = "disadvantage_limit_percent";
pub const RULE_QUIRK_LIMIT: &str = "quirk_limit";

/// Запись документа: ссылка (`{pack, sid}` или `{inline}`) плюс поля экземпляра.
fn entry<T: serde::Serialize>(src: &T, extra: Value) -> Value {
    let mut o = serde_json::to_value(src).unwrap_or_default();
    if let (Some(o), Value::Object(x)) = (o.as_object_mut(), extra) {
        o.extend(x.into_iter().filter(|(_, v)| !v.is_null()));
    }
    o
}

fn items(out: &mut Vec<Value>, list: &[Item], carried: bool) {
    for e in list {
        let on = carried && e.equipped;
        out.push(entry(
            &e.src,
            json!({ "qty": e.qty.max(1), "equipped": on }),
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
pub fn document(ch: &Character) -> Value {
    let attributes: Vec<Value> = ATTRIBUTES
        .iter()
        .map(|c| json!({ "code": c, "levels": ch.attribute(c) }))
        .collect();
    let traits: Vec<Value> = ch
        .traits
        .iter()
        .map(|t| {
            let mods: Vec<Value> = t
                .modifiers
                .iter()
                .map(|m| entry(&m.src, json!({})))
                .collect();
            entry(&t.src, json!({ "level": t.level, "modifiers": mods }))
        })
        .collect();
    let skills: Vec<Value> = ch
        .skills
        .iter()
        .map(|s| {
            let spec = s.spec.as_deref().map(str::trim).filter(|x| !x.is_empty());
            entry(&s.src, json!({ "points": s.points, "spec": spec }))
        })
        .collect();
    let techniques: Vec<Value> = ch
        .techniques
        .iter()
        .map(|t| entry(&t.src, json!({ "points": t.points })))
        .collect();
    let spells: Vec<Value> = ch
        .spells
        .iter()
        .map(|s| entry(&s.src, json!({ "points": s.points })))
        .collect();
    let mut its = Vec::new();
    items(&mut its, &ch.items, true);
    // Виды point_ledger в схеме: award/spend/refund/adjust.
    let ledger: Vec<Value> = ch
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
    json!({
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
    })
}

/// Вход расчёта и результат.
#[derive(Debug)]
pub struct Calc {
    pub input: Input,
    pub computed: Computed,
}

/// Расчёт через готовое подключение.
pub async fn compute(db: &gurps_db::Db, ch: &Character) -> Result<Calc, String> {
    let doc = document(ch);
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
    let quirks = c
        .input
        .traits
        .iter()
        .filter(|t| t.kind.as_deref() == Some("quirk"))
        .count();
    let p = &c.computed.points;
    let disadv = -(p.disadvantages - p.quirks);
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
