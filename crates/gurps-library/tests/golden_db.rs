//! Golden-тесты этапа 4 на БД: старые фикстуры examples/test-chars импортируются
//! через sheet-bridge в тестовую БД (custom-пак персонажа = тестовый пак),
//! затем gurps_db::calc_input::load -> gurps_calc::compute.
//! Ожидания — из прежних crates/gurps-cli/tests/golden.rs и spells_modifiers.rs без изменений.
//! Соглашение gurps-calc: Dodge/Parry/Block включают DB щита, поэтому «листовые» значения = значение − db.
//! Без сервера SurrealDB (testkit) тесты молча пропускаются.

use gurps_db::gurps_calc::{self, Computed};
use gurps_db::testkit;
use serde_json::{json, Value};
use std::path::PathBuf;

struct Env {
    _srv: testkit::Server,
    db: gurps_db::Db,
    bdb: sheet_bridge::db::Db,
}

async fn env() -> Option<Env> {
    let (srv, db) = testkit::start().await?;
    let c = &srv.cfg;
    let bcfg = sheet_bridge::config::DbConfig {
        url: c.host.clone(),
        ns: c.namespace.clone(),
        db: c.database.clone(),
        user: c.user.clone(),
        pass: c.password.clone(),
    };
    let bdb = sheet_bridge::db::Db::connect(&bcfg).await.unwrap();
    // Тестовый справочник: в сиде колледж называется light_darkness, импорт ищет по имени листа.
    bdb.one(
        "UPSERT ref_college:light_and_darkness SET code = 'light_and_darkness', pack = pack:basic;",
    )
    .await
    .unwrap();
    Some(Env { _srv: srv, db, bdb })
}

fn fixture(name: &str) -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../examples/test-chars/{name}.json"));
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

impl Env {
    async fn calc(&self, id: &str, sheet: &Value) -> Computed {
        let f = std::env::temp_dir().join(format!("golden_db_{}_{id}.json", std::process::id()));
        std::fs::write(&f, serde_json::to_string(sheet).unwrap()).unwrap();
        let o = sheet_bridge::character::import::CharOpts {
            id: Some(id.into()),
            tl: "TL3".into(),
            spell_class: "regular".into(),
            spell_system: "standard".into(),
        };
        sheet_bridge::character::import::run(&self.bdb, &f, &o, false)
            .await
            .unwrap();
        let _ = std::fs::remove_file(&f);
        let inp = gurps_db::calc_input::load(&self.db, id).await.unwrap();
        let out = gurps_calc::compute(&inp);
        assert!(
            out.missing_constants.0.is_empty(),
            "{id}: {:?}",
            out.missing_constants
        );
        out
    }
    async fn named(&self, name: &str) -> Computed {
        self.calc(&name.replace('-', "_"), &fixture(name)).await
    }
}

fn dice(d: &impl serde::Serialize) -> String {
    let v = serde_json::to_value(d).unwrap();
    let (n, m) = (v["dice"].as_i64().unwrap(), v["mod"].as_i64().unwrap());
    match m {
        0 => format!("{n}d"),
        m if m > 0 => format!("{n}d+{m}"),
        m => format!("{n}d{m}"),
    }
}

fn attack<'a>(s: &'a Computed, item: Option<&str>, sid: &str) -> &'a gurps_calc::attacks::Attack {
    s.attacks
        .iter()
        .find(|a| a.item.as_deref() == item && (a.sid == sid || a.name.as_deref() == Some(sid)))
        .unwrap_or_else(|| panic!("attack {item:?} {sid}"))
}

fn dmg(a: &gurps_calc::attacks::Attack) -> String {
    format!(
        "{} {}",
        a.damage_text.as_deref().unwrap_or(""),
        a.damage_type.as_deref().unwrap_or("")
    )
}

/// Нагрузка как JSON (поля в gurps-calc — Option).
struct En {
    carried_weight: f64,
    basic_lift: f64,
    level: i64,
    code: String,
    base_move: f64,
    move_value: i64,
    base_dodge: i64,
    dodge: i64,
}

fn en(s: &Computed) -> En {
    let v = serde_json::to_value(&s.encumbrance).unwrap();
    En {
        carried_weight: v["carried_weight"].as_f64().unwrap(),
        basic_lift: v["basic_lift"].as_f64().unwrap(),
        level: v["level"].as_i64().unwrap(),
        code: v["code"].as_str().unwrap().into(),
        base_move: v["base_move"].as_f64().unwrap(),
        move_value: v["move_value"].as_i64().unwrap(),
        base_dodge: v["base_dodge"].as_i64().unwrap(),
        dodge: v["dodge"].as_i64().unwrap(),
    }
}

fn level(s: &Computed, sid: &str) -> Option<i64> {
    s.skills
        .iter()
        .chain(&s.techniques)
        .chain(&s.spells)
        .find(|r| r.sid == sid)
        .unwrap_or_else(|| panic!("{sid}"))
        .level
}

struct Expect {
    hp: f64,
    will: f64,
    per: f64,
    fp: f64,
    speed: f64,
    mv: f64,
    bl: f64,
    thr: &'static str,
    sw: &'static str,
    attrs: i64,
    secondary: i64,
    adv: i64,
    disadv: i64,
    total: i64,
}

fn check(name: &str, s: &Computed, e: Expect) {
    let a = &s.attributes;
    for (k, x) in [
        ("HP", e.hp),
        ("Will", e.will),
        ("Per", e.per),
        ("FP", e.fp),
        ("Speed", e.speed),
        ("Move", e.mv),
    ] {
        assert_eq!(a.get(k), Some(x), "{name}: {k}");
    }
    assert_eq!(a.basic_lift, Some(e.bl), "{name}: BL");
    assert_eq!(dice(a.thrust.as_ref().unwrap()), e.thr, "{name}: thrust");
    assert_eq!(dice(a.swing.as_ref().unwrap()), e.sw, "{name}: swing");
    let p = &s.points;
    assert_eq!(p.basic_attributes, e.attrs, "{name}: attribute points");
    assert_eq!(
        p.secondary_attributes, e.secondary,
        "{name}: secondary points"
    );
    assert_eq!(p.advantages, e.adv, "{name}: advantages + perks");
    assert_eq!(p.disadvantages, e.disadv, "{name}: disadvantages + quirks");
    assert_eq!(p.total, e.total, "{name}: total");
}

#[tokio::test(flavor = "multi_thread")]
async fn base_characters() {
    let Some(env) = env().await else { return };
    let w = env.named("warrior").await;
    check(
        "warrior",
        &w,
        Expect {
            hp: 15.0,
            will: 10.0,
            per: 10.0,
            fp: 12.0,
            speed: 6.0,
            mv: 7.0,
            bl: 45.0,
            thr: "1d",
            sw: "2d-1",
            attrs: 90,
            secondary: 9,
            adv: 31,
            disadv: -25,
            total: 105,
        },
    );
    // Lifting ST — только BL.
    assert_eq!(w.attributes.get("ST"), Some(13.0));
    assert_eq!(w.attributes.striking_st.unwrap_or(13.0), 13.0);
    let m = env.named("mage").await;
    check(
        "mage",
        &m,
        Expect {
            hp: 11.0,
            will: 15.0,
            per: 12.0,
            fp: 14.0,
            speed: 5.25,
            mv: 5.0,
            bl: 16.0,
            thr: "1d-2",
            sw: "1d-1",
            attrs: 80,
            secondary: 8,
            adv: 30,
            disadv: -25,
            total: 93,
        },
    );
    let r = env.named("rogue").await;
    check(
        "rogue",
        &r,
        Expect {
            hp: 10.0,
            will: 11.0,
            per: 13.0,
            fp: 10.0,
            speed: 6.75,
            mv: 7.0,
            bl: 24.0,
            thr: "1d",
            sw: "2d-1",
            attrs: 110,
            secondary: 28,
            adv: 29,
            disadv: -25,
            total: 142,
        },
    );
    // Striking ST — только урон.
    assert_eq!(r.attributes.get("ST"), Some(11.0));
    assert_eq!(r.attributes.get("HP"), Some(10.0));
    assert_eq!(r.attributes.striking_st, Some(13.0));
}

#[tokio::test(flavor = "multi_thread")]
async fn skills() {
    let Some(env) = env().await else { return };
    let s = env.named("warrior-skills").await;
    for (sid, l) in [
        ("broadsword", 14),
        ("shield", 14),
        ("brawling", 12),
        ("survival", 11),
        ("tracking", 11),
        ("stealth", 7),
        ("kicking", 11),
    ] {
        assert_eq!(level(&s, sid), Some(l), "{sid}");
    }
    let surv = s.skills.iter().find(|r| r.sid == "survival").unwrap();
    assert_eq!(
        (surv.spec.as_deref(), surv.relative),
        (Some("woodlands"), Some(1)),
        "Per+1 (Outdoorsman)"
    );
    assert!(
        s.skills
            .iter()
            .find(|r| r.sid == "stealth")
            .unwrap()
            .source
            .starts_with("default:"),
        "Stealth по умолчанию DX-5"
    );
    assert_eq!(s.points.skills + s.points.techniques, 19);
    assert_eq!(s.points.advantages, 31 + 10, "+ талант 10");
    assert_eq!(s.points.total, 134);
    assert_eq!(s.attributes.basic_lift, Some(45.0));
    let r = env.named("rogue-skills").await;
    assert_eq!(
        (level(&r, "knife"), level(&r, "climbing")),
        (Some(15), Some(9))
    );
    assert!(r
        .skills
        .iter()
        .find(|x| x.sid == "climbing")
        .unwrap()
        .source
        .starts_with("default:"));
    assert_eq!(r.points.skills, 2);
    assert_eq!(r.points.total, 144);
}

#[tokio::test(flavor = "multi_thread")]
async fn encumbrance() {
    let Some(env) = env().await else { return };
    let s = env.named("warrior-equipment").await;
    let e = en(&s);
    assert_eq!((e.carried_weight, e.basic_lift), (46.0, 45.0));
    assert_eq!((e.level, e.code.as_str()), (1, "light"), "46 lb > BL 45");
    assert_eq!((e.base_move, e.move_value), (7.0, 5));
    assert_eq!((e.base_dodge, e.dodge), (9, 8));
    assert_eq!(s.points.total, 134);
    let r = env.named("rogue-equipment").await;
    let e = en(&r);
    assert_eq!(
        (e.carried_weight, e.level, e.code.as_str()),
        (5.5, 0, "none")
    );
    assert_eq!((e.basic_lift, e.move_value, e.dodge), (24.0, 7, 9));
    assert_eq!(r.points.total, 144);
    // Рюкзак не при себе: 46 − 10 − 1 − 1 = 34.
    let mut ch = fixture("warrior-equipment");
    for it in ch["character"]["equipment"].as_array_mut().unwrap() {
        if it["container"] == json!(true) {
            it["carried"] = json!(false);
        }
    }
    let s = env.calc("warrior_no_pack", &ch).await;
    assert_eq!(
        (en(&s).carried_weight, en(&s).level, en(&s).move_value),
        (34.0, 0, 7)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn warrior_weapons_and_defences() {
    let Some(env) = env().await else { return };
    let s = env.named("warrior-combat").await;
    let sw = attack(&s, Some("broadsword"), "swing");
    assert_eq!(dmg(sw), "2d cut");
    assert_eq!(dmg(attack(&s, Some("broadsword"), "thrust")), "1d+1 cr");
    assert_eq!(sw.parry, Some(11));
    let d = &s.defenses;
    assert_eq!(d.db, 2);
    assert_eq!(
        (d.parry.map(|p| p - d.db), d.parry_item.as_deref()),
        (Some(11), Some("broadsword"))
    );
    assert_eq!(d.block.map(|b| b - d.db), Some(11));
    assert_eq!(d.dodge.unwrap() - d.db, 9, "Dodge 10 с CR, −1 Light");
    assert_eq!(d.dodge, Some(11), "Dodge CR + нагрузка + DB = 11");
    assert_eq!(d.dodge_bonus, 1);
    assert_eq!(attack(&s, None, "kick").parry, None);
    assert_eq!(s.points.total, 134);

    let mut ch = fixture("warrior-combat");
    ch["character"]["equipment"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["id"] != json!("backpack-frame"));
    let s = env.calc("warrior_light", &ch).await;
    assert_eq!(en(&s).level, 0);
    assert_eq!(
        s.defenses.dodge.unwrap() - s.defenses.db,
        10,
        "Dodge 10 без нагрузки с CR"
    );
    ch["character"]["traits"]
        .as_array_mut()
        .unwrap()
        .retain(|t| t["id"] != json!("combat-reflexes"));
    let s = env.calc("warrior_no_cr", &ch).await;
    let d = &s.defenses;
    assert_eq!(
        (
            d.parry.unwrap() - d.db,
            d.block.unwrap() - d.db,
            d.dodge.unwrap() - d.db
        ),
        (10, 10, 9)
    );
}

/// Ждёт сида Мастера: натуральные атаки basic (punch thr-1, kick thr) — заглушки (mod 0),
/// и связи техники Kicking с атакой kick в схеме нет.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "заглушки attack_mode punch/kick в сиде; техника -> атака не связана"]
async fn warrior_unarmed_pending() {
    let Some(env) = env().await else { return };
    let s = env.named("warrior-combat").await;
    assert_eq!(dmg(attack(&s, None, "kick")), "1d cr");
    assert_eq!(dmg(attack(&s, None, "punch")), "1d-1 cr");
    assert_eq!(attack(&s, None, "kick").level, Some(11), "Kicking");
    let mut ch = fixture("warrior-combat");
    for sk in ch["character"]["skills"].as_array_mut().unwrap() {
        if sk["id"] == json!("brawling") {
            sk["points"] = json!(4);
        }
    }
    let s = env.calc("warrior_brawl4", &ch).await;
    assert_eq!(dmg(attack(&s, None, "punch")), "1d cr");
    assert_eq!(dmg(attack(&s, None, "kick")), "1d+1 cr");
}

#[tokio::test(flavor = "multi_thread")]
async fn rogue_weapons_and_defences() {
    let Some(env) = env().await else { return };
    let s = env.named("rogue-combat").await;
    assert_eq!(dmg(attack(&s, Some("small_knife"), "thrust")), "1d-1 imp");
    assert_eq!(dmg(attack(&s, Some("small_knife"), "swing")), "2d-4 cut");
    let d = &s.defenses;
    assert_eq!(
        (d.parry, d.parry_item.as_deref()),
        (Some(9), Some("small_knife")),
        "floor(15/2)+3-1"
    );
    assert_eq!((d.dodge, d.block, d.db), (Some(9), None, 0));
    assert_eq!(attack(&s, None, "punch").parry, Some(10));
    assert_eq!(s.points.total, 144);
}

#[tokio::test(flavor = "multi_thread")]
async fn spells_and_modifiers() {
    let Some(env) = env().await else { return };
    let s = env.named("mage-spells").await;
    for (sid, l) in [("light", 14), ("fireball", 16), ("teleport", 16)] {
        assert_eq!(level(&s, sid), Some(l), "{sid}");
    }
    assert_eq!(s.points.spells, 13);
    // Eidetic Memory 5 −30% = 3.5 -> 4: итог 93 + 13 − 1.
    assert_eq!(s.points.total, 93 + 13 - 1);
    assert_eq!(s.points.advantages, 30 - 1);

    // Пол −80%: Magery 2 = 25, −110% -> −80% = 5 (преимущества 29 − 25 + 5).
    let md = |id: &str, name: &str, pct: i64| json!({"id": id, "name": name, "cost_percent": pct, "source": "basic-set-sample"});
    let mut ch = fixture("mage-spells");
    let set = |ch: &mut Value, mods: Vec<Value>| {
        for t in ch["character"]["traits"].as_array_mut().unwrap() {
            if t["id"] == json!("magery") {
                t["modifiers"] = json!(mods);
            }
        }
    };
    let eo = md("emergencies-only", "Emergencies Only", -30);
    let acc = md("accessibility", "Accessibility", -10);
    set(&mut ch, vec![eo.clone(), eo.clone(), eo, acc.clone(), acc]);
    let s = env.calc("mage_floor", &ch).await;
    assert_eq!(s.points.advantages, 29 - 25 + 5);
    // 25 × (100 + 50 − 5)% = 36.25 -> 37.
    set(
        &mut ch,
        vec![
            md("area-effect", "Area Effect", 50),
            md("costs-fatigue", "Costs Fatigue", -5),
        ],
    );
    let s = env.calc("mage_mix", &ch).await;
    assert_eq!(s.points.advantages, 29 - 25 + 37);
}
