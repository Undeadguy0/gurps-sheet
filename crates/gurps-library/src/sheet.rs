//! Лист для UI из результата `gurps-calc` (этап 4): форма прежнего `gurps_rules::Sheet`,
//! все числа - из [`Calc`]; строки пояснений собираются здесь (приложение) из
//! структурных данных расчёта (`AttrRow`, `SkillLevel`, `Encumbrance`, `Defenses`).
//! Подписи атрибутов и уровней нагрузки - переводы из БД (`label(kind, code)`).
use crate::calc::{sid, Calc, Issue};
use gurps_core::{Character, Feature, Stat};
use gurps_db::gurps_calc::{self, skills::SkillLevel};
use serde::Serialize;

/// Откуда вклад в значение.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "ref", rename_all = "snake_case")]
pub enum SourceKind {
    Purchased,
    Trait(String),
    Derived(String),
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Contribution {
    pub source: SourceKind,
    pub amount: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Value {
    pub base: f64,
    pub base_reason: String,
    pub contributions: Vec<Contribution>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct StatRow {
    pub key: String,
    pub label: String,
    pub level: i32,
    pub step: f64,
    pub cost_per_step: i32,
    pub points: i32,
    pub total: f64,
    pub value: Value,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DamageRow {
    pub thrust: String,
    pub swing: String,
    pub based_on_st: i32,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PointsBreakdown {
    pub attributes: i32,
    pub secondary: i32,
    pub advantages: i32,
    pub perks: i32,
    pub disadvantages: i32,
    pub quirks: i32,
    pub skills: i32,
    pub spells: i32,
    pub total: i32,
    pub budget: i32,
    pub unspent: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct EncumbranceLevel {
    pub level: u8,
    pub name: String,
    pub max_weight: f64,
    pub move_value: i32,
    pub dodge: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Encumbrance {
    pub level: u8,
    pub name: String,
    pub carried_weight: f64,
    pub total_weight: f64,
    pub total_cost: f64,
    pub carried_cost: f64,
    pub basic_lift: f64,
    pub base_move: f64,
    pub move_value: i32,
    pub dodge_penalty: i32,
    pub base_dodge: i32,
    pub dodge: i32,
    pub overloaded: bool,
    pub table: Vec<EncumbranceLevel>,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Defense {
    pub value: i32,
    pub from: String,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ArmorRow {
    pub item: String,
    pub dr: String,
    pub value: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Defenses {
    pub dodge: Defense,
    pub parry: Option<Defense>,
    pub block: Option<Defense>,
    pub db: i32,
    pub db_from: Option<String>,
    pub bonus: i32,
    pub armor: Vec<ArmorRow>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AttackRow {
    pub mode: String,
    pub skill: String,
    pub skill_id: Option<String>,
    pub via_technique: bool,
    pub level: Option<i32>,
    pub damage_expr: String,
    pub damage: String,
    pub damage_type: String,
    pub reach: String,
    pub parry: Option<i32>,
    pub can_parry: bool,
    pub parry_modifier: i32,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct WeaponRow {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub equipment: Option<String>,
    pub available: bool,
    pub ready: bool,
    pub attacks: Vec<AttackRow>,
}

/// Строка навыка (и приёма/заклинания: `max_level` только у приёмов).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SkillRow {
    pub index: usize,
    pub level: Option<i32>,
    /// `DX+1` - относительно атрибута (у приёма - относительно навыка).
    pub relative: Option<String>,
    pub from_default: bool,
    pub max_level: Option<i32>,
    pub explain: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Sheet {
    pub name: String,
    pub attributes: Vec<StatRow>,
    pub secondary: Vec<StatRow>,
    pub basic_lift: StatRow,
    pub damage: DamageRow,
    pub encumbrance: Encumbrance,
    pub weapons: Vec<WeaponRow>,
    pub defenses: Defenses,
    pub points: PointsBreakdown,
    #[serde(skip)]
    pub skills: Vec<SkillRow>,
    #[serde(skip)]
    pub techniques: Vec<SkillRow>,
    #[serde(skip)]
    pub spells: Vec<SkillRow>,
}

impl Sheet {
    /// Строка атрибута по ключу листа (`ST`, `Speed`, `Move`, `BasicLift`...).
    pub fn row(&self, stat: Stat) -> Option<&StatRow> {
        let k = stat.key();
        self.attributes
            .iter()
            .chain(&self.secondary)
            .chain(std::iter::once(&self.basic_lift))
            .find(|r| r.key == k)
    }
}

pub fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", (v * 100.0).round() / 100.0)
    }
}

fn signed(v: f64) -> String {
    if v >= 0.0 {
        format!("+{}", fmt_num(v))
    } else {
        fmt_num(v)
    }
}

/// Код атрибута БД -> `Stat` листа.
fn stat_of(code: &str) -> Option<Stat> {
    match code {
        "Speed" => Some(Stat::BasicSpeed),
        "Move" => Some(Stat::BasicMove),
        c => Stat::parse(c),
    }
}

/// Бонусы черт к атрибуту (для разбивки вклада; сумма - из расчёта).
fn trait_bonuses(ch: &Character, stat: Stat) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    for t in &ch.traits {
        let lv = t.levels.max(1) as f64;
        for f in &t.def.features {
            if let Feature::AttributeBonus {
                target,
                amount,
                per_level,
            } = f
            {
                if *target == stat {
                    out.push((
                        t.display_name(),
                        if *per_level { amount * lv } else { *amount },
                    ));
                }
            }
        }
    }
    out
}

fn stat_row(
    ch: &Character,
    r: &gurps_calc::attributes::AttrRow,
    label: &dyn Fn(&str, &str) -> Option<String>,
) -> StatRow {
    let key = stat_of(&r.code).map_or(r.code.clone(), |s| s.key().to_string());
    let mut contributions = Vec::new();
    let mut explain = vec![format!("база {}", fmt_num(r.base))];
    if r.levels != 0 {
        let amount = r.levels as f64 * r.step;
        contributions.push(Contribution {
            source: SourceKind::Purchased,
            amount,
            reason: format!("куплено ({} оч.)", r.points),
        });
        explain.push(format!("{} куплено ({} оч.)", signed(amount), r.points));
    }
    if r.bonus != 0.0 {
        let named = stat_of(&r.code)
            .map(|s| trait_bonuses(ch, s))
            .unwrap_or_default();
        let named_sum: f64 = named.iter().map(|(_, a)| a).sum();
        for (n, a) in &named {
            contributions.push(Contribution {
                source: SourceKind::Trait(n.clone()),
                amount: *a,
                reason: format!("черта {n}"),
            });
            explain.push(format!("{} черта {n}", signed(*a)));
        }
        let rest = r.bonus - named_sum;
        if rest.abs() > 1e-9 {
            contributions.push(Contribution {
                source: SourceKind::Other("bonus".into()),
                amount: rest,
                reason: "прочие бонусы".into(),
            });
            explain.push(format!("{} прочие бонусы", signed(rest)));
        }
    }
    if r.set {
        explain.push(format!("= {} (задано)", fmt_num(r.value)));
    } else {
        explain.push(format!("= {}", fmt_num(r.value)));
    }
    StatRow {
        label: label("attribute", &r.code).unwrap_or_else(|| key.clone()),
        key,
        level: r.levels as i32,
        step: r.step,
        cost_per_step: r.cost_per_level.unwrap_or(0) as i32,
        points: r.points as i32,
        total: r.value,
        value: Value {
            base: r.base,
            base_reason: "база".into(),
            contributions,
        },
        explain,
    }
}

fn rel(attr: &str, d: i64) -> String {
    match d {
        0 => attr.to_string(),
        d if d > 0 => format!("{attr}+{d}"),
        d => format!("{attr}{d}"),
    }
}

fn skill_row(index: usize, l: Option<&SkillLevel>, attr: &str) -> SkillRow {
    let Some(l) = l else {
        return SkillRow {
            index,
            level: None,
            relative: None,
            from_default: false,
            max_level: None,
            explain: vec!["нет в расчёте".into()],
        };
    };
    let mut explain = vec![format!("{} оч.", l.points)];
    if let Some(r) = l.relative {
        explain.push(rel(attr, r));
    }
    if l.is_default() {
        explain.push(format!("по умолчанию ({})", l.source));
    }
    if let Some(m) = l.max_level {
        explain.push(format!("предел {m}"));
    }
    if let Some(v) = l.level {
        explain.push(format!("= {v}"));
    }
    SkillRow {
        index,
        level: l.level.map(|v| v as i32),
        relative: l.relative.map(|r| rel(attr, r)),
        from_default: l.is_default(),
        max_level: l.max_level.map(|v| v as i32),
        explain,
    }
}

/// Ищет запись расчёта по (sid, spec), по порядку; занятые пропускает.
fn take<'a>(
    list: &'a [SkillLevel],
    used: &mut [bool],
    s: &str,
    spec: Option<&str>,
) -> Option<&'a SkillLevel> {
    let spec = spec.map(str::trim).filter(|x| !x.is_empty());
    let i = list.iter().enumerate().position(|(i, l)| {
        !used[i] && l.sid == s && l.spec.as_deref().map(str::trim).filter(|x| !x.is_empty()) == spec
    })?;
    used[i] = true;
    list.get(i)
}

pub fn build(ch: &Character, c: &Calc, label: &dyn Fn(&str, &str) -> Option<String>) -> Sheet {
    let out = &c.computed;
    let a = &out.attributes;
    let mut attributes = Vec::new();
    let mut secondary = Vec::new();
    for r in &a.rows {
        let row = stat_row(ch, r, label);
        if r.kind == "basic" {
            attributes.push(row);
        } else {
            secondary.push(row);
        }
    }
    // Порядок листа: ST, DX, IQ, HT; HP, Will, Per, FP, Speed, Move; прочие - в конце.
    let order = |r: &StatRow| {
        Stat::PRIMARY
            .iter()
            .chain(Stat::SECONDARY.iter())
            .position(|s| s.key() == r.key)
            .unwrap_or(usize::MAX)
    };
    attributes.sort_by_key(order);
    secondary.sort_by_key(order);
    let bl = a.basic_lift.unwrap_or(0.0);
    let lift_st = a.lifting_st.unwrap_or(0.0);
    let basic_lift = StatRow {
        key: Stat::BasicLift.key().into(),
        label: label("attribute", "BasicLift").unwrap_or_else(|| "Basic Lift".into()),
        level: 0,
        step: 1.0,
        cost_per_step: 0,
        points: 0,
        total: bl,
        value: Value {
            base: bl,
            base_reason: format!("ST {}² / 5", fmt_num(lift_st)),
            contributions: Vec::new(),
        },
        explain: vec![
            format!("ST для подъёма {}", fmt_num(lift_st)),
            format!("= {}² / 5 = {}", fmt_num(lift_st), fmt_num(bl)),
        ],
    };
    let dice = |d: Option<gurps_calc::model::Dice>| {
        d.map(gurps_calc::attacks::dice_text).unwrap_or_default()
    };
    let sst = a.striking_st.unwrap_or(0.0);
    let damage = DamageRow {
        thrust: dice(a.thrust),
        swing: dice(a.swing),
        based_on_st: sst.floor() as i32,
        explain: vec![format!("ST для удара {} -> таблица урона", fmt_num(sst))],
    };

    let p = &out.points;
    let points = PointsBreakdown {
        attributes: p.basic_attributes as i32,
        secondary: p.secondary_attributes as i32,
        advantages: p.advantages as i32,
        perks: p.perks as i32,
        disadvantages: (p.disadvantages - p.quirks) as i32,
        quirks: p.quirks as i32,
        skills: (p.skills + p.techniques) as i32,
        spells: p.spells as i32,
        total: p.total as i32,
        budget: p.ledger_budget as i32,
        unspent: p.unspent as i32,
    };

    let e = &out.encumbrance;
    let eq = &out.equipment;
    let enc_name = |code: Option<&str>| {
        code.map(|c| label("encumbrance", c).unwrap_or_else(|| c.to_string()))
            .unwrap_or_default()
    };
    let level = e.level.unwrap_or(0).clamp(0, 255) as u8;
    let base_dodge = e.base_dodge.unwrap_or(0) as i32;
    let dodge = e.dodge.unwrap_or(0) as i32;
    let mut enc_explain = vec![format!(
        "несёт {} из BL {}",
        fmt_num(e.carried_weight),
        fmt_num(e.basic_lift.unwrap_or(0.0))
    )];
    if e.overloaded {
        enc_explain.push("перегруз: больше предельного веса".into());
    }
    let encumbrance = Encumbrance {
        level,
        name: enc_name(e.code.as_deref()),
        carried_weight: e.carried_weight,
        total_weight: eq.total_weight,
        total_cost: eq.total_cost,
        carried_cost: eq.carried_cost,
        basic_lift: e.basic_lift.unwrap_or(0.0),
        base_move: e.base_move.unwrap_or(0.0),
        move_value: e.move_value.unwrap_or(0) as i32,
        dodge_penalty: base_dodge - dodge,
        base_dodge,
        dodge,
        overloaded: e.overloaded,
        table: e
            .table
            .iter()
            .map(|l| EncumbranceLevel {
                level: l.level.clamp(0, 255) as u8,
                name: enc_name(Some(&l.code)),
                max_weight: l.max_weight.unwrap_or(0.0),
                move_value: l.move_value.unwrap_or(0) as i32,
                dodge: l.dodge.unwrap_or(0) as i32,
            })
            .collect(),
        explain: enc_explain,
    };

    let d = &out.defenses;
    let def = |v: i64, from: String, bonus: i64| {
        let mut explain = vec![format!("от {from}")];
        if bonus != 0 {
            explain.push(format!("{} бонусы", signed(bonus as f64)));
        }
        if d.db != 0 {
            explain.push(format!("{} DB", signed(d.db as f64)));
        }
        explain.push(format!("= {v}"));
        Defense {
            value: v as i32,
            from,
            explain,
        }
    };
    let armor = eq
        .armor
        .iter()
        .map(|r| {
            let value = r.locations.iter().filter_map(|l| l.dr).max().unwrap_or(0);
            let dr = r
                .locations
                .iter()
                .map(|l| {
                    format!(
                        "{} {}",
                        l.loc,
                        l.dr.map(|v| v.to_string()).unwrap_or_else(|| "?".into())
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            ArmorRow {
                item: r.item.clone(),
                dr,
                value: value as i32,
            }
        })
        .collect();
    let defenses = Defenses {
        dodge: def(d.dodge.unwrap_or(0), "нагрузки".into(), d.dodge_bonus),
        parry: d.parry.map(|v| {
            def(
                v,
                d.parry_from
                    .clone()
                    .or_else(|| d.parry_item.clone())
                    .unwrap_or_default(),
                d.parry_bonus,
            )
        }),
        block: d.block.map(|v| {
            def(
                v,
                d.block_skill
                    .clone()
                    .or_else(|| d.block_item.clone())
                    .unwrap_or_default(),
                d.block_bonus,
            )
        }),
        db: d.db as i32,
        db_from: d.db_item.clone(),
        bonus: d.dodge_bonus as i32,
        armor,
    };

    let weapons = ch
        .weapons
        .iter()
        .enumerate()
        .map(|(index, w)| {
            // Предмет -> атаки предмета; без предмета - естественная атака (sid режима).
            let item = w.def.equipment.as_deref().map(sid);
            let own = sid(&w.def.id);
            let attacks: Vec<AttackRow> = out
                .attacks
                .iter()
                .filter(|x| match &item {
                    Some(i) => x.item.as_deref() == Some(i.as_str()),
                    None => x.item.is_none() && x.sid == own,
                })
                .map(|x| {
                    let mut explain = Vec::new();
                    if let (Some(s), Some(l)) = (&x.skill, x.level) {
                        explain.push(format!("{s}: {l}"));
                    }
                    AttackRow {
                        mode: x.name.clone().unwrap_or_else(|| x.sid.clone()),
                        skill: x.skill.clone().unwrap_or_default(),
                        skill_id: x.skill.clone(),
                        via_technique: false,
                        level: x.level.map(|v| v as i32),
                        damage_expr: x.damage_text.clone().unwrap_or_default(),
                        damage: x.damage_text.clone().unwrap_or_default(),
                        damage_type: x.damage_type.clone().unwrap_or_default(),
                        reach: x
                            .reach
                            .iter()
                            .map(i64::to_string)
                            .collect::<Vec<_>>()
                            .join(","),
                        parry: x.parry.map(|v| v as i32),
                        can_parry: x.parry.is_some(),
                        parry_modifier: 0,
                        explain,
                    }
                })
                .collect();
            WeaponRow {
                index,
                id: w.def.id.clone(),
                name: w.def.name.clone(),
                equipment: w.def.equipment.clone(),
                available: !attacks.is_empty(),
                ready: !attacks.is_empty(),
                attacks,
            }
        })
        .collect();

    let mut used = vec![false; out.skills.len()];
    let skills = ch
        .skills
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let l = take(
                &out.skills,
                &mut used,
                &sid(&s.def.id),
                s.specialization.as_deref(),
            );
            skill_row(i, l, s.def.attribute.key())
        })
        .collect();
    let mut used = vec![false; out.techniques.len()];
    let techniques = ch
        .techniques
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let l = take(&out.techniques, &mut used, &sid(&t.def.id), None);
            skill_row(i, l, &t.def.skill)
        })
        .collect();
    let mut used = vec![false; out.spells.len()];
    let spells = ch
        .spells
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let l = take(&out.spells, &mut used, &sid(&s.def.id), None);
            skill_row(i, l, s.def.attribute.key())
        })
        .collect();

    Sheet {
        name: ch.name.clone(),
        attributes,
        secondary,
        basic_lift,
        damage,
        encumbrance,
        weapons,
        defenses,
        points,
        skills,
        techniques,
        spells,
    }
}

/// Проверки листа: правила-числа из БД (`calc::validate`) + проверки приложения.
pub fn issues(ch: &Character, calc: &Result<Calc, String>, sheet: &Sheet) -> Vec<Issue> {
    let mut out = Vec::new();
    let warn = |out: &mut Vec<Issue>, message: String| {
        out.push(Issue {
            severity: "warning",
            message,
        })
    };
    let err = |out: &mut Vec<Issue>, message: String| {
        out.push(Issue {
            severity: "error",
            message,
        })
    };
    match calc {
        Err(e) => err(&mut out, format!("расчёт недоступен: {e}")),
        Ok(c) => out.extend(crate::calc::validate(ch, c)),
    }
    if calc.is_ok() && sheet.points.unspent < 0 {
        warn(
            &mut out,
            format!("бюджет превышен на {} оч.", -sheet.points.unspent),
        );
    }
    if sheet.encumbrance.overloaded {
        warn(
            &mut out,
            format!(
                "несёт {} фунтов - больше предела: персонаж не может двигаться",
                fmt_num(sheet.encumbrance.carried_weight)
            ),
        );
    }
    for t in &ch.traits {
        if t.def.is_leveled() && t.levels < t.def.min_level() {
            err(
                &mut out,
                format!(
                    "черта «{}»: нужно не меньше {} уровней",
                    t.def.name,
                    t.def.min_level()
                ),
            );
        }
        if let Some(max) = t.def.max_levels {
            if t.levels > max {
                err(
                    &mut out,
                    format!("черта «{}»: больше {max} уровней", t.def.name),
                );
            }
        }
    }
    for s in &ch.skills {
        let name = s.display_name();
        if s.points < 0 {
            err(&mut out, format!("навык «{name}»: отрицательные очки"));
        } else if !(s.points <= 2 || s.points % 4 == 0) {
            warn(
                &mut out,
                format!(
                    "навык «{name}»: {} оч. нет в таблице (1, 2, 4, 8, 12...)",
                    s.points
                ),
            );
        }
        if s.def.specialization_required && gurps_core::is_blank(&s.specialization) {
            warn(&mut out, format!("навык «{name}»: нужна специализация"));
        }
    }
    for t in &ch.techniques {
        if t.points < 0 {
            err(
                &mut out,
                format!("приём «{}»: отрицательные очки", t.def.name),
            );
        }
        if !ch.skills.iter().any(|s| s.def.id == t.def.skill) {
            warn(
                &mut out,
                format!("приём «{}»: нужен навык {}", t.def.name, t.def.skill),
            );
        }
    }
    out
}
