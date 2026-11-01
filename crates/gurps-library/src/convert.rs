//! Документ персонажа БД (`gurps_db::calc_input::doc_of`) -> лист v2.
//!
//! Разовый перенос листов формата v1: `sheet-bridge character import` кладёт старый лист в БД,
//! затем [`from_doc`] даёт персонажа на ссылках `(pack, sid)`. Контейнеры в документе плоские
//! (вложенность не переносится), имя и заметки не входят в документ.
use gurps_core::{
    Character, Item, LedgerEntry, LedgerKind, Modifier, Skill, Spell, Src, Technique, Trait,
};
use serde_json::Value;

fn s(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}
fn i(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(Value::as_i64)
}
fn list<'a>(doc: &'a Value, k: &str) -> impl Iterator<Item = &'a Value> {
    doc.get(k).and_then(Value::as_array).into_iter().flatten()
}
fn src<D>(e: &Value) -> Src<D> {
    Src::Ref {
        pack: s(e, "pack"),
        sid: s(e, "sid"),
    }
}

pub fn from_doc(doc: &Value, name: &str) -> Character {
    let mut ch = Character::new(name);
    for a in list(doc, "attributes") {
        ch.set_attribute(&s(a, "code"), i(a, "levels").unwrap_or(0) as i32);
    }
    ch.session.hp = i(doc, "current_hp").map(|v| v as i32);
    ch.session.fp = i(doc, "current_fp").map(|v| v as i32);
    ch.traits = list(doc, "traits")
        .map(|t| Trait {
            src: src(t),
            level: i(t, "level").map(|v| v as i32),
            modifiers: list(t, "modifiers")
                .map(|m| Modifier {
                    src: src(m),
                    explanation: None,
                })
                .collect(),
            explanation: None,
        })
        .collect();
    let pts = |e: &Value| i(e, "points").unwrap_or(0) as i32;
    ch.skills = list(doc, "skills")
        .map(|e| Skill {
            src: src(e),
            points: pts(e),
            spec: e.get("spec").and_then(Value::as_str).map(String::from),
            explanation: None,
        })
        .collect();
    ch.techniques = list(doc, "techniques")
        .map(|e| Technique {
            src: src(e),
            points: pts(e),
            explanation: None,
        })
        .collect();
    ch.spells = list(doc, "spells")
        .map(|e| Spell {
            src: src(e),
            points: pts(e),
            explanation: None,
        })
        .collect();
    ch.items = list(doc, "items")
        .map(|e| {
            let mut it = Item::new(src(e));
            it.qty = i(e, "qty").unwrap_or(1).max(1) as u32;
            it.equipped = e.get("equipped").and_then(Value::as_bool).unwrap_or(false);
            it
        })
        .collect();
    ch.ledger = list(doc, "ledger")
        .enumerate()
        .map(|(n, l)| {
            let amount = i(l, "delta").unwrap_or(0) as i32;
            let kind = match l.get("kind").and_then(Value::as_str) {
                Some("adjust") => LedgerKind::Correction,
                Some("spend") | Some("refund") => LedgerKind::Other,
                _ if n == 0 => LedgerKind::Starting,
                _ => LedgerKind::SessionAward,
            };
            LedgerEntry {
                id: n as u32 + 1,
                date: String::new(),
                amount,
                kind,
                note: if kind == LedgerKind::Other {
                    "перенос".into()
                } else {
                    String::new()
                },
                skill: None,
            }
        })
        .collect();
    ch
}
