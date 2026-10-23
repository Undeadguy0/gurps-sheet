//! Carried weight and encumbrance (B17, # unverified).
//!
//! Levels by carried weight vs Basic Lift: None ≤ 1×BL, Light ≤ 2×, Medium ≤ 3×,
//! Heavy ≤ 6×, Extra-Heavy ≤ 10×. Move is multiplied by 1, 0.8, 0.6, 0.4, 0.2 and
//! rounded down (never below 1 if base Move is positive); Dodge gets −level.

use crate::value::fmt_num;
use gurps_core::Equipment;
use serde::{Deserialize, Serialize};

pub const BL_MULTIPLES: [f64; 5] = [1.0, 2.0, 3.0, 6.0, 10.0];
pub const MOVE_MULTIPLIERS: [f64; 5] = [1.0, 0.8, 0.6, 0.4, 0.2];
pub const LEVEL_NAMES: [&str; 5] = ["None", "Light", "Medium", "Heavy", "Extra-Heavy"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncumbranceLevel {
    pub level: u8,
    pub name: String,
    /// Upper weight limit for this level (lb).
    pub max_weight: f64,
    pub move_value: i32,
    pub dodge: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Encumbrance {
    /// 0 = None ... 4 = Extra-Heavy.
    pub level: u8,
    pub name: String,
    /// Weight counted for encumbrance (carried items only).
    pub carried_weight: f64,
    /// All items, carried or not.
    pub total_weight: f64,
    pub total_cost: f64,
    pub carried_cost: f64,
    pub basic_lift: f64,
    pub base_move: f64,
    /// Move under this encumbrance.
    pub move_value: i32,
    /// Dodge penalty (0..-4).
    pub dodge_penalty: i32,
    /// floor(Basic Speed) + 3 (no Combat Reflexes, shield or other bonuses yet).
    pub base_dodge: i32,
    pub dodge: i32,
    /// More than 10×BL: the character cannot move with this load.
    pub overloaded: bool,
    pub table: Vec<EncumbranceLevel>,
    pub explain: Vec<String>,
}

const EPS: f64 = 1e-9;

pub fn level_for(weight: f64, bl: f64) -> (u8, bool) {
    for (i, m) in BL_MULTIPLES.iter().enumerate() {
        if weight <= bl * m + EPS {
            return (i as u8, false);
        }
    }
    (4, true)
}

pub fn move_for(base_move: f64, level: u8) -> i32 {
    let m = (base_move * MOVE_MULTIPLIERS[level as usize] + EPS).floor() as i32;
    if base_move >= 1.0 {
        m.max(1)
    } else {
        m.max(0)
    }
}

pub fn compute(
    items: &[Equipment],
    basic_lift: f64,
    base_move: f64,
    basic_speed: f64,
) -> Encumbrance {
    let carried_weight: f64 = items.iter().map(Equipment::carried_weight).sum();
    let total_weight: f64 = items.iter().map(Equipment::total_weight).sum();
    let total_cost: f64 = items.iter().map(Equipment::total_cost).sum();
    let carried_cost: f64 = items.iter().map(carried_cost_of).sum();
    let (level, overloaded) = level_for(carried_weight, basic_lift);
    let base_dodge = (basic_speed + EPS).floor() as i32 + 3;
    let table = (0..5u8)
        .map(|l| EncumbranceLevel {
            level: l,
            name: LEVEL_NAMES[l as usize].into(),
            max_weight: basic_lift * BL_MULTIPLES[l as usize],
            move_value: move_for(base_move, l),
            dodge: base_dodge - l as i32,
        })
        .collect();
    let move_value = if overloaded {
        0
    } else {
        move_for(base_move, level)
    };
    let dodge_penalty = -(level as i32);
    let mut explain = vec![format!(
        "carried {} lb, BL {} lb",
        fmt_num(carried_weight),
        fmt_num(basic_lift)
    )];
    if overloaded {
        explain.push(format!(
            "> 10×BL ({} lb): too heavy to move",
            fmt_num(basic_lift * 10.0)
        ));
    } else {
        let lim = basic_lift * BL_MULTIPLES[level as usize];
        explain.push(format!(
            "≤ {}×BL ({} lb) -> {} ({level})",
            BL_MULTIPLES[level as usize],
            fmt_num(lim),
            LEVEL_NAMES[level as usize]
        ));
        explain.push(format!(
            "Move floor({} × {}) = {move_value}",
            fmt_num(base_move),
            MOVE_MULTIPLIERS[level as usize]
        ));
    }
    explain.push(format!(
        "Dodge floor({}) + 3 {} = {}",
        fmt_num(basic_speed),
        if level > 0 {
            format!("- {level}")
        } else {
            String::new()
        },
        base_dodge + dodge_penalty
    ));
    Encumbrance {
        level,
        name: LEVEL_NAMES[level as usize].into(),
        carried_weight,
        total_weight,
        total_cost,
        carried_cost,
        basic_lift,
        base_move,
        move_value,
        dodge_penalty,
        base_dodge,
        dodge: base_dodge + dodge_penalty,
        overloaded,
        table,
        explain,
    }
}

fn carried_cost_of(e: &Equipment) -> f64 {
    if !e.carried {
        return 0.0;
    }
    e.def.cost * e.quantity as f64 + e.contents.iter().map(carried_cost_of).sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gurps_core::EquipmentDef;

    fn item(w: f64, q: u32) -> Equipment {
        Equipment::new(
            EquipmentDef {
                name: "x".into(),
                weight: w,
                cost: 10.0,
                ..Default::default()
            },
            q,
        )
    }

    #[test]
    fn thresholds_are_inclusive() {
        let bl = 20.0;
        let cases = [
            (0.0, 0),
            (20.0, 0),
            (20.5, 1),
            (40.0, 1),
            (40.1, 2),
            (60.0, 2),
            (120.0, 3),
            (200.0, 4),
        ];
        for (w, l) in cases {
            assert_eq!(level_for(w, bl), (l, false), "{w}");
        }
        assert_eq!(level_for(200.5, bl), (4, true));
    }

    #[test]
    fn move_multipliers_round_down() {
        let m: Vec<i32> = (0..5).map(|l| move_for(7.0, l)).collect();
        assert_eq!(m, [7, 5, 4, 2, 1]);
        assert_eq!(move_for(5.0, 4), 1);
        assert_eq!(move_for(3.0, 4), 1, "minimum 1");
        assert_eq!(move_for(10.0, 1), 8, "float 0.8*10 is exactly 8");
    }

    #[test]
    fn not_carried_is_ignored() {
        let mut a = item(30.0, 1);
        a.carried = false;
        let e = compute(&[a, item(5.0, 2)], 20.0, 5.0, 5.5);
        assert_eq!((e.carried_weight, e.total_weight, e.level), (10.0, 40.0, 0));
        assert_eq!((e.total_cost, e.carried_cost), (30.0, 20.0));
        assert_eq!((e.base_dodge, e.dodge, e.move_value), (8, 8, 5));
        assert_eq!(e.table.len(), 5);
    }

    #[test]
    fn overloaded() {
        let e = compute(&[item(250.0, 1)], 20.0, 5.0, 5.0);
        assert!(e.overloaded);
        assert_eq!((e.level, e.move_value, e.dodge), (4, 0, 4));
    }
}
