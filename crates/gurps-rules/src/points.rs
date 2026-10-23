use gurps_core::{Character, Stat, TraitCategory};
use serde::{Deserialize, Serialize};

/// Cost in character points of one purchased step of a stat (B14–B17).
pub fn cost_per_step(stat: Stat) -> i32 {
    match stat {
        Stat::ST | Stat::HT => 10,
        Stat::DX | Stat::IQ => 20,
        Stat::HP => 2,
        Stat::Will | Stat::Per => 5,
        Stat::FP => 3,
        Stat::BasicSpeed => 5, // per 0.25
        Stat::BasicMove => 5,
        Stat::BasicLift => 0,
    }
}

pub fn stat_points(stat: Stat, purchased: i32) -> i32 {
    cost_per_step(stat) * purchased
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointsBreakdown {
    pub attributes: i32,
    pub secondary: i32,
    pub advantages: i32,
    pub perks: i32,
    pub disadvantages: i32,
    pub quirks: i32,
    pub skills: i32,
    /// Points in spells.
    #[serde(default)]
    pub spells: i32,
    pub total: i32,
    pub budget: i32,
    pub unspent: i32,
}

pub fn breakdown(ch: &Character) -> PointsBreakdown {
    let mut b = PointsBreakdown {
        budget: ch.points_budget,
        ..Default::default()
    };
    for s in Stat::PRIMARY {
        b.attributes += stat_points(s, ch.attributes.purchased(s));
    }
    for s in Stat::SECONDARY {
        b.secondary += stat_points(s, ch.attributes.purchased(s));
    }
    for t in &ch.traits {
        let slot = match t.def.category {
            TraitCategory::Advantage => &mut b.advantages,
            TraitCategory::Perk => &mut b.perks,
            TraitCategory::Disadvantage => &mut b.disadvantages,
            TraitCategory::Quirk => &mut b.quirks,
        };
        *slot += t.cost_with(ch.rules.negative_rounding);
    }
    b.skills = crate::skills::skill_points(ch);
    b.spells = crate::spells::spell_points(ch);
    b.total = b.attributes
        + b.secondary
        + b.advantages
        + b.perks
        + b.disadvantages
        + b.quirks
        + b.skills
        + b.spells;
    b.unspent = b.budget - b.total;
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use gurps_core::{Trait, TraitDef};

    #[test]
    fn per_step_costs() {
        let expect = [
            (Stat::ST, 10),
            (Stat::DX, 20),
            (Stat::IQ, 20),
            (Stat::HT, 10),
            (Stat::HP, 2),
            (Stat::Will, 5),
            (Stat::Per, 5),
            (Stat::FP, 3),
            (Stat::BasicSpeed, 5),
            (Stat::BasicMove, 5),
        ];
        for (s, c) in expect {
            assert_eq!(cost_per_step(s), c, "{s}");
        }
        assert_eq!(stat_points(Stat::DX, -1), -20);
    }

    fn tr(cat: TraitCategory, cost: i32) -> Trait {
        Trait::new(
            TraitDef {
                id: "t".into(),
                name: "t".into(),
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
    fn breakdown_by_category() {
        let mut ch = Character::new("x");
        ch.attributes.st = 2; // 20
        ch.attributes.iq = 1; // 20
        ch.attributes.basic_speed_quarters = 2; // 10
        ch.attributes.hp = 1; // 2
        ch.traits.push(tr(TraitCategory::Advantage, 15));
        ch.traits.push(tr(TraitCategory::Disadvantage, -10));
        ch.traits.push(tr(TraitCategory::Quirk, -1));
        ch.traits.push(tr(TraitCategory::Perk, 1));
        let b = breakdown(&ch);
        assert_eq!(b.attributes, 40);
        assert_eq!(b.secondary, 12);
        assert_eq!(b.advantages, 15);
        assert_eq!(b.disadvantages, -10);
        assert_eq!(b.quirks, -1);
        assert_eq!(b.perks, 1);
        assert_eq!(b.total, 57);
        assert_eq!(b.unspent, 150 - 57);
    }
}
