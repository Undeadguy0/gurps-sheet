//! Stage 4: spells and trait modifiers.
//! NOT GM data: expected values below are our own reading of the rules
//! (spells = IQ/H or IQ/VH skills + Magery level; modifiers B101-B102, unverified).

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn mage_spells_and_modified_trait() {
    let ch = gurps_io::load(root().join("examples/test-chars/mage-spells.json")).unwrap();
    assert!(gurps_rules::validate::validate(&ch).is_empty());
    let s = gurps_rules::compute_sheet(&ch);
    let lv: Vec<_> = s.spells.iter().map(|r| (r.id.as_str(), r.level)).collect();
    // IQ 14, Magery 2: Light 1 pt H = IQ-2 +2 = 14; Fireball 4 pts H = IQ+0 +2 = 16;
    // Teleport 8 pts VH = IQ+0 +2 = 16.
    assert_eq!(
        lv,
        [
            ("light", Some(14)),
            ("fireball", Some(16)),
            ("teleport", Some(16))
        ]
    );
    assert_eq!(s.spells[0].explain[2], "+2 from trait Magery 2");
    assert_eq!(s.points.spells, 13);
    // Eidetic Memory 5 with Emergencies Only -30%: 3.5 -> 4.
    let em = ch
        .traits
        .iter()
        .find(|t| t.def.id == "eidetic-memory")
        .unwrap();
    assert_eq!(em.cost(), 4);
    assert_eq!(
        gurps_rules::modifiers::trait_cost_explain(em),
        [
            "base cost 5",
            "-30% Emergencies Only",
            "total -30%",
            "5 × 70% = 3.5 -> 4 (rounded up)"
        ]
    );
    // GM mage total 93: + 13 spell points, -1 from the modifier.
    assert_eq!(s.points.total, 93 + 13 - 1);
    assert_eq!(s.points.advantages, 30 - 1);
}

#[test]
fn modifier_floor_minus_80() {
    let mut ch = gurps_io::load(root().join("examples/test-chars/mage-spells.json")).unwrap();
    let lib =
        gurps_plugins::modifier_library(&gurps_plugins::load_all(&root().join("plugins")).unwrap());
    let m = |id: &str| {
        gurps_core::TraitModifier::new(lib.iter().find(|(_, d)| d.id == id).unwrap().1.clone())
    };
    // Magery 2 = 25 pts; -30 -30 -10 -10 ... = -110% floored to -80%: 25 × 20% = 5.
    let mg = ch.traits.iter_mut().find(|t| t.def.id == "magery").unwrap();
    mg.modifiers = vec![
        m("emergencies-only"),
        m("emergencies-only"),
        m("emergencies-only"),
        m("accessibility"),
        m("accessibility"),
    ];
    assert_eq!(mg.base_cost(), 25);
    assert_eq!(mg.cost(), 5);
    let ex = gurps_rules::modifiers::trait_cost_explain(mg);
    assert!(
        ex.contains(&"total -110% floored at -80%".to_string()),
        "{ex:?}"
    );
    // enhancement + limitation: 25 × (100 + 50 - 5)% = 36.25 -> 37
    mg.modifiers = vec![m("area-effect"), m("costs-fatigue")];
    assert_eq!(mg.cost(), 37);
}
