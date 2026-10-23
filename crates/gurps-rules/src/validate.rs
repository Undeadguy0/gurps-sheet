use crate::points;
use gurps_core::Character;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
}

/// Checks a character for rule violations.
pub fn validate(ch: &Character) -> Vec<Issue> {
    let mut out = Vec::new();
    let b = points::breakdown(ch);
    if b.unspent < 0 {
        out.push(Issue {
            severity: Severity::Warning,
            message: format!("over budget by {}", crate::pts(-b.unspent)),
        });
    }
    // Disadvantage limit (quirks are counted separately); warn only.
    let limit = ch.disadvantage_limit();
    if -b.disadvantages > limit {
        out.push(Issue {
            severity: Severity::Warning,
            message: format!(
                "disadvantages total {} which is over the limit of {}",
                crate::pts(-b.disadvantages),
                crate::pts(limit)
            ),
        });
    }
    let quirks = ch
        .traits
        .iter()
        .filter(|t| t.def.category == gurps_core::TraitCategory::Quirk)
        .count();
    if quirks > 5 || b.quirks < -5 {
        out.push(Issue {
            severity: Severity::Warning,
            message: format!(
                "{quirks} quirks for {}: the usual maximum is 5 quirks at -1",
                crate::pts(b.quirks)
            ),
        });
    }
    let carried: f64 = ch
        .equipment
        .iter()
        .map(gurps_core::Equipment::carried_weight)
        .sum();
    if carried > 0.0 {
        let sheet = crate::compute_sheet(ch);
        if sheet.encumbrance.overloaded {
            out.push(Issue {
                severity: Severity::Warning,
                message: format!(
                    "carried {} lb is more than 10×BL ({} lb): the character cannot move",
                    crate::value::fmt_num(carried),
                    crate::value::fmt_num(sheet.basic_lift.total * 10.0)
                ),
            });
        }
    }
    for t in &ch.traits {
        if t.def.is_leveled() && t.levels < t.def.min_level() {
            out.push(Issue {
                severity: Severity::Error,
                message: format!(
                    "trait '{}' needs at least {} level(s)",
                    t.def.name,
                    t.def.min_level()
                ),
            });
        }
        if let Some(max) = t.def.max_levels {
            if t.levels > max {
                out.push(Issue {
                    severity: Severity::Error,
                    message: format!("trait '{}' exceeds max level {max}", t.def.name),
                });
            }
        }
    }
    let warn = |out: &mut Vec<Issue>, message: String| {
        out.push(Issue {
            severity: Severity::Warning,
            message,
        })
    };
    for s in &ch.skills {
        let name = s.display_name();
        if s.points < 0 {
            out.push(Issue {
                severity: Severity::Error,
                message: format!("skill '{name}' has negative points"),
            });
        } else if !crate::skills::is_standard_points(s.points) {
            warn(
                &mut out,
                format!(
                    "skill '{name}': {} is not on the cost table (1, 2, 4, 8, 12...); extra points are wasted",
                    crate::pts(s.points)
                ),
            );
        }
        if s.def.specialization_required && gurps_core::is_blank(&s.specialization) {
            warn(&mut out, format!("skill '{name}' needs a specialization"));
        }
    }
    for t in &ch.techniques {
        if t.points < 0 {
            out.push(Issue {
                severity: Severity::Error,
                message: format!("technique '{}' has negative points", t.def.name),
            });
        }
        if !ch.skills.iter().any(|s| s.def.id == t.def.skill) {
            warn(
                &mut out,
                format!("technique '{}' needs skill '{}'", t.def.name, t.def.skill),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn over_budget_warns() {
        let mut ch = Character::default();
        ch.points_budget = 10;
        ch.attributes.dx = 1;
        let v = validate(&ch);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].severity, Severity::Warning);
        assert!(validate(&Character::default()).is_empty());
    }

    fn tr(cat: gurps_core::TraitCategory, cost: i32) -> gurps_core::Trait {
        gurps_core::Trait::new(
            gurps_core::TraitDef {
                id: format!("t{cost}"),
                name: "T".into(),
                category: cat,
                cost,
                cost_per_level: 0,
                max_levels: None,
                min_levels: None,
                page: None,
                features: vec![],
                description: None,
            },
            0,
        )
    }

    #[test]
    fn disadvantage_and_quirk_limits_warn() {
        use gurps_core::TraitCategory::*;
        let mut ch = Character::default(); // 150 starting -> limit 75
        ch.traits.push(tr(Disadvantage, -75));
        assert!(validate(&ch).iter().all(|i| !i.message.contains("limit")));
        ch.traits.push(tr(Disadvantage, -5));
        let v = validate(&ch);
        assert!(
            v.iter().any(
                |i| i.message == "disadvantages total 80 pts which is over the limit of 75 pts"
            ),
            "{v:?}"
        );
        ch.rules.disadvantage_limit = Some(100);
        assert!(validate(&ch).iter().all(|i| !i.message.contains("limit")));
        for _ in 0..6 {
            ch.traits.push(tr(Quirk, -1));
        }
        assert!(validate(&ch)
            .iter()
            .any(|i| i.message.starts_with("6 quirks")));
    }

    #[test]
    fn min_levels() {
        use gurps_core::{Trait, TraitDef};
        let def = TraitDef {
            id: "m".into(),
            name: "Magery".into(),
            category: Default::default(),
            cost: 5,
            cost_per_level: 10,
            max_levels: None,
            min_levels: Some(0),
            page: None,
            features: vec![],
            description: None,
        };
        let mut ch = Character::default();
        ch.traits.push(Trait::new(def.clone(), 0));
        assert!(validate(&ch).is_empty(), "Magery 0 is valid");
        assert_eq!(ch.traits[0].cost(), 5);
        let mut leveled = def;
        leveled.min_levels = None;
        ch.traits = vec![Trait::new(leveled, 0)];
        assert_eq!(validate(&ch)[0].severity, Severity::Error);
    }
}
