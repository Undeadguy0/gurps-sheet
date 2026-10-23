//! Spell levels: IQ-based Hard/Very Hard skills plus Magery (B235-B236; pages unverified).

use crate::skills::relative_level;
use gurps_core::{Character, Feature, Stat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpellRow {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub college: Vec<String>,
    pub attribute: String,
    pub difficulty: String,
    pub points: i32,
    /// `None` with 0 points (spells have no default).
    pub level: Option<i32>,
    /// E.g. "IQ+1" (Magery included).
    pub relative: Option<String>,
    /// Magery bonus included in `level`.
    pub bonus: i32,
    pub prerequisites: Option<String>,
    pub explain: Vec<String>,
}

fn signed(v: i32) -> String {
    if v >= 0 {
        format!("+{v}")
    } else {
        v.to_string()
    }
}

/// Spell bonuses from traits: (trait display name, amount, colleges).
fn spell_bonuses(ch: &Character) -> Vec<(String, i32, &[String])> {
    let mut out = Vec::new();
    for t in &ch.traits {
        for f in &t.def.features {
            if let Feature::SpellBonus {
                amount,
                colleges,
                per_level,
            } = f
            {
                // per level uses the real level: Magery 0 gives no bonus
                let a = if *per_level {
                    amount * t.levels as f64
                } else {
                    *amount
                };
                out.push((t.display_name(), a.floor() as i32, colleges.as_slice()));
            }
        }
    }
    out
}

pub fn compute_spells(ch: &Character, attr: &dyn Fn(Stat) -> f64) -> Vec<SpellRow> {
    let bonuses = spell_bonuses(ch);
    ch.spells
        .iter()
        .enumerate()
        .map(|(index, s)| {
            let d = s.def.difficulty;
            let akey = s.def.attribute.key();
            let base = attr(s.def.attribute).floor() as i32;
            let mut explain = vec![format!("{akey} {base}")];
            let mut bonus = 0;
            let level = match relative_level(d, s.points) {
                Some(rel) => {
                    explain.push(format!(
                        "{}, {akey}/{} -> {akey}{} = {}",
                        crate::pts(s.points),
                        d.key(),
                        signed(rel),
                        base + rel
                    ));
                    for (name, b, colleges) in &bonuses {
                        let applies = colleges.is_empty()
                            || colleges.iter().any(|c| s.def.college.contains(c));
                        if applies && *b != 0 {
                            bonus += b;
                            explain.push(format!("{} from trait {name}", signed(*b)));
                        }
                    }
                    let l = base + rel + bonus;
                    explain.push(format!("= {l}"));
                    Some(l)
                }
                None => {
                    explain.push("spells have no default: buy at least 1 pt".into());
                    None
                }
            };
            SpellRow {
                index,
                id: s.def.id.clone(),
                name: s.def.name.clone(),
                college: s.def.college.clone(),
                attribute: akey.into(),
                difficulty: d.key().into(),
                points: s.points,
                level,
                relative: level.map(|l| format!("{akey}{}", signed(l - base))),
                bonus,
                prerequisites: s.def.prerequisites.clone(),
                explain,
            }
        })
        .collect()
}

pub fn spell_points(ch: &Character) -> i32 {
    ch.spells.iter().map(|s| s.points.max(0)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gurps_core::{Difficulty, Spell, SpellDef};

    fn spell(d: Difficulty, college: &str, pts: i32) -> Spell {
        Spell::new(
            SpellDef {
                id: "x".into(),
                name: "X".into(),
                attribute: Stat::IQ,
                difficulty: d,
                college: vec![college.into()],
                prerequisites: None,
                page: None,
                description: None,
            },
            pts,
        )
    }

    #[test]
    fn hard_and_very_hard_with_magery() {
        let mut ch = Character::default();
        ch.spells.push(spell(Difficulty::Hard, "Fire", 1));
        ch.spells.push(spell(Difficulty::VeryHard, "Fire", 4));
        ch.spells.push(spell(Difficulty::Hard, "Fire", 0));
        let iq = |_| 14.0;
        let r = compute_spells(&ch, &iq);
        assert_eq!(r[0].level, Some(12));
        assert_eq!(r[1].level, Some(13));
        assert_eq!(r[2].level, None);
        let mut magery = gurps_core::Trait::new(
            gurps_core::TraitDef {
                id: "magery".into(),
                name: "Magery".into(),
                category: Default::default(),
                cost: 5,
                cost_per_level: 10,
                max_levels: None,
                min_levels: Some(0),
                page: None,
                features: vec![Feature::SpellBonus {
                    amount: 1.0,
                    colleges: vec![],
                    per_level: true,
                }],
                description: None,
            },
            0,
        );
        ch.traits.push(magery.clone());
        assert_eq!(
            compute_spells(&ch, &iq)[0].level,
            Some(12),
            "Magery 0: no bonus"
        );
        magery.levels = 2;
        ch.traits[0] = magery;
        let r = compute_spells(&ch, &iq);
        assert_eq!(r[0].level, Some(14));
        assert_eq!(r[0].relative.as_deref(), Some("IQ+0"));
        assert_eq!(r[0].explain[2], "+2 from trait Magery 2");
        assert_eq!(spell_points(&ch), 5);
    }
}
