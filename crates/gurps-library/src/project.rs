//! Записи каталога БД -> определения листа (`gurps_core::defs`).
//!
//! Ссылки БД приводятся к строкам: запись каталога `skill:['basic', 'x']` -> sid `x`,
//! справочник `ref_attribute:DX` -> код `DX`. Имя — перевод `en`, описание — все языки.
use gurps_core::LocalizedText;
use gurps_db::catalog::{parse_catalog_record, Entry};
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// Ссылки БД -> sid/код (рекурсивно).
pub fn normalize(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(ref_text(s).unwrap_or_else(|| s.clone())),
        Value::Array(a) => Value::Array(a.iter().map(normalize).collect()),
        Value::Object(o) => Value::Object(
            o.iter()
                .filter(|(k, _)| !k.starts_with("_tr"))
                .map(|(k, x)| (k.clone(), normalize(x)))
                .collect(),
        ),
        x => x.clone(),
    }
}

fn ref_text(s: &str) -> Option<String> {
    if let Some((_, id)) = parse_catalog_record(s) {
        return Some(id.sid);
    }
    let (t, k) = s.split_once(':')?;
    (t.starts_with("ref_")
        && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !k.is_empty()
        && !k.contains(char::is_whitespace))
    .then(|| {
        k.trim_matches(|c| c == '\'' || c == '"' || c == '`' || c == '⟨' || c == '⟩')
            .to_string()
    })
}

fn page(n: &Value) -> Option<String> {
    let r = n.get("source_refs")?.as_array()?.first()?;
    let book = r.get("book").and_then(Value::as_str).unwrap_or("");
    Some(match r.get("page").and_then(Value::as_i64) {
        Some(p) => format!("{book}{p}"),
        None => book.to_string(),
    })
}

fn description(e: &Entry) -> Option<LocalizedText> {
    let m: BTreeMap<String, String> =
        e.tr.iter()
            .filter_map(|(l, f)| Some((l.clone(), f.get("description")?.clone())))
            .collect();
    (!m.is_empty()).then_some(LocalizedText::Localized(m))
}

fn get<'a>(n: &'a Value, k: &str) -> &'a Value {
    n.get(k).unwrap_or(&Value::Null)
}

/// Режим атаки БД -> [`AttackMode`] (`parry: {value}` -> число).
pub fn attack(m: &Value, sid: &str) -> Value {
    let d = get(m, "damage");
    json!({
        "sid": sid,
        "name": get(m, "name"),
        "natural": get(m, "natural").as_bool().unwrap_or(false),
        "skill": get(m, "skill"),
        "damage": { "base": get(d, "base"), "dice": get(d, "dice"), "mod": get(d, "mod").as_i64().unwrap_or(0), "type": get(d, "type") },
        "reach": m.get("reach").cloned().filter(|x| !x.is_null()).unwrap_or(json!([])),
        "parry": get(get(m, "parry"), "value"),
        "defaults": m.get("defaults").cloned().filter(|x| !x.is_null()).unwrap_or(json!([])),
        "skill_modifier": get(m, "skill_modifier"),
        "skill_damage_bonus": get(m, "skill_damage_bonus"),
        "technique": get(m, "technique"),
    })
}

fn strip_nulls(v: Value) -> Value {
    match v {
        Value::Object(o) => Value::Object(
            o.into_iter()
                .filter(|(_, x)| !x.is_null())
                .map(|(k, x)| (k, strip_nulls(x)))
                .collect::<Map<_, _>>(),
        ),
        Value::Array(a) => Value::Array(a.into_iter().map(strip_nulls).collect()),
        x => x,
    }
}

/// Тело определения (без имени/страницы/описания) по таблице.
fn body(table: &str, sid: &str, n: &Value) -> Value {
    match table {
        "trait" => json!({ "kind": get(n, "kind"), "cost": get(n, "cost"),
            "features": get(n, "features"), "prereqs": get(n, "prereqs") }),
        "modifier" => json!({ "type": get(n, "type"), "value_kind": get(get(n, "value"), "kind"),
            "amount": get(get(n, "value"), "amount") }),
        "skill" => {
            let mut defaults: Vec<Value> = get(n, "_attr_defaults")
                .as_array()
                .into_iter()
                .flatten()
                .map(|d| json!({ "attribute": get(d, "attribute"), "mod": get(d, "mod") }))
                .collect();
            defaults.extend(
                get(n, "_defaults_to")
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|d| json!({ "skill": get(d, "out"), "mod": get(d, "mod") })),
            );
            json!({ "attribute": get(n, "attribute"), "difficulty": get(n, "difficulty"),
                "defaults": defaults, "spec_required": get(n, "spec_mode").as_str() == Some("required"),
                "specializations": get(n, "specializations"), "features": get(n, "features"),
                "prereqs": get(n, "prereqs") })
        }
        "technique" => {
            json!({ "parent_skill": get(n, "parent_skill"), "difficulty": get(n, "difficulty"),
            "default_mod": get(n, "default_mod"), "max_mod": get(n, "max_mod"), "prereqs": get(n, "prereqs") })
        }
        "spell" => json!({ "colleges": get(n, "colleges"), "difficulty": get(n, "difficulty"),
            "prereqs": get(n, "prereqs") }),
        "item" => {
            let sh = get(n, "shield");
            let shield = (!sh.is_null()).then(|| {
                json!({ "db": get(sh, "db").as_i64().unwrap_or(0),
                    "block_skill": get(get(sh, "block_skill"), "skill"),
                    "block_spec": get(get(sh, "block_skill"), "spec") })
            });
            let modes: Vec<Value> = get(n, "attack_modes")
                .as_array()
                .into_iter()
                .flatten()
                .map(|m| {
                    let name = get(m, "name").as_str().unwrap_or("");
                    attack(m, &format!("item/{name}"))
                })
                .collect();
            json!({ "weight": get(n, "weight"), "cost": get(n, "cost"),
                "container": get(n, "container").is_object(),
                "armor": get(get(n, "armor"), "locations"), "shield": shield,
                "attack_modes": modes, "features": get(n, "features") })
        }
        "attack_mode" => attack(n, sid),
        _ => json!({}),
    }
}

/// Запись каталога -> определение `D`.
pub fn def<D: DeserializeOwned>(e: &Entry) -> Result<D, String> {
    let n = normalize(&e.data);
    let mut v = body(e.table, &e.id.sid, &n);
    let o = v.as_object_mut().ok_or("body")?;
    o.insert("sid".into(), e.id.sid.clone().into());
    let name = e
        .text("en", "name")
        .map_or_else(|| e.id.sid.clone(), str::to_string);
    if e.table == "attack_mode" {
        o.entry("name").or_insert(Value::Null);
        if o["name"].is_null() {
            o.insert("name".into(), name.into());
        }
    } else {
        o.insert("name".into(), name.into());
        o.insert("page".into(), page(&n).into());
        o.insert(
            "description".into(),
            serde_json::to_value(description(e)).unwrap_or_default(),
        );
    }
    serde_json::from_value(strip_nulls(v)).map_err(|err| format!("{}:{}: {err}", e.table, e.id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refs_become_codes() {
        let v = normalize(&json!({
            "attribute": "ref_attribute:DX",
            "skill": "skill:['basic', 'sword_a']",
            "note": "Note: text",
            "_tr": {"en": {}}
        }));
        assert_eq!(
            v,
            json!({"attribute": "DX", "skill": "sword_a", "note": "Note: text"})
        );
    }
}
