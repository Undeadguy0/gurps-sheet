//! 3d6 success rolls (B343, critical rules B348).

use serde::{Deserialize, Serialize};

/// Source of d6 results; inject a deterministic one in tests.
pub trait DiceRng {
    /// Returns a value in 1..=6.
    fn d6(&mut self) -> u8;
}

/// Small non-cryptographic PRNG (SplitMix64).
#[derive(Debug, Clone)]
pub struct SplitMix(pub u64);

impl SplitMix {
    pub fn from_time() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9);
        SplitMix(nanos)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

impl DiceRng for SplitMix {
    fn d6(&mut self) -> u8 {
        (self.next_u64() % 6) as u8 + 1
    }
}

/// Replays fixed dice (cycling).
#[derive(Debug, Clone)]
pub struct FixedDice {
    values: Vec<u8>,
    idx: usize,
}

impl FixedDice {
    pub fn new(values: Vec<u8>) -> Self {
        assert!(!values.is_empty());
        Self { values, idx: 0 }
    }
}

impl DiceRng for FixedDice {
    fn d6(&mut self) -> u8 {
        let v = self.values[self.idx % self.values.len()];
        self.idx += 1;
        v
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    CriticalSuccess,
    Success,
    Failure,
    CriticalFailure,
}

impl Outcome {
    pub fn is_success(self) -> bool {
        matches!(self, Outcome::CriticalSuccess | Outcome::Success)
    }
}

/// Outcome of a 3d6 roll against an effective skill.
pub fn outcome(roll: i32, skill: i32) -> Outcome {
    if roll <= 4 || (roll == 5 && skill >= 15) || (roll == 6 && skill >= 16) {
        return Outcome::CriticalSuccess;
    }
    if roll == 18 {
        return Outcome::CriticalFailure;
    }
    if roll == 17 {
        return if skill <= 15 {
            Outcome::CriticalFailure
        } else {
            Outcome::Failure
        };
    }
    if roll >= skill + 10 {
        return Outcome::CriticalFailure;
    }
    if roll <= skill {
        Outcome::Success
    } else {
        Outcome::Failure
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollResult {
    pub target: i32,
    pub modifier: i32,
    pub effective: i32,
    pub dice: [u8; 3],
    pub roll: i32,
    /// effective - roll (positive = margin of success).
    pub margin: i32,
    pub outcome: Outcome,
}

pub fn roll_3d6(rng: &mut impl DiceRng) -> [u8; 3] {
    [rng.d6(), rng.d6(), rng.d6()]
}

pub fn success_roll(target: i32, modifier: i32, rng: &mut impl DiceRng) -> RollResult {
    let dice = roll_3d6(rng);
    let roll = dice.iter().map(|&d| d as i32).sum();
    let effective = target + modifier;
    RollResult {
        target,
        modifier,
        effective,
        dice,
        roll,
        margin: effective - roll,
        outcome: outcome(roll, effective),
    }
}

/// A damage roll (e.g. "2d-1"): dice, adds and the total with the minimum applied
/// (B378, # unverified: 0 for crushing, otherwise 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageRoll {
    pub expr: String,
    pub dice: Vec<u8>,
    pub adds: i32,
    pub total: i32,
}

/// Parses "NdM", "Nd", "Nd+K", "Nd-K" (ignores a trailing damage type).
pub fn parse_dice(expr: &str) -> Option<(u32, i32)> {
    let s = expr.split_whitespace().next()?;
    let (n, rest) = s.split_once('d')?;
    let n: u32 = n.parse().ok()?;
    let adds = if rest.is_empty() {
        0
    } else {
        rest.parse().ok()?
    };
    (n > 0 && n <= 100).then_some((n, adds))
}

pub fn damage_roll(expr: &str, crushing: bool, rng: &mut impl DiceRng) -> Option<DamageRoll> {
    let (n, adds) = parse_dice(expr)?;
    let dice: Vec<u8> = (0..n).map(|_| rng.d6()).collect();
    let sum = dice.iter().map(|&d| d as i32).sum::<i32>() + adds;
    Some(DamageRoll {
        expr: expr.into(),
        dice,
        adds,
        total: sum.max(if crushing { 0 } else { 1 }),
    })
}

#[cfg(test)]
mod tests {
    use super::Outcome::*;
    use super::*;

    #[test]
    fn three_and_four_always_crit() {
        for skill in [-5, 3, 10, 20] {
            assert_eq!(outcome(3, skill), CriticalSuccess);
            assert_eq!(outcome(4, skill), CriticalSuccess);
        }
    }

    #[test]
    fn five_and_six_thresholds() {
        assert_eq!(outcome(5, 14), Success);
        assert_eq!(outcome(5, 15), CriticalSuccess);
        assert_eq!(outcome(6, 15), Success);
        assert_eq!(outcome(6, 16), CriticalSuccess);
        assert_eq!(outcome(5, 4), Failure);
    }

    #[test]
    fn seventeen_and_eighteen() {
        assert_eq!(outcome(18, 30), CriticalFailure);
        assert_eq!(outcome(17, 15), CriticalFailure);
        assert_eq!(outcome(17, 16), Failure);
        assert_eq!(outcome(17, 25), Failure); // 17 always fails
        assert_eq!(outcome(16, 25), Success);
    }

    #[test]
    fn ten_over_skill_is_crit_fail() {
        assert_eq!(outcome(15, 5), CriticalFailure);
        assert_eq!(outcome(14, 5), Failure);
        assert_eq!(outcome(13, 3), CriticalFailure);
    }

    #[test]
    fn normal_range() {
        assert_eq!(outcome(10, 10), Success);
        assert_eq!(outcome(11, 10), Failure);
    }

    #[test]
    fn injected_rng() {
        let mut rng = FixedDice::new(vec![1, 1, 2]);
        let r = success_roll(12, -2, &mut rng);
        assert_eq!(r.dice, [1, 1, 2]);
        assert_eq!(r.roll, 4);
        assert_eq!(r.effective, 10);
        assert_eq!(r.margin, 6);
        assert_eq!(r.outcome, CriticalSuccess);
        let mut rng = FixedDice::new(vec![6]);
        assert_eq!(success_roll(20, 0, &mut rng).outcome, CriticalFailure);
    }

    #[test]
    fn damage_rolls() {
        assert_eq!(parse_dice("2d-1"), Some((2, -1)));
        assert_eq!(parse_dice("1d"), Some((1, 0)));
        assert_eq!(parse_dice("1d+2 cut"), Some((1, 2)));
        assert_eq!(parse_dice("x"), None);
        let mut rng = FixedDice::new(vec![1]);
        let r = damage_roll("1d-3", false, &mut rng).unwrap();
        assert_eq!((r.dice.clone(), r.total), (vec![1], 1));
        assert_eq!(damage_roll("1d-3", true, &mut rng).unwrap().total, 0);
        let mut rng = FixedDice::new(vec![6, 5]);
        assert_eq!(damage_roll("2d+1", false, &mut rng).unwrap().total, 12);
    }

    #[test]
    fn seeded_is_deterministic() {
        let a = success_roll(12, 0, &mut SplitMix(7));
        let b = success_roll(12, 0, &mut SplitMix(7));
        assert_eq!(a, b);
    }

    #[test]
    fn splitmix_range() {
        let mut r = SplitMix(42);
        for _ in 0..1000 {
            let v = r.d6();
            assert!((1..=6).contains(&v));
        }
    }
}
