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
