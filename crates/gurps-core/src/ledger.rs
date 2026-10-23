//! Character point pool: a ledger of earned and non-character spends.
//!
//! The character's own cost is never stored; it is computed from the sheet.
//! unspent = earned − live character cost − spends (may go negative; the UI warns).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LedgerKind {
    /// Starting points (the old `points_budget`).
    Starting,
    SessionAward,
    /// GM bonus: roleplay, achieved goal, good roll...
    GmBonus,
    /// Points from training/study (optionally linked to a skill).
    Training,
    /// Disadvantage or quirk taken in play (gives points).
    NewDisadvantage,
    /// Manual correction (either sign).
    Correction,
    /// Spent on influence rolls (B347, # unverified).
    Influence,
    /// Impulse Buy (Power-Ups 5, # unverified).
    ImpulseBuy,
    /// Buying off a disadvantage.
    BuyOff,
    /// Any other spend; the note is required.
    Other,
}

impl LedgerKind {
    pub const ALL: [LedgerKind; 10] = [
        Self::Starting,
        Self::SessionAward,
        Self::GmBonus,
        Self::Training,
        Self::NewDisadvantage,
        Self::Correction,
        Self::Influence,
        Self::ImpulseBuy,
        Self::BuyOff,
        Self::Other,
    ];
    /// Spends that do not go into the character sheet.
    pub fn is_spend(self) -> bool {
        matches!(
            self,
            Self::Influence | Self::ImpulseBuy | Self::BuyOff | Self::Other
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: u32,
    /// Free-form date, usually YYYY-MM-DD.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub date: String,
    /// Signed: income > 0, spends < 0, corrections either way.
    pub amount: i32,
    pub kind: LedgerKind,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// Training: the skill id it was for (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
}

impl LedgerEntry {
    /// Normalizes the sign by kind and checks required fields.
    pub fn normalized(mut self) -> Result<Self, String> {
        self.note = self.note.trim().to_string();
        self.date = self.date.trim().to_string();
        if self.kind == LedgerKind::Other && self.note.is_empty() {
            return Err("an 'other' spend needs a note".into());
        }
        if self.amount == 0 {
            return Err("amount must not be zero".into());
        }
        if self.kind.is_spend() {
            self.amount = -self.amount.abs();
        } else if self.kind != LedgerKind::Correction {
            self.amount = self.amount.abs();
        }
        if self.kind != LedgerKind::Training {
            self.skill = None;
        }
        Ok(self)
    }
}

/// Totals of a ledger (no character cost involved).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerTotals {
    pub starting: i32,
    /// Everything earned after the start (incl. corrections).
    pub income: i32,
    /// starting + income.
    pub earned: i32,
    /// Spent outside the sheet (positive number).
    pub spent_outside: i32,
    /// earned − spent_outside: what the sheet may cost.
    pub available: i32,
}

pub fn totals(entries: &[LedgerEntry]) -> LedgerTotals {
    let mut t = LedgerTotals::default();
    for e in entries {
        match e.kind {
            LedgerKind::Starting => t.starting += e.amount,
            k if k.is_spend() => t.spent_outside -= e.amount,
            _ => t.income += e.amount,
        }
    }
    t.earned = t.starting + t.income;
    t.available = t.earned - t.spent_outside;
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(id: u32, kind: LedgerKind, amount: i32, note: &str) -> LedgerEntry {
        LedgerEntry {
            id,
            date: String::new(),
            amount,
            kind,
            note: note.into(),
            skill: None,
        }
    }

    #[test]
    fn signs_and_totals() {
        let es: Vec<_> = [
            e(1, LedgerKind::Starting, 150, ""),
            e(2, LedgerKind::SessionAward, -3, ""),
            e(3, LedgerKind::GmBonus, 1, "подарок от Мастера"),
            e(4, LedgerKind::Correction, -2, ""),
            e(5, LedgerKind::Influence, 2, ""),
            e(6, LedgerKind::BuyOff, -5, ""),
        ]
        .into_iter()
        .map(|x| x.normalized().unwrap())
        .collect();
        assert_eq!(es[1].amount, 3);
        assert_eq!(es[3].amount, -2);
        assert_eq!(es[4].amount, -2);
        let t = totals(&es);
        assert_eq!(
            (t.starting, t.income, t.earned, t.spent_outside, t.available),
            (150, 2, 152, 7, 145)
        );
    }

    #[test]
    fn other_needs_note_and_zero_is_rejected() {
        assert!(e(1, LedgerKind::Other, 2, " ").normalized().is_err());
        assert!(e(1, LedgerKind::Other, 2, "взятка").normalized().is_ok());
        assert!(e(1, LedgerKind::GmBonus, 0, "").normalized().is_err());
        let s = serde_json::to_string(&e(1, LedgerKind::BuyOff, -1, "")).unwrap();
        assert!(s.contains("\"buy-off\""), "{s}");
    }
}
