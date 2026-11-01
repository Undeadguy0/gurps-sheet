//! `gurps` — JSON-emitting CLI (friendly to Nushell: `gurps calc hero.json | from json`).
//!
//! Commands:
//!   calc <file.json>
//!   roll <target> [--mod N] [--seed S]
//!   validate <file.json>
//!
//! `calc`/`validate` считают через общую БД (`gurps-calc`), подключение - env `SURREALDB_*`.
#![cfg_attr(test, allow(clippy::field_reassign_with_default))]

use gurps_dice::{success_roll, SplitMix};
use gurps_library::calc::{Engine, Issue};
use gurps_library::sheet::Sheet;
use gurps_library::DbConfig;
use serde_json::{json, Value};
use std::process::ExitCode;

const USAGE: &str = "usage:\n  gurps calc <file.json>\n  gurps roll <target> [--mod N] [--seed S]\n  gurps validate <file.json>";

/// Лист и проверки персонажа через БД.
fn compute(path: &str, cfg: &DbConfig) -> Result<(Sheet, Vec<Issue>), String> {
    let ch = gurps_io::load(path).map_err(|e| e.to_string())?;
    let lib = gurps_library::Library::load_db(cfg)?;
    let eng = Engine::connect(cfg)?;
    let calc = eng.compute(&ch);
    let sheet = match &calc {
        Ok(c) => gurps_library::sheet::build(&ch, &lib, c, &|_, _| None),
        Err(_) => Sheet::default(),
    };
    let issues = gurps_library::sheet::issues(&ch, &lib, &calc, &sheet);
    Ok((sheet, issues))
}

fn run(args: &[String], cfg: &DbConfig) -> Result<(Value, bool), String> {
    match args.first().map(String::as_str) {
        Some("calc") => {
            let path = args.get(1).ok_or(USAGE)?;
            let (sheet, _) = compute(path, cfg)?;
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
            let issues = match compute(path, cfg) {
                Ok((_, issues)) => issues,
                Err(e) => vec![Issue {
                    severity: "error",
                    message: e,
                }],
            };
            let valid = !issues.iter().any(|i| i.severity == "error");
            Ok((json!({"valid": valid, "issues": issues}), valid))
        }
        _ => Err(USAGE.to_string()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args, &DbConfig::from_env()) {
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

    fn no_db() -> DbConfig {
        DbConfig {
            host: "ws://127.0.0.1:1".into(),
            namespace: "x".into(),
            database: "x".into(),
            user: "x".into(),
            password: "x".into(),
        }
    }

    #[test]
    fn calc_and_validate_via_db() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let Some((srv, _db)) = rt.block_on(gurps_db::testkit::start()) else {
            return;
        };
        let mut ch = gurps_core::Character::new("CLI".to_string());
        ch.set_attribute("ST", 3);
        ch.set_attribute("DX", 1);
        let p = std::env::temp_dir().join(format!("gurps-cli-{}.json", std::process::id()));
        gurps_io::save(&p, &ch).unwrap();
        let path = p.display().to_string();
        let (v, ok) = run(&s(&["calc", &path]), &srv.cfg).unwrap();
        assert!(ok);
        assert_eq!(v["attributes"][0]["key"], "ST");
        assert_eq!(v["attributes"][0]["total"], 13.0);
        // 30 (ST) + 20 (DX)
        assert_eq!(v["points"]["total"], 50);
        let (v, ok) = run(&s(&["validate", &path]), &srv.cfg).unwrap();
        assert!(ok, "{v}");
        std::fs::remove_file(p).unwrap();
        // Пример v2: все ссылки находятся в каталоге.
        let hero = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/hero.json");
        let (v, _) = run(&s(&["validate", hero]), &srv.cfg).unwrap();
        assert!(!v.to_string().contains("нет в каталоге"), "{v}");
    }

    #[test]
    fn roll_seeded() {
        let (v, _) = run(&s(&["roll", "12", "--mod", "-2", "--seed", "1"]), &no_db()).unwrap();
        assert_eq!(v["effective"], 10);
        let r = v["roll"].as_i64().unwrap();
        assert!((3..=18).contains(&r));
    }

    #[test]
    fn validate_errors() {
        let (_, ok) = run(&s(&["validate", "/nonexistent.json"]), &no_db()).unwrap();
        assert!(!ok);
        assert!(run(&s(&["bogus"]), &no_db()).is_err());
        assert!(run(&s(&["roll", "x"]), &no_db()).is_err());
    }
}
