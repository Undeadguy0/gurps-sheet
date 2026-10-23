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

/// Fixture trait snapshots must match the sample plugin library.
#[test]
fn fixtures_match_plugin_library() {
    let plugins = gurps_plugins::load_all(&root().join("plugins")).unwrap();
    let lib = gurps_plugins::trait_library(&plugins);
    for name in ["warrior", "mage", "rogue"] {
        let ch = gurps_io::load(root().join(format!("examples/test-chars/{name}.json"))).unwrap();
        for t in &ch.traits {
            let (_, def) = lib
                .iter()
                .find(|(_, d)| d.id == t.def.id)
                .unwrap_or_else(|| panic!("{name}: trait {} not in library", t.def.id));
            assert_eq!(
                &t.def, def,
                "{name}: trait {} differs from library",
                t.def.id
            );
        }
    }
}
