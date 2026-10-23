//! `gurps` — JSON-emitting CLI (friendly to Nushell: `gurps calc hero.json | from json`).
//!
//! Commands:
//!   calc <file.json>
//!   roll <target> [--mod N] [--seed S]
//!   validate <file.json>

use gurps_rules::dice::{success_roll, SplitMix};
use serde_json::{json, Value};
use std::process::ExitCode;

const USAGE: &str = "usage:\n  gurps calc <file.json>\n  gurps roll <target> [--mod N] [--seed S]\n  gurps validate <file.json>";

fn run(args: &[String]) -> Result<(Value, bool), String> {
    match args.first().map(String::as_str) {
        Some("calc") => {
            let path = args.get(1).ok_or(USAGE)?;
            let ch = gurps_io::load(path).map_err(|e| e.to_string())?;
            let sheet = gurps_rules::compute_sheet(&ch);
            Ok((
                serde_json::to_value(sheet).map_err(|e| e.to_string())?,
                true,
            ))
        }
        Some("roll") => {
            let target: i32 = args
                .get(1)
                .ok_or(USAGE)?
                .parse()
                .map_err(|_| "target must be an integer".to_string())?;
            let mut modifier = 0;
            let mut seed = None;
            let mut i = 2;
            while i < args.len() {
                let val = args
                    .get(i + 1)
                    .ok_or(format!("{} needs a value", args[i]))?;
                match args[i].as_str() {
                    "--mod" | "-m" => {
                        modifier = val.parse().map_err(|_| "--mod must be an integer")?
                    }
                    "--seed" => seed = Some(val.parse().map_err(|_| "--seed must be an integer")?),
                    other => return Err(format!("unknown option {other}\n{USAGE}")),
                }
                i += 2;
            }
            let mut rng = seed.map(SplitMix).unwrap_or_else(SplitMix::from_time);
            let r = success_roll(target, modifier, &mut rng);
            Ok((serde_json::to_value(r).map_err(|e| e.to_string())?, true))
        }
        Some("validate") => {
            let path = args.get(1).ok_or(USAGE)?;
            match gurps_io::load(path) {
                Err(e) => Ok((
                    json!({"valid": false, "issues": [{"severity":"error","message": e.to_string()}]}),
                    false,
                )),
                Ok(ch) => {
                    let issues = gurps_rules::validate::validate(&ch);
                    let valid = !issues
                        .iter()
                        .any(|i| i.severity == gurps_rules::validate::Severity::Error);
                    Ok((json!({"valid": valid, "issues": issues}), valid))
                }
            }
        }
        _ => Err(USAGE.to_string()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok((v, ok)) => {
            println!("{}", serde_json::to_string_pretty(&v).expect("json"));
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    fn hero() -> String {
        format!("{}/../../examples/hero.json", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn calc_hero() {
        let (v, ok) = run(&s(&["calc", &hero()])).unwrap();
        assert!(ok);
        assert_eq!(v["attributes"][0]["key"], "ST");
        assert_eq!(v["attributes"][0]["total"], 13.0);
        // 20 (ST) + 20 (DX) + 10 (HT) + 10 + 6 - 5
        assert_eq!(v["points"]["total"], 61);
    }

    #[test]
    fn roll_seeded() {
        let (v, _) = run(&s(&["roll", "12", "--mod", "-2", "--seed", "1"])).unwrap();
        assert_eq!(v["effective"], 10);
        let r = v["roll"].as_i64().unwrap();
        assert!((3..=18).contains(&r));
    }

    #[test]
    fn validate_hero_and_errors() {
        let (v, ok) = run(&s(&["validate", &hero()])).unwrap();
        assert!(ok, "{v}");
        let (_, ok) = run(&s(&["validate", "/nonexistent.json"])).unwrap();
        assert!(!ok);
        assert!(run(&s(&["bogus"])).is_err());
        assert!(run(&s(&["roll", "x"])).is_err());
    }
}
