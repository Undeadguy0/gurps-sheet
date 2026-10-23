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
    for name in ["warrior", "mage", "rogue", "warrior-skills", "rogue-skills"] {
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
        for t in &ch.techniques {
            let (_, d) = techs
                .iter()
                .find(|(_, d)| d.id == t.def.id)
                .unwrap_or_else(|| panic!("{name}: technique {} not in library", t.def.id));
            let mut d = d.clone();
            d.description = None;
            assert_eq!(t.def, d, "{name}: technique {} differs", t.def.id);
        }
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
