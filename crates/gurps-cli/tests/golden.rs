//! Golden tests: test characters supplied by the GM (examples/test-chars).
//! Expected values are the GM's; do not adjust them to match the code.

use gurps_core::Stat;
use gurps_rules::{compute_sheet, Sheet};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sheet(name: &str) -> Sheet {
    let p = root().join(format!("examples/test-chars/{name}.json"));
    let ch = gurps_io::load(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    assert!(
        gurps_rules::validate::validate(&ch).is_empty(),
        "{name} has validation issues"
    );
    compute_sheet(&ch)
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
    attrs: i32,
    secondary: i32,
    adv: i32,
    disadv: i32,
    total: i32,
}

fn check(name: &str, e: Expect) {
    let s = sheet(name);
    let got = [
        ("HP", s.total(Stat::HP), e.hp),
        ("Will", s.total(Stat::Will), e.will),
        ("Per", s.total(Stat::Per), e.per),
        ("FP", s.total(Stat::FP), e.fp),
        ("Speed", s.total(Stat::BasicSpeed), e.speed),
        ("Move", s.total(Stat::BasicMove), e.mv),
        ("BL", s.total(Stat::BasicLift), e.bl),
    ];
    for (k, g, x) in got {
        assert_eq!(g, x, "{name}: {k}");
    }
    assert_eq!(s.damage.thrust, e.thr, "{name}: thrust");
    assert_eq!(s.damage.swing, e.sw, "{name}: swing");
    let p = &s.points;
    assert_eq!(p.attributes, e.attrs, "{name}: attribute points");
    assert_eq!(p.secondary, e.secondary, "{name}: secondary points");
    assert_eq!(p.advantages + p.perks, e.adv, "{name}: advantage points");
    assert_eq!(
        p.disadvantages + p.quirks,
        e.disadv,
        "{name}: disadvantage points"
    );
    assert_eq!(p.total, e.total, "{name}: total points");
}

#[test]
fn warrior() {
    check(
        "warrior",
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
    // Lifting ST affects only Basic Lift.
    let s = sheet("warrior");
    assert_eq!(s.total(Stat::ST), 13.0);
    assert_eq!(s.damage.based_on_st, 13);
}

#[test]
fn mage() {
    check(
        "mage",
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
}

#[test]
fn rogue() {
    check(
        "rogue",
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
    // Striking ST affects only damage: ST stays 11, damage uses ST 13.
    let s = sheet("rogue");
    assert_eq!(s.total(Stat::ST), 11.0);
    assert_eq!(s.total(Stat::HP), 10.0);
    assert_eq!(s.damage.based_on_st, 13);
    assert_eq!(s.damage.explain.last().unwrap(), "= 13");
}

/// Fixture trait snapshots must match the sample plugin library
/// (explanation text is presentation only and is not part of the snapshot).
#[test]
fn fixtures_match_plugin_library() {
    let plugins = gurps_plugins::load_all(&root().join("plugins")).unwrap();
    let lib = gurps_plugins::trait_library(&plugins);
    let skills = gurps_plugins::skill_library(&plugins);
    let techs = gurps_plugins::technique_library(&plugins);
    let equip = gurps_plugins::equipment_library(&plugins);
    let weapons = gurps_plugins::weapon_library(&plugins);
    for name in [
        "warrior",
        "mage",
        "rogue",
        "warrior-skills",
        "rogue-skills",
        "warrior-equipment",
        "rogue-equipment",
        "warrior-combat",
        "rogue-combat",
    ] {
        let ch = gurps_io::load(root().join(format!("examples/test-chars/{name}.json"))).unwrap();
        for t in &ch.traits {
            let (_, lib_def) = lib
                .iter()
                .find(|(_, d)| d.id == t.def.id)
                .unwrap_or_else(|| panic!("{name}: trait {} not in library", t.def.id));
            let mut def = lib_def.clone();
            def.description = None;
            assert_eq!(
                t.def, def,
                "{name}: trait {} differs from library",
                t.def.id
            );
        }
        for s in &ch.skills {
            let (_, d) = skills
                .iter()
                .find(|(_, d)| d.id == s.def.id)
                .unwrap_or_else(|| panic!("{name}: skill {} not in library", s.def.id));
            let mut d = d.clone();
            d.description = None;
            assert_eq!(s.def, d, "{name}: skill {} differs from library", s.def.id);
        }
        for w in &ch.weapons {
            let (_, d) = weapons
                .iter()
                .find(|(_, d)| d.id == w.def.id)
                .unwrap_or_else(|| panic!("{name}: weapon {} not in library", w.def.id));
            let mut d = d.clone();
            d.description = None;
            assert_eq!(w.def, d, "{name}: weapon {} differs", w.def.id);
        }
        for t in &ch.techniques {
            let (_, d) = techs
                .iter()
                .find(|(_, d)| d.id == t.def.id)
                .unwrap_or_else(|| panic!("{name}: technique {} not in library", t.def.id));
            let mut d = d.clone();
            d.description = None;
            assert_eq!(t.def, d, "{name}: technique {} differs", t.def.id);
        }
        fn walk(list: &[gurps_core::Equipment], f: &mut dyn FnMut(&gurps_core::Equipment)) {
            for e in list {
                f(e);
                walk(&e.contents, f);
            }
        }
        walk(&ch.equipment, &mut |e| {
            let (_, d) = equip
                .iter()
                .find(|(_, d)| d.id == e.def.id)
                .unwrap_or_else(|| panic!("{name}: equipment {} not in library", e.def.id));
            let mut d = d.clone();
            d.description = None;
            assert_eq!(e.def, d, "{name}: equipment {} differs", e.def.id);
        });
    }
}

/// Skill levels for the GM's stage-2 data (docs/test-data-stage2-3.md).
fn skill_levels(name: &str) -> (Sheet, Vec<(String, Option<i32>)>) {
    let s = sheet(name);
    let lv = s
        .skills
        .iter()
        .map(|r| (r.name.clone(), r.level))
        .chain(s.techniques.iter().map(|r| (r.name.clone(), r.level)))
        .collect();
    (s, lv)
}

#[test]
fn warrior_skills() {
    let (s, lv) = skill_levels("warrior-skills");
    let want = [
        ("Broadsword", 14),
        ("Shield", 14),
        ("Brawling", 12),
        ("Survival (Woodlands)", 11),
        ("Tracking", 11),
        ("Stealth", 7),
        ("Kicking (Brawling)", 11),
    ];
    let want: Vec<_> = want
        .iter()
        .map(|(n, l)| (n.to_string(), Some(*l)))
        .collect();
    assert_eq!(lv, want);
    let surv = &s.skills[3];
    assert_eq!((surv.bonus, surv.relative.as_deref()), (1, Some("Per+1")));
    assert!(s.skills[5].from_default, "Stealth at default DX-5");
    let kick = &s.techniques[0];
    assert_eq!(kick.max_level, Some(12), "technique cap = Brawling");
    assert_eq!(s.points.skills, 19);
    assert_eq!(s.points.advantages + s.points.perks, 31 + 10, "+ talent 10");
    assert_eq!(s.points.total, 134);
    // attributes and damage unchanged from the base warrior
    assert_eq!(s.total(Stat::BasicLift), 45.0);
    assert_eq!(
        (s.damage.thrust.as_str(), s.damage.swing.as_str()),
        ("1d", "2d-1")
    );
}

#[test]
fn rogue_skills() {
    let (s, lv) = skill_levels("rogue-skills");
    assert_eq!(
        lv,
        vec![
            ("Knife".to_string(), Some(15)),
            ("Climbing".to_string(), Some(9))
        ]
    );
    assert!(s.skills[1].from_default);
    assert_eq!(s.points.skills, 2);
    assert_eq!(s.points.total, 144);
}

/// Encumbrance from the GM's stage-2 data (docs/test-data-stage2-3.md).
#[test]
fn warrior_encumbrance() {
    let s = sheet("warrior-equipment");
    let e = &s.encumbrance;
    assert_eq!((e.carried_weight, e.total_cost), (46.0, 875.0));
    assert_eq!(e.basic_lift, 45.0);
    assert_eq!((e.level, e.name.as_str()), (1, "Light"), "46 lb > BL 45");
    assert_eq!(e.base_move, 7.0);
    assert_eq!(e.move_value, 5, "floor(7 * 0.8)");
    assert_eq!(e.dodge_penalty, -1);
    // Dodge without Combat Reflexes: floor(6.00) + 3 = 9, encumbered 8 (GM: "without CR ... 8")
    assert_eq!((e.base_dodge, e.dodge), (9, 8));
    // points are unchanged by equipment
    assert_eq!(s.points.total, 134);
}

#[test]
fn rogue_encumbrance() {
    let s = sheet("rogue-equipment");
    let e = &s.encumbrance;
    assert_eq!((e.carried_weight, e.total_cost), (5.5, 140.0));
    assert_eq!((e.level, e.name.as_str()), (0, "None"));
    assert_eq!(e.basic_lift, 24.0);
    assert_eq!(e.move_value, 7);
    assert_eq!(e.dodge, 9, "floor(6.75) + 3");
    assert_eq!(s.points.total, 144);
}

#[test]
fn not_carried_pack_drops_encumbrance() {
    let p = root().join("examples/test-chars/warrior-equipment.json");
    let mut ch = gurps_io::load(&p).unwrap();
    let pack = ch.equipment.iter_mut().find(|e| e.def.container).unwrap();
    pack.carried = false; // pack 10 + basics 1 + knives 1 left behind
    let e = compute_sheet(&ch).encumbrance;
    assert_eq!((e.carried_weight, e.total_weight, e.level), (34.0, 46.0, 0));
    assert_eq!(e.move_value, 7);
}

fn attack<'a>(s: &'a Sheet, weapon: &str, mode: &str) -> &'a gurps_rules::combat::AttackRow {
    s.weapons
        .iter()
        .find(|w| w.id == weapon)
        .unwrap_or_else(|| panic!("weapon {weapon}"))
        .attacks
        .iter()
        .find(|a| a.mode == mode)
        .unwrap_or_else(|| panic!("{weapon} {mode}"))
}

fn dmg(a: &gurps_rules::combat::AttackRow) -> String {
    format!("{} {}", a.damage, a.damage_type)
}

#[test]
fn warrior_weapons_and_defences() {
    let s = sheet("warrior-combat");
    // GM: Broadsword sw+1 cut = 2d cut, thr+1 cr = 1d+1 cr; Kick 1d cr; Punch 1d-1 cr
    assert_eq!(dmg(attack(&s, "broadsword", "swing")), "2d cut");
    assert_eq!(dmg(attack(&s, "broadsword", "thrust")), "1d+1 cr");
    assert_eq!(
        dmg(attack(&s, "kick", "kick")),
        "1d cr",
        "no Brawling bonus at DX+0"
    );
    assert_eq!(dmg(attack(&s, "punch", "punch")), "1d-1 cr");
    assert_eq!(
        attack(&s, "kick", "kick").level,
        Some(11),
        "Kicking technique"
    );
    assert_eq!(attack(&s, "kick", "kick").parry, None);
    assert_eq!(attack(&s, "broadsword", "swing").parry, Some(11));
    let d = &s.defenses;
    assert_eq!(
        d.parry.as_ref().map(|p| (p.value, p.from.as_str())),
        Some((11, "Broadsword"))
    );
    assert_eq!(d.block.as_ref().map(|b| b.value), Some(11));
    assert_eq!(d.dodge.value, 9, "Dodge 10 with CR, -1 Light encumbrance");
    assert_eq!(d.db, 2);
    assert_eq!(
        d.dodge.value + d.db,
        11,
        "GM: Dodge CR + encumbered + DB = 11"
    );
    assert_eq!(d.bonus, 1);
    assert!(d
        .armor
        .iter()
        .any(|a| a.item == "Mail Shirt" && a.dr == "4/2*" && a.value == 4));
    assert_eq!(s.points.total, 134, "weapons do not change points");
}

#[test]
fn warrior_defences_unencumbered_and_without_cr() {
    let p = root().join("examples/test-chars/warrior-combat.json");
    let mut ch = gurps_io::load(&p).unwrap();
    // drop the backpack (and its contents): 46 - 10 - 1 - 1 = 34 lb <= BL 45
    ch.equipment.retain(|e| e.def.id != "backpack-frame");
    let s = compute_sheet(&ch);
    assert_eq!(s.encumbrance.level, 0);
    assert_eq!(
        s.defenses.dodge.value, 10,
        "GM: Dodge 10 unencumbered with CR"
    );
    ch.traits.retain(|t| t.def.id != "combat-reflexes");
    let s = compute_sheet(&ch);
    let d = &s.defenses;
    // GM: without CR 10/10/9 (Parry/Block/Dodge)
    assert_eq!(
        (
            d.parry.as_ref().unwrap().value,
            d.block.as_ref().unwrap().value,
            d.dodge.value
        ),
        (10, 10, 9)
    );
}

#[test]
fn rogue_weapons_and_defences() {
    let s = sheet("rogue-combat");
    assert_eq!(dmg(attack(&s, "small-knife", "thrust")), "1d-1 imp");
    assert_eq!(dmg(attack(&s, "small-knife", "swing")), "2d-4 cut");
    let d = &s.defenses;
    assert_eq!(
        d.parry.as_ref().map(|p| (p.value, p.from.as_str())),
        Some((9, "Small Knife")),
        "floor(15/2)+3-1"
    );
    assert_eq!(d.dodge.value, 9);
    assert!(d.block.is_none());
    assert_eq!(d.db, 0);
    // unarmed parry from plain DX: floor(14/2)+3 = 10 (shown on the row, not as the sheet's Parry)
    assert_eq!(attack(&s, "punch", "punch").parry, Some(10));
    assert!(d
        .armor
        .iter()
        .any(|a| a.item == "Leather Jacket" && a.value == 1));
    assert_eq!(s.points.total, 144);
}

#[test]
fn brawling_damage_bonus_at_dx_plus_2() {
    let p = root().join("examples/test-chars/warrior-combat.json");
    let mut ch = gurps_io::load(&p).unwrap();
    let b = ch
        .skills
        .iter_mut()
        .find(|s| s.def.id == "brawling")
        .unwrap();
    b.points = 4; // DX/E 4 pt = DX+2 = 14
    let s = compute_sheet(&ch);
    assert_eq!(
        dmg(attack(&s, "punch", "punch")),
        "1d cr",
        "1d-1 +1 per die"
    );
    assert_eq!(dmg(attack(&s, "kick", "kick")), "1d+1 cr");
    b_reset(&mut ch);
    let s = compute_sheet(&ch);
    assert_eq!(
        dmg(attack(&s, "punch", "punch")),
        "1d-1 cr",
        "DX+1 is not enough"
    );
}

fn b_reset(ch: &mut gurps_core::Character) {
    ch.skills
        .iter_mut()
        .find(|s| s.def.id == "brawling")
        .unwrap()
        .points = 2; // DX+1
}
