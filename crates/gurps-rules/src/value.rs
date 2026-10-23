use serde::{Deserialize, Serialize};

/// Where a contribution comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "ref", rename_all = "snake_case")]
pub enum SourceKind {
    /// Bought with character points.
    Purchased,
    /// From a trait (trait name).
    Trait(String),
    /// Derived from another value (stat key).
    Derived(String),
    /// Anything else (temporary modifiers, rounding...).
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contribution {
    pub source: SourceKind,
    pub amount: f64,
    pub reason: String,
}

/// An explainable value: `base + Σ contributions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Value {
    pub base: f64,
    pub base_reason: String,
    pub contributions: Vec<Contribution>,
}

impl Value {
    pub fn new(base: f64, base_reason: impl Into<String>) -> Self {
        Self {
            base,
            base_reason: base_reason.into(),
            contributions: Vec::new(),
        }
    }

    pub fn add(&mut self, source: SourceKind, amount: f64, reason: impl Into<String>) {
        if amount != 0.0 {
            self.contributions.push(Contribution {
                source,
                amount,
                reason: reason.into(),
            });
        }
    }

    pub fn total(&self) -> f64 {
        self.base + self.contributions.iter().map(|c| c.amount).sum::<f64>()
    }

    /// Human-readable explanation lines, e.g. `["base 10 (default)", "+2 purchased", "= 12"]`.
    pub fn explain(&self) -> Vec<String> {
        let mut out = vec![format!(
            "base {} ({})",
            fmt_num(self.base),
            self.base_reason
        )];
        for c in &self.contributions {
            let sign = if c.amount >= 0.0 { "+" } else { "" };
            out.push(format!("{sign}{} {}", fmt_num(c.amount), c.reason));
        }
        out.push(format!("= {}", fmt_num(self.total())));
        out
    }
}

pub fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        let s = format!("{v:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_and_explain() {
        let mut v = Value::new(10.0, "default");
        v.add(SourceKind::Purchased, 2.0, "purchased");
        v.add(SourceKind::Trait("X".into()), 1.0, "from trait X");
        v.add(SourceKind::Purchased, 0.0, "ignored");
        assert_eq!(v.total(), 13.0);
        assert_eq!(v.contributions.len(), 2);
        assert_eq!(
            v.explain(),
            vec![
                "base 10 (default)",
                "+2 purchased",
                "+1 from trait X",
                "= 13"
            ]
        );
    }

    #[test]
    fn number_format() {
        assert_eq!(fmt_num(5.25), "5.25");
        assert_eq!(fmt_num(5.5), "5.5");
        assert_eq!(fmt_num(-3.0), "-3");
    }
}
