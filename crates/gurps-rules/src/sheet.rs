use crate::points::{self, PointsBreakdown};
use crate::value::{fmt_num, SourceKind, Value};
use gurps_core::damage::basic_damage;
use gurps_core::{Character, Feature, Stat};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One row of the attributes table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatRow {
    pub key: String,
    pub label: String,
    /// Purchased steps (Basic Speed: quarter steps).
    pub level: i32,
    pub points: i32,
    pub total: f64,
    pub value: Value,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageRow {
    pub thrust: String,
    pub swing: String,
    /// ST used for the damage table (ST + Striking ST).
    pub based_on_st: i32,
    /// How `based_on_st` was obtained.
    pub st_value: Value,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sheet {
    pub name: String,
    pub attributes: Vec<StatRow>,
    pub secondary: Vec<StatRow>,
    pub basic_lift: StatRow,
    pub damage: DamageRow,
    pub points: PointsBreakdown,
}

impl Sheet {
    pub fn row(&self, stat: Stat) -> Option<&StatRow> {
        let k = stat.key();
        self.attributes
            .iter()
            .chain(self.secondary.iter())
            .chain(std::iter::once(&self.basic_lift))
            .find(|r| r.key == k)
    }

    pub fn total(&self, stat: Stat) -> f64 {
        self.row(stat).map(|r| r.total).unwrap_or(0.0)
    }
}

type Named = Vec<(String, f64)>;

/// Bonus contributions from traits, grouped by target.
struct Bonuses {
    by_stat: BTreeMap<Stat, Named>,
    lifting: Named,
    striking: Named,
}

fn trait_bonuses(ch: &Character) -> Bonuses {
    let mut by_stat: BTreeMap<Stat, Named> = BTreeMap::new();
    let mut lifting = Vec::new();
    let mut striking = Vec::new();
    for t in &ch.traits {
        let lv = t.levels.max(1) as f64;
        for f in &t.def.features {
            match f {
                Feature::AttributeBonus {
                    target,
                    amount,
                    per_level,
                } => {
                    let a = if *per_level { amount * lv } else { *amount };
                    by_stat
                        .entry(*target)
                        .or_default()
                        .push((t.display_name(), a));
                }
                Feature::LiftingStBonus { amount, per_level } => {
                    let a = if *per_level { amount * lv } else { *amount };
                    lifting.push((t.display_name(), a));
                }
                Feature::StrikingStBonus { amount, per_level } => {
                    let a = if *per_level { amount * lv } else { *amount };
                    striking.push((t.display_name(), a));
                }
            }
        }
    }
    Bonuses {
        by_stat,
        lifting,
        striking,
    }
}

fn apply_common(v: &mut Value, stat: Stat, ch: &Character, bonuses: &BTreeMap<Stat, Named>) {
    let purchased = ch.attributes.purchased(stat);
    let amount = purchased as f64 * stat.step();
    v.add(
        SourceKind::Purchased,
        amount,
        format!("purchased ({} pts)", points::stat_points(stat, purchased)),
    );
    if let Some(list) = bonuses.get(&stat) {
        for (name, a) in list {
            v.add(
                SourceKind::Trait(name.clone()),
                *a,
                format!("from trait {name}"),
            );
        }
    }
}

fn row(stat: Stat, ch: &Character, value: Value) -> StatRow {
    let level = ch.attributes.purchased(stat);
    StatRow {
        key: stat.key().into(),
        label: stat.label().into(),
        level,
        points: points::stat_points(stat, level),
        total: value.total(),
        explain: value.explain(),
        value,
    }
}

/// Basic Lift = ST²/5 lbs; rounded to a whole number if 10 or more (B15).
pub fn basic_lift(st: f64) -> f64 {
    let bl = st * st / 5.0;
    if bl >= 10.0 {
        bl.round()
    } else {
        // keep one decimal for small values
        (bl * 10.0).round() / 10.0
    }
}

pub fn compute_sheet(ch: &Character) -> Sheet {
    let Bonuses {
        by_stat: bonuses,
        lifting,
        striking,
    } = trait_bonuses(ch);

    let mut prim = BTreeMap::new();
    let mut attributes = Vec::new();
    for s in Stat::PRIMARY {
        let mut v = Value::new(10.0, "default");
        apply_common(&mut v, s, ch, &bonuses);
        prim.insert(s, v.total());
        attributes.push(row(s, ch, v));
    }
    let st = prim[&Stat::ST];
    let dx = prim[&Stat::DX];
    let iq = prim[&Stat::IQ];
    let ht = prim[&Stat::HT];

    let mut secondary = Vec::new();
    let mut speed_total: f64 = 0.0;
    for s in Stat::SECONDARY {
        let mut v = match s {
            Stat::HP => Value::new(st, format!("= ST {}", fmt_num(st))),
            Stat::Will => Value::new(iq, format!("= IQ {}", fmt_num(iq))),
            Stat::Per => Value::new(iq, format!("= IQ {}", fmt_num(iq))),
            Stat::FP => Value::new(ht, format!("= HT {}", fmt_num(ht))),
            Stat::BasicSpeed => Value::new(
                (dx + ht) / 4.0,
                format!("= (DX {} + HT {}) / 4", fmt_num(dx), fmt_num(ht)),
            ),
            Stat::BasicMove => Value::new(
                speed_total.floor(),
                format!("= floor(Basic Speed {})", fmt_num(speed_total)),
            ),
            _ => unreachable!(),
        };
        apply_common(&mut v, s, ch, &bonuses);
        if s == Stat::BasicSpeed {
            speed_total = v.total();
        }
        secondary.push(row(s, ch, v));
    }

    // Basic Lift: computed from ST (+ Lifting ST), shown as a difference from that base.
    let lift_st = st + lifting.iter().map(|(_, a)| a).sum::<f64>();
    let base_bl = basic_lift(st);
    let mut blv = Value::new(base_bl, format!("= ST {}² / 5", fmt_num(st)));
    if lift_st != st {
        let mut acc = st;
        let mut prev = base_bl;
        for (name, a) in &lifting {
            acc += a;
            let now = basic_lift(acc);
            blv.add(
                SourceKind::Trait(name.clone()),
                now - prev,
                format!("from trait {name} (ST {} for lifting)", fmt_num(acc)),
            );
            prev = now;
        }
    }
    let basic_lift_row = row(Stat::BasicLift, ch, blv);

    let mut dmg_st = Value::new(st, format!("= ST {}", fmt_num(st)));
    for (name, a) in &striking {
        dmg_st.add(
            SourceKind::Trait(name.clone()),
            *a,
            format!("from trait {name}"),
        );
    }
    let based_on_st = dmg_st.total().floor() as i32;
    let (thr, sw) = basic_damage(based_on_st);
    Sheet {
        name: ch.name.clone(),
        attributes,
        secondary,
        basic_lift: basic_lift_row,
        damage: DamageRow {
            thrust: thr.to_string(),
            swing: sw.to_string(),
            based_on_st,
            explain: dmg_st.explain(),
            st_value: dmg_st,
        },
        points: points::breakdown(ch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gurps_core::{Trait, TraitCategory, TraitDef};

    fn st_trait() -> Trait {
        Trait::new(
            TraitDef {
                id: "increased-st".into(),
                name: "Increased ST (example)".into(),
                category: TraitCategory::Advantage,
                cost: 0,
                cost_per_level: 10,
                max_levels: None,
                min_levels: None,
                page: None,
                features: vec![Feature::AttributeBonus {
                    target: Stat::ST,
                    amount: 1.0,
                    per_level: true,
                }],
                description: None,
            },
            1,
        )
    }

    #[test]
    fn defaults() {
        let s = compute_sheet(&Character::default());
        for st in Stat::PRIMARY {
            assert_eq!(s.total(st), 10.0);
        }
        assert_eq!(s.total(Stat::HP), 10.0);
        assert_eq!(s.total(Stat::Will), 10.0);
        assert_eq!(s.total(Stat::Per), 10.0);
        assert_eq!(s.total(Stat::FP), 10.0);
        assert_eq!(s.total(Stat::BasicSpeed), 5.0);
        assert_eq!(s.total(Stat::BasicMove), 5.0);
        assert_eq!(s.total(Stat::BasicLift), 20.0);
        assert_eq!(s.damage.thrust, "1d-2");
        assert_eq!(s.damage.swing, "1d");
        assert_eq!(s.points.total, 0);
    }

    #[test]
    fn secondary_formulas() {
        let mut ch = Character::default();
        ch.attributes.st = 3; // 13
        ch.attributes.dx = 2; // 12
        ch.attributes.iq = -1; // 9
        ch.attributes.ht = 1; // 11
        let s = compute_sheet(&ch);
        assert_eq!(s.total(Stat::HP), 13.0);
        assert_eq!(s.total(Stat::Will), 9.0);
        assert_eq!(s.total(Stat::Per), 9.0);
        assert_eq!(s.total(Stat::FP), 11.0);
        assert_eq!(s.total(Stat::BasicSpeed), 5.75);
        assert_eq!(s.total(Stat::BasicMove), 5.0);
        assert_eq!(s.total(Stat::BasicLift), 34.0); // 169/5 = 33.8 -> 34
        assert_eq!(s.points.attributes, 30 + 40 - 20 + 10);
    }

    #[test]
    fn purchased_secondary() {
        let mut ch = Character::default();
        ch.attributes.basic_speed_quarters = 4; // +1.0 speed = 20 pts
        ch.attributes.hp = 2; // 4 pts
        ch.attributes.fp = -1; // -3 pts
        ch.attributes.will = 1;
        ch.attributes.per = 1;
        ch.attributes.basic_move = -1;
        let s = compute_sheet(&ch);
        assert_eq!(s.total(Stat::BasicSpeed), 6.0);
        assert_eq!(s.total(Stat::BasicMove), 5.0); // floor(6)=6, -1
        assert_eq!(s.total(Stat::HP), 12.0);
        assert_eq!(s.total(Stat::FP), 9.0);
        assert_eq!(s.points.secondary, 20 + 4 - 3 + 5 + 5 - 5);
    }

    #[test]
    fn basic_lift_rounding() {
        assert_eq!(basic_lift(10.0), 20.0);
        assert_eq!(basic_lift(12.0), 29.0); // 28.8
        assert_eq!(basic_lift(7.0), 9.8); // below 10: not rounded to whole
        assert_eq!(basic_lift(3.0), 1.8);
        assert_eq!(basic_lift(15.0), 45.0);
    }

    /// Acceptance: ST 12 bought (+20 pts) and a trait giving +1 ST.
    #[test]
    fn acceptance_st_explanation() {
        let mut ch = Character::default();
        ch.attributes.st = 2;
        ch.traits.push(st_trait());
        let s = compute_sheet(&ch);
        let row = s.row(Stat::ST).unwrap();
        assert_eq!(row.total, 13.0);
        assert_eq!(row.points, 20);
        assert_eq!(row.value.base, 10.0);
        assert_eq!(row.value.contributions.len(), 2);
        assert_eq!(row.value.contributions[0].amount, 2.0);
        assert_eq!(row.value.contributions[0].source, SourceKind::Purchased);
        assert_eq!(row.value.contributions[1].amount, 1.0);
        assert_eq!(
            row.value.contributions[1].source,
            SourceKind::Trait("Increased ST (example) 1".into())
        );
        assert_eq!(
            row.explain,
            vec![
                "base 10 (default)",
                "+2 purchased (20 pts)",
                "+1 from trait Increased ST (example) 1",
                "= 13"
            ]
        );
        // propagation to derived values
        assert_eq!(s.total(Stat::HP), 13.0);
        assert_eq!(s.total(Stat::BasicLift), 34.0);
        assert_eq!(s.damage.swing, "2d-1");
        // points: 20 attributes + 10 trait
        assert_eq!(s.points.attributes, 20);
        assert_eq!(s.points.advantages, 10);
        assert_eq!(s.points.total, 30);
    }

    #[test]
    fn lifting_st_only_affects_basic_lift() {
        let mut ch = Character::default();
        ch.traits.push(Trait::new(
            TraitDef {
                id: "lifting-st".into(),
                name: "Lifting ST".into(),
                category: TraitCategory::Advantage,
                cost: 0,
                cost_per_level: 3,
                max_levels: None,
                min_levels: None,
                page: Some("B65".into()),
                features: vec![Feature::LiftingStBonus {
                    amount: 1.0,
                    per_level: true,
                }],
                description: None,
            },
            2,
        ));
        let s = compute_sheet(&ch);
        assert_eq!(s.total(Stat::ST), 10.0);
        assert_eq!(s.total(Stat::HP), 10.0);
        assert_eq!(s.total(Stat::BasicLift), 29.0); // ST 12 for lifting
        assert_eq!(s.basic_lift.value.base, 20.0);
        assert_eq!(s.basic_lift.value.contributions[0].amount, 9.0);
        assert_eq!(s.points.advantages, 6);
    }

    #[test]
    fn striking_st_only_affects_damage() {
        let mut ch = Character::default();
        let mut t = st_trait();
        t.def.features = vec![Feature::StrikingStBonus {
            amount: 1.0,
            per_level: true,
        }];
        t.levels = 3;
        ch.traits.push(t);
        let s = compute_sheet(&ch);
        assert_eq!(s.total(Stat::ST), 10.0);
        assert_eq!(s.total(Stat::HP), 10.0);
        assert_eq!(s.total(Stat::BasicLift), 20.0);
        assert_eq!(s.damage.based_on_st, 13);
        assert_eq!(
            (s.damage.thrust.as_str(), s.damage.swing.as_str()),
            ("1d", "2d-1")
        );
        assert_eq!(s.damage.st_value.contributions.len(), 1);
    }

    #[test]
    fn trait_bonus_to_secondary() {
        let mut ch = Character::default();
        let mut t = st_trait();
        t.def.features = vec![Feature::AttributeBonus {
            target: Stat::Per,
            amount: 2.0,
            per_level: false,
        }];
        t.levels = 3;
        ch.traits.push(t);
        let s = compute_sheet(&ch);
        assert_eq!(s.total(Stat::Per), 12.0);
        assert_eq!(s.total(Stat::Will), 10.0);
    }

    #[test]
    fn derived_explanation_mentions_formula() {
        let s = compute_sheet(&Character::default());
        let sp = s.row(Stat::BasicSpeed).unwrap();
        assert_eq!(sp.explain[0], "base 5 (= (DX 10 + HT 10) / 4)");
        let json = serde_json::to_value(&s).unwrap();
        assert!(json["attributes"][0]["value"]["base"].is_number());
    }
}
