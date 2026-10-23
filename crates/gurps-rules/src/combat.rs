//! Weapons (attack levels, damage) and active defences (B324-B327, B374, # unverified).
//!
//! * Weapon damage: basic thrust/swing for striking ST (ST + Striking ST) plus the
//!   expression's adds, e.g. sw 2d-1 with `sw+1` = 2d; fixed dice are used as is.
//! * Skill damage bonus (Brawling at DX+2+: +1 per die) is data-driven per attack.
//! * Parry = floor(level / 2) + 3 + weapon parry modifier; Block = floor(Shield / 2) + 3;
//!   Dodge = floor(Basic Speed) + 3 − encumbrance; trait `defense_bonus` adds to all.
//! * Shield DB (from equipment stats "DB n", carried) is reported separately: it applies
//!   only against attacks from the front.

use crate::encumbrance::Encumbrance;
use crate::skills::{SkillRow, TechniqueRow};
use gurps_core::damage::DiceSpec;
use gurps_core::weapon::{parse_db, parse_dr};
use gurps_core::{Character, DamageExpr, Equipment, Feature, SkillDefault, Stat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackRow {
    pub mode: String,
    /// Skill or technique name used (original names).
    pub skill: String,
    pub level: Option<i32>,
    /// Data expression, e.g. "sw+1".
    pub damage_expr: String,
    /// Resolved dice, e.g. "2d".
    pub damage: String,
    pub damage_type: String,
    pub reach: String,
    /// Final Parry; `None` = no parry (or no skill).
    pub parry: Option<i32>,
    /// False when this attack cannot parry at all.
    pub can_parry: bool,
    pub parry_modifier: i32,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponRow {
    pub index: usize,
    pub id: String,
    pub name: String,
    /// Linked equipment id, if any.
    pub equipment: Option<String>,
    /// The linked equipment is owned (or the weapon is unarmed).
    pub available: bool,
    /// Owned and carried (counts for best Parry).
    pub ready: bool,
    pub attacks: Vec<AttackRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Defense {
    pub value: i32,
    /// What it comes from (weapon or skill name).
    pub from: String,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmorRow {
    pub item: String,
    /// As written, e.g. "4/2*".
    pub dr: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Defenses {
    pub dodge: Defense,
    /// Best Parry among ready weapons.
    pub parry: Option<Defense>,
    pub block: Option<Defense>,
    /// Shield DB (frontal attacks only), 0 without a carried shield.
    pub db: i32,
    pub db_from: Option<String>,
    /// Trait bonus to defences (Combat Reflexes).
    pub bonus: i32,
    pub armor: Vec<ArmorRow>,
}

/// (dodge, parry, block) bonus from traits, with trait names.
fn defense_bonuses(ch: &Character) -> ([i32; 3], Vec<String>) {
    let mut b = [0; 3];
    let mut names = Vec::new();
    for t in &ch.traits {
        for f in &t.def.features {
            if let Feature::DefenseBonus {
                amount,
                defenses,
                per_level,
            } = f
            {
                let a = if *per_level {
                    amount * t.levels.max(1) as f64
                } else {
                    *amount
                } as i32;
                for (i, k) in ["dodge", "parry", "block"].iter().enumerate() {
                    if defenses.is_empty() || defenses.iter().any(|d| d.eq_ignore_ascii_case(k)) {
                        b[i] += a;
                    }
                }
                names.push(t.display_name());
            }
        }
    }
    (b, names)
}

fn sg(v: i32) -> String {
    if v >= 0 {
        format!("+{v}")
    } else {
        v.to_string()
    }
}

fn walk<'a>(list: &'a [Equipment], carried: bool, f: &mut dyn FnMut(&'a Equipment, bool)) {
    for e in list {
        let c = carried && e.carried;
        f(e, c);
        walk(&e.contents, c, f);
    }
}

pub struct Ctx<'a> {
    pub skills: &'a [SkillRow],
    pub techniques: &'a [TechniqueRow],
    pub attr: &'a dyn Fn(Stat) -> f64,
    pub thrust: DiceSpec,
    pub swing: DiceSpec,
}

fn best_skill<'a>(skills: &'a [SkillRow], id: &str) -> Option<&'a SkillRow> {
    skills
        .iter()
        .filter(|r| r.id == id && r.level.is_some())
        .max_by_key(|r| r.level)
}

pub fn compute_weapons(ch: &Character, c: &Ctx) -> Vec<WeaponRow> {
    let (bonus, _) = defense_bonuses(ch);
    let mut owned: Vec<(&str, bool)> = Vec::new();
    walk(&ch.equipment, true, &mut |e, carried| {
        owned.push((e.def.id.as_str(), carried))
    });
    let a = |s: Stat| (c.attr)(s).floor() as i32;
    ch.weapons
        .iter()
        .enumerate()
        .map(|(index, w)| {
            let (available, ready) = match &w.def.equipment {
                None => (true, true),
                Some(id) => {
                    let mine: Vec<bool> =
                        owned.iter().filter(|(o, _)| o == id).map(|x| x.1).collect();
                    (!mine.is_empty(), mine.iter().any(|x| *x))
                }
            };
            let attacks = w
                .def
                .attacks
                .iter()
                .map(|at| {
                    let skill_id = at.skill.as_deref().unwrap_or(&w.def.skill);
                    let skill = best_skill(c.skills, skill_id);
                    let tech = at.technique.as_deref().and_then(|t| {
                        c.techniques
                            .iter()
                            .filter(|r| r.id == t && r.level.is_some())
                            .max_by_key(|r| r.level)
                    });
                    let mut explain = Vec::new();
                    let (level, used) = if let Some(t) = tech {
                        explain.push(format!("technique {} {}", t.name, t.level.unwrap()));
                        (t.level, t.name.clone())
                    } else if let Some(s) = skill {
                        let l = s.level.unwrap() + at.skill_modifier;
                        if at.skill_modifier != 0 {
                            explain.push(format!(
                                "skill {}{} = {l}",
                                s.name,
                                sg(at.skill_modifier)
                            ));
                        } else {
                            explain.push(format!("skill {} {l}", s.name));
                        }
                        (Some(l), s.name.clone())
                    } else {
                        let best = at
                            .defaults
                            .iter()
                            .filter_map(|d| match d {
                                SkillDefault::Attribute {
                                    attribute,
                                    modifier,
                                } => Some((
                                    a(*attribute) + modifier,
                                    format!("{}{}", attribute.key(), sg(*modifier)),
                                )),
                                SkillDefault::Skill { skill, modifier } => {
                                    best_skill(c.skills, skill).map(|r| {
                                        (
                                            r.level.unwrap() + modifier,
                                            format!("{}{}", r.name, sg(*modifier)),
                                        )
                                    })
                                }
                            })
                            .max_by_key(|x| x.0);
                        match best {
                            Some((l, n)) => {
                                explain.push(format!("default {n} = {l}"));
                                (Some(l), n)
                            }
                            None => {
                                explain.push(format!("no skill {skill_id} and no default"));
                                (None, skill_id.to_string())
                            }
                        }
                    };
                    let (mut dice, base) = match DamageExpr::parse(&at.damage) {
                        Some(DamageExpr::Thrust(m)) => (
                            DiceSpec {
                                dice: c.thrust.dice,
                                adds: c.thrust.adds + m,
                            },
                            format!("thr {}", c.thrust),
                        ),
                        Some(DamageExpr::Swing(m)) => (
                            DiceSpec {
                                dice: c.swing.dice,
                                adds: c.swing.adds + m,
                            },
                            format!("sw {}", c.swing),
                        ),
                        Some(DamageExpr::Fixed(d)) => (d, "fixed".into()),
                        None => (DiceSpec { dice: 0, adds: 0 }, "unparsed".into()),
                    };
                    explain.push(format!("damage {} = {base} -> {dice}", at.damage));
                    if let (Some(b), Some(s)) = (at.skill_damage_bonus, skill) {
                        let attr = Stat::parse(&s.attribute).map(a).unwrap_or(10);
                        let rel = s.level.unwrap() - attr;
                        if rel >= b.min_relative {
                            dice.adds += b.per_die * dice.dice as i32;
                            explain.push(format!(
                                "{} at {}{} (>= {}): {} per die -> {dice}",
                                s.name,
                                s.attribute,
                                sg(rel),
                                sg(b.min_relative),
                                sg(b.per_die)
                            ));
                        } else {
                            explain.push(format!(
                                "no {} damage bonus: {}{} < {}{}",
                                s.name,
                                s.attribute,
                                sg(rel),
                                s.attribute,
                                sg(b.min_relative)
                            ));
                        }
                    }
                    let can_parry = at.parry.is_some();
                    let pm = at.parry.unwrap_or(0);
                    let parry = match (at.parry, level) {
                        (Some(m), Some(l)) => {
                            let p = l.div_euclid(2) + 3 + m + bonus[1];
                            explain.push(format!(
                                "Parry floor({l}/2) + 3{}{} = {p}",
                                if m != 0 {
                                    format!(" {}", sg(m))
                                } else {
                                    String::new()
                                },
                                if bonus[1] != 0 {
                                    format!(" {}", sg(bonus[1]))
                                } else {
                                    String::new()
                                }
                            ));
                            Some(p)
                        }
                        (None, _) => {
                            explain.push("cannot parry".into());
                            None
                        }
                        _ => None,
                    };
                    AttackRow {
                        mode: at.mode.clone(),
                        skill: used,
                        level,
                        damage_expr: at.damage.clone(),
                        damage: dice.to_string(),
                        damage_type: at.damage_type.clone(),
                        reach: at.reach.clone(),
                        parry,
                        can_parry,
                        parry_modifier: pm,
                        explain,
                    }
                })
                .collect();
            WeaponRow {
                index,
                id: w.def.id.clone(),
                name: w.def.name.clone(),
                equipment: w.def.equipment.clone(),
                available,
                ready,
                attacks,
            }
        })
        .collect()
}

pub fn compute_defenses(
    ch: &Character,
    weapons: &[WeaponRow],
    skills: &[SkillRow],
    enc: &Encumbrance,
) -> Defenses {
    let (bonus, names) = defense_bonuses(ch);
    let from_traits = if names.is_empty() {
        String::new()
    } else {
        format!(" ({})", names.join(", "))
    };
    let bonus_txt = |b: i32| {
        if b != 0 {
            format!(" {}{from_traits}", sg(b))
        } else {
            String::new()
        }
    };
    let dodge_v = enc.base_dodge + bonus[0] + enc.dodge_penalty;
    let mut dodge_explain = vec![format!(
        "floor(Basic Speed) + 3 = {}{}",
        enc.base_dodge,
        bonus_txt(bonus[0])
    )];
    if enc.dodge_penalty != 0 {
        dodge_explain.push(format!("encumbrance {} ({})", enc.dodge_penalty, enc.name));
    }
    dodge_explain.push(format!("= {dodge_v}"));

    let parry = weapons
        .iter()
        .filter(|w| w.ready)
        .flat_map(|w| w.attacks.iter().map(move |a| (w, a)))
        .filter_map(|(w, a)| a.parry.map(|p| (p, w, a)))
        .max_by_key(|x| x.0)
        .map(|(p, w, a)| Defense {
            value: p,
            from: w.name.clone(),
            explain: a
                .explain
                .iter()
                .filter(|l| {
                    l.starts_with("Parry")
                        || l.starts_with("skill")
                        || l.starts_with("default")
                        || l.starts_with("technique")
                })
                .cloned()
                .collect(),
        });

    let mut db = 0;
    let mut db_from = None;
    let mut armor = Vec::new();
    walk(&ch.equipment, true, &mut |e, carried| {
        if !carried {
            return;
        }
        if let Some(st) = e.def.stats.as_deref() {
            if let Some(d) = parse_db(st) {
                if d > db {
                    db = d;
                    db_from = Some(e.def.name.clone());
                }
            }
            if let Some((dr, value)) = parse_dr(st) {
                armor.push(ArmorRow {
                    item: e.def.name.clone(),
                    dr,
                    value,
                });
            }
        }
    });
    let block = if db > 0 {
        best_skill(skills, "shield").map(|s| {
            let l = s.level.unwrap();
            let v = l.div_euclid(2) + 3 + bonus[2];
            Defense {
                value: v,
                from: s.name.clone(),
                explain: vec![
                    format!("skill {} {l}", s.name),
                    format!("Block floor({l}/2) + 3{} = {v}", bonus_txt(bonus[2])),
                ],
            }
        })
    } else {
        None
    };
    Defenses {
        dodge: Defense {
            value: dodge_v,
            from: "Basic Speed".into(),
            explain: dodge_explain,
        },
        parry,
        block,
        db,
        db_from,
        bonus: bonus[0].max(bonus[1]).max(bonus[2]),
        armor,
    }
}
