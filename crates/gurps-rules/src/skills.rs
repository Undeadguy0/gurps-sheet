//! Skill and technique levels (B170-B174, B229-B230; pages unverified).

use gurps_core::{Character, Difficulty, Feature, SkillDefault, Stat};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Level steps bought with `points` above the 1-point level:
/// 1 pt -> 0, 2 -> 1, 4 -> 2, 8 -> 3, then +1 per 4 more points.
/// Points between steps buy nothing extra. `None` for 0 or fewer points.
pub fn level_steps(points: i32) -> Option<i32> {
    match points {
        p if p <= 0 => None,
        1 => Some(0),
        2 | 3 => Some(1),
        4..=7 => Some(2),
        p => Some(3 + (p - 8) / 4),
    }
}

/// Relative skill level (vs. the controlling attribute) for `points`.
pub fn relative_level(d: Difficulty, points: i32) -> Option<i32> {
    level_steps(points).map(|s| s + d.base_offset())
}

/// Points needed for a relative level; `None` below the 1-point level.
pub fn points_for_relative(d: Difficulty, relative: i32) -> Option<i32> {
    match relative - d.base_offset() {
        s if s < 0 => None,
        0 => Some(1),
        1 => Some(2),
        2 => Some(4),
        s => Some(8 + 4 * (s - 3)),
    }
}

/// True if `points` is exactly on the cost table (no wasted points).
pub fn is_standard_points(points: i32) -> bool {
    points == 0
        || level_steps(points)
            .is_some_and(|s| points_for_relative(Difficulty::Easy, s) == Some(points))
}

/// Levels a technique is raised above its default with `points`:
/// Average 1 pt per level, Hard 2 pts for the first level then 1 per level.
pub fn technique_levels(d: Difficulty, points: i32) -> i32 {
    match d {
        Difficulty::Hard | Difficulty::VeryHard => (points - 1).max(0),
        _ => points.max(0),
    }
}

/// Points for `levels` above the technique default.
pub fn technique_points(d: Difficulty, levels: i32) -> i32 {
    match d {
        _ if levels <= 0 => 0,
        Difficulty::Hard | Difficulty::VeryHard => levels + 1,
        _ => levels,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillRow {
    pub index: usize,
    pub id: String,
    /// Original name with specialization, e.g. "Survival (Woodlands)".
    pub name: String,
    pub attribute: String,
    pub difficulty: String,
    pub points: i32,
    /// Effective level; `None` when unknown and no default applies.
    pub level: Option<i32>,
    /// E.g. "DX+2", "Per-5".
    pub relative: Option<String>,
    /// True when the level comes from a default rather than points.
    pub from_default: bool,
    /// Talent bonus included in `level`.
    pub bonus: i32,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechniqueRow {
    pub index: usize,
    pub id: String,
    /// E.g. "Kicking (Brawling)".
    pub name: String,
    pub skill: String,
    pub difficulty: String,
    pub points: i32,
    pub level: Option<i32>,
    /// E.g. "Brawling-1".
    pub relative: Option<String>,
    /// Highest level allowed (parent skill + max).
    pub max_level: Option<i32>,
    pub explain: Vec<String>,
}

fn signed(v: i32) -> String {
    if v >= 0 {
        format!("+{v}")
    } else {
        v.to_string()
    }
}

/// Talent bonuses by skill id: (trait display name, amount).
fn talent_bonuses(ch: &Character) -> BTreeMap<&str, Vec<(String, i32)>> {
    let mut m: BTreeMap<&str, Vec<(String, i32)>> = BTreeMap::new();
    for t in &ch.traits {
        let lv = t.levels.max(1) as f64;
        for f in &t.def.features {
            if let Feature::SkillBonus {
                skills,
                amount,
                per_level,
            } = f
            {
                let a = if *per_level { amount * lv } else { *amount };
                for s in skills {
                    m.entry(s.as_str())
                        .or_default()
                        .push((t.display_name(), a.floor() as i32));
                }
            }
        }
    }
    m
}

/// Computes all skill rows. `attr` gives current attribute totals.
pub fn compute_skills(ch: &Character, attr: &dyn Fn(Stat) -> f64) -> Vec<SkillRow> {
    let talents = talent_bonuses(ch);
    let a = |s: Stat| attr(s).floor() as i32;
    // Pass 1: purchased levels (without talents) by skill id, best specialization.
    let mut bought: BTreeMap<&str, (i32, String)> = BTreeMap::new();
    for s in &ch.skills {
        if let Some(rel) = relative_level(s.def.difficulty, s.points) {
            let lv = a(s.def.attribute) + rel;
            let e = bought.entry(&s.def.id).or_insert((lv, s.display_name()));
            if lv > e.0 {
                *e = (lv, s.display_name());
            }
        }
    }
    ch.skills
        .iter()
        .enumerate()
        .map(|(index, s)| {
            let d = s.def.difficulty;
            let base = a(s.def.attribute);
            let akey = s.def.attribute.key();
            let mut explain = vec![format!("{akey} {base}")];
            let purchased = relative_level(d, s.points).map(|rel| {
                explain.push(format!(
                    "{}, {akey}/{} -> {akey}{} = {}",
                    crate::pts(s.points),
                    d.key(),
                    signed(rel),
                    base + rel
                ));
                base + rel
            });
            // Best default (other skills: purchased level only, so no cycles).
            let best_default = s
                .def
                .defaults
                .iter()
                .filter_map(|df| match df {
                    SkillDefault::Attribute {
                        attribute,
                        modifier,
                    } => Some((
                        a(*attribute) + modifier,
                        format!("{}{}", attribute.key(), signed(*modifier)),
                    )),
                    SkillDefault::Skill { skill, modifier } => {
                        bought.get(skill.as_str()).and_then(|(lv, name)| {
                            (skill != &s.def.id)
                                .then(|| (lv + modifier, format!("{name}{}", signed(*modifier))))
                        })
                    }
                })
                .max_by_key(|(lv, _)| *lv);
            if let Some((lv, why)) = &best_default {
                explain.push(format!("default {why} = {lv}"));
            }
            let (raw, from_default) = match (purchased, &best_default) {
                (Some(p), Some((dl, _))) if *dl > p => (Some(*dl), true),
                (Some(p), _) => (Some(p), false),
                (None, Some((dl, _))) => (Some(*dl), true),
                (None, None) => (None, false),
            };
            let mut bonus = 0;
            if raw.is_some() {
                for (name, b) in talents.get(s.def.id.as_str()).into_iter().flatten() {
                    bonus += b;
                    explain.push(format!("{} from trait {name}", signed(*b)));
                }
            }
            let level = raw.map(|l| l + bonus);
            match level {
                Some(l) => explain.push(format!("= {l}")),
                None => explain.push("no points and no default".into()),
            }
            SkillRow {
                index,
                id: s.def.id.clone(),
                name: s.display_name(),
                attribute: akey.into(),
                difficulty: d.key().into(),
                points: s.points,
                level,
                relative: level.map(|l| format!("{akey}{}", signed(l - base))),
                from_default,
                bonus,
                explain,
            }
        })
        .collect()
}

/// Computes technique rows from the already computed skill rows.
pub fn compute_techniques(ch: &Character, skills: &[SkillRow]) -> Vec<TechniqueRow> {
    ch.techniques
        .iter()
        .enumerate()
        .map(|(index, t)| {
            let d = t.def.difficulty;
            let parent = skills
                .iter()
                .filter(|r| r.id == t.def.skill && r.level.is_some())
                .max_by_key(|r| r.level);
            let parent_name = parent.map(|p| p.name.clone()).unwrap_or_else(|| {
                ch.skills
                    .iter()
                    .find(|s| s.def.id == t.def.skill)
                    .map(|s| s.def.name.clone())
                    .unwrap_or_else(|| t.def.skill.clone())
            });
            let mut explain = Vec::new();
            let (level, max_level) = match parent.and_then(|p| p.level.map(|l| (l, p))) {
                Some((pl, p)) => {
                    let def = pl + t.def.modifier;
                    let cap = pl + t.def.max;
                    let up = technique_levels(d, t.points);
                    explain.push(format!(
                        "default {}{} = {def}",
                        p.name,
                        signed(t.def.modifier)
                    ));
                    if up > 0 {
                        explain.push(format!("{} ({}) -> +{up}", crate::pts(t.points), d.key()));
                    }
                    let lv = (def + up).min(cap);
                    if def + up > cap {
                        explain.push(format!("capped at {}{} = {cap}", p.name, signed(t.def.max)));
                    }
                    explain.push(format!("= {lv}"));
                    (Some((lv, pl)), Some(cap))
                }
                None => {
                    explain.push(format!("needs skill {parent_name}"));
                    (None, None)
                }
            };
            TechniqueRow {
                index,
                id: t.def.id.clone(),
                name: format!("{} ({parent_name})", t.def.name),
                skill: t.def.skill.clone(),
                difficulty: d.key().into(),
                points: t.points,
                level: level.map(|(l, _)| l),
                relative: level.map(|(l, pl)| format!("{parent_name}{}", signed(l - pl))),
                max_level,
                explain,
            }
        })
        .collect()
}

/// Total points spent on skills and techniques.
pub fn skill_points(ch: &Character) -> i32 {
    ch.skills.iter().map(|s| s.points.max(0)).sum::<i32>()
        + ch.techniques.iter().map(|t| t.points.max(0)).sum::<i32>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute_sheet;
    use gurps_core::{Skill, SkillDef, Technique, TechniqueDef, Trait, TraitDef};
    use Difficulty::*;

    #[test]
    fn cost_table() {
        // B170 progression (unverified page): 1, 2, 4, 8, then +4 per level.
        let pts = [1, 2, 4, 8, 12, 16, 20];
        for (d, first) in [(Easy, 0), (Average, -1), (Hard, -2), (VeryHard, -3)] {
            for (i, p) in pts.iter().enumerate() {
                assert_eq!(relative_level(d, *p), Some(first + i as i32), "{d:?} {p}");
                assert_eq!(points_for_relative(d, first + i as i32), Some(*p));
            }
            assert_eq!(relative_level(d, 0), None);
            assert_eq!(points_for_relative(d, first - 1), None);
        }
        assert_eq!(
            relative_level(Average, 3),
            Some(0),
            "3 pts buy the 2-pt level"
        );
        assert_eq!(relative_level(Hard, 7), Some(0));
        assert!(is_standard_points(12) && !is_standard_points(6) && is_standard_points(0));
    }

    #[test]
    fn technique_costs() {
        assert_eq!(technique_levels(Average, 3), 3);
        assert_eq!(technique_levels(Hard, 1), 0);
        assert_eq!(technique_levels(Hard, 2), 1);
        assert_eq!(technique_levels(Hard, 4), 3);
        assert_eq!(technique_points(Hard, 1), 2);
        assert_eq!(technique_points(Average, 2), 2);
        assert_eq!(technique_points(Hard, 0), 0);
    }

    pub(crate) fn sk(id: &str, attr: Stat, d: Difficulty, defaults: Vec<SkillDefault>) -> SkillDef {
        SkillDef {
            id: id.into(),
            name: id.into(),
            attribute: attr,
            difficulty: d,
            defaults,
            specialization_required: false,
            specializations: vec![],
            page: None,
            description: None,
        }
    }

    fn attr_def(a: Stat, m: i32) -> SkillDefault {
        SkillDefault::Attribute {
            attribute: a,
            modifier: m,
        }
    }

    #[test]
    fn defaults_pick_best() {
        let mut ch = Character::default();
        ch.attributes.dx = 2; // DX 12
        ch.skills.push(Skill::new(
            sk(
                "broadsword",
                Stat::DX,
                Average,
                vec![attr_def(Stat::DX, -5)],
            ),
            8,
        ));
        ch.skills.push(Skill::new(
            sk(
                "shortsword",
                Stat::DX,
                Average,
                vec![
                    attr_def(Stat::DX, -5),
                    SkillDefault::Skill {
                        skill: "broadsword".into(),
                        modifier: -2,
                    },
                ],
            ),
            0,
        ));
        ch.skills.push(Skill::new(
            sk("stealth", Stat::DX, Average, vec![attr_def(Stat::DX, -5)]),
            0,
        ));
        ch.skills
            .push(Skill::new(sk("arcane", Stat::IQ, Hard, vec![]), 0));
        // 1 point in a skill whose default is better: the default wins.
        ch.skills.push(Skill::new(
            sk(
                "tumble",
                Stat::DX,
                Hard,
                vec![SkillDefault::Skill {
                    skill: "broadsword".into(),
                    modifier: -1,
                }],
            ),
            1,
        ));
        let s = compute_sheet(&ch);
        let lv: Vec<_> = s.skills.iter().map(|r| r.level).collect();
        assert_eq!(lv, vec![Some(14), Some(12), Some(7), None, Some(13)]);
        assert_eq!(s.skills[0].relative.as_deref(), Some("DX+2"));
        assert!(s.skills[1].from_default && !s.skills[0].from_default);
        assert_eq!(s.skills[1].relative.as_deref(), Some("DX+0"));
        assert!(s.skills[4].from_default);
        assert_eq!(s.skills[0].explain.last().unwrap(), "= 14");
        assert_eq!(s.points.skills, 9);
    }

    #[test]
    fn talent_and_technique_cap() {
        let mut ch = Character::default();
        ch.attributes.dx = 2;
        let talent = TraitDef {
            id: "talent".into(),
            name: "Outdoorsman".into(),
            category: Default::default(),
            cost: 0,
            cost_per_level: 10,
            max_levels: Some(4),
            min_levels: None,
            page: None,
            features: vec![Feature::SkillBonus {
                skills: vec!["tracking".into()],
                amount: 1.0,
                per_level: true,
            }],
            description: None,
        };
        ch.traits.push(Trait::new(talent, 2));
        ch.skills.push(Skill::new(
            sk(
                "tracking",
                Stat::Per,
                Average,
                vec![attr_def(Stat::Per, -5)],
            ),
            2,
        ));
        ch.skills
            .push(Skill::new(sk("brawling", Stat::DX, Easy, vec![]), 1));
        let kick = TechniqueDef {
            id: "kicking".into(),
            name: "Kicking".into(),
            skill: "brawling".into(),
            difficulty: Hard,
            modifier: -2,
            max: 0,
            page: None,
            description: None,
        };
        ch.techniques.push(Technique::new(kick.clone(), 2));
        ch.techniques.push(Technique::new(kick, 5));
        let s = compute_sheet(&ch);
        assert_eq!(s.skills[0].level, Some(12), "Per 10 +0 +2 talent");
        assert_eq!(s.skills[0].bonus, 2);
        assert_eq!(s.techniques[0].level, Some(11));
        assert_eq!(s.techniques[0].relative.as_deref(), Some("brawling-1"));
        assert_eq!(s.techniques[1].level, Some(12), "capped at Brawling");
        assert_eq!(s.techniques[1].max_level, Some(12));
        assert!(s.techniques[1]
            .explain
            .iter()
            .any(|l| l.starts_with("capped")));
        assert_eq!(s.points.skills, 3 + 7);
        ch.skills.remove(1);
        let s = compute_sheet(&ch);
        assert_eq!(s.techniques[0].level, None);
        assert!(!crate::validate::validate(&ch).is_empty());
    }
}
