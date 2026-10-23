//! Explanation lines for trait cost with enhancements/limitations.

use gurps_core::modifier::MIN_TOTAL_PERCENT;
use gurps_core::{NegativeRounding, Trait};

fn pct(v: i32) -> String {
    if v >= 0 {
        format!("+{v}%")
    } else {
        format!("{v}%")
    }
}

/// E.g. ["base cost 20", "+40% Area Effect", "total +40%", "20 × 140% = 28 -> 28"].
/// Empty when the trait has no modifiers.
pub fn trait_cost_explain(t: &Trait) -> Vec<String> {
    trait_cost_explain_with(t, NegativeRounding::default())
}

/// [`trait_cost_explain`] with the character's negative rounding rule.
pub fn trait_cost_explain_with(t: &Trait, r: NegativeRounding) -> Vec<String> {
    if t.modifiers.is_empty() {
        return Vec::new();
    }
    let base = t.base_cost();
    let mut out = vec![format!("base cost {base}")];
    for m in &t.modifiers {
        out.push(format!("{} {}", pct(m.def.cost_percent), m.def.name));
    }
    let total = t.modifier_percent();
    if total < MIN_TOTAL_PERCENT {
        out.push(format!(
            "total {} floored at {}",
            pct(total),
            pct(MIN_TOTAL_PERCENT)
        ));
    } else {
        out.push(format!("total {}", pct(total)));
    }
    let eff = 100 + total.max(MIN_TOTAL_PERCENT);
    let exact = base as f64 * eff as f64 / 100.0;
    out.push(format!(
        "{base} × {eff}% = {} -> {} ({})",
        crate::value::fmt_num(exact),
        t.cost_with(r),
        if exact < 0.0 && r == NegativeRounding::ByMagnitude && exact.fract() != 0.0 {
            "rounded away from zero"
        } else {
            "rounded up"
        }
    ));
    out
}
