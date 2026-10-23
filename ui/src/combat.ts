// TS mirror of gurps_rules::combat and gurps_core::damage (browser mock + tests).
// The Rust crates are the source of truth; explanation lines match them word for word.
import type { ArmorRow, AttackRow, Defense, Defenses, WeaponDef, WeaponRow } from './types';

export interface Dice { dice: number; adds: number }
const d = (dice: number, adds: number): Dice => ({ dice, adds });
/** (thrust, swing) for ST 1..40, same as gurps_core::damage::TABLE. */
const TABLE: [Dice, Dice][] = [
  [d(1, -6), d(1, -5)], [d(1, -6), d(1, -5)], [d(1, -5), d(1, -4)], [d(1, -5), d(1, -4)], [d(1, -4), d(1, -3)],
  [d(1, -4), d(1, -3)], [d(1, -3), d(1, -2)], [d(1, -3), d(1, -2)], [d(1, -2), d(1, -1)], [d(1, -2), d(1, 0)],
  [d(1, -1), d(1, 1)], [d(1, -1), d(1, 2)], [d(1, 0), d(2, -1)], [d(1, 0), d(2, 0)], [d(1, 1), d(2, 1)],
  [d(1, 1), d(2, 2)], [d(1, 2), d(3, -1)], [d(1, 2), d(3, 0)], [d(2, -1), d(3, 1)], [d(2, -1), d(3, 2)],
  [d(2, 0), d(4, -1)], [d(2, 0), d(4, 0)], [d(2, 1), d(4, 1)], [d(2, 1), d(4, 2)], [d(2, 2), d(5, -1)],
  [d(2, 2), d(5, 0)], [d(3, -1), d(5, 1)], [d(3, -1), d(5, 1)], [d(3, 0), d(5, 2)], [d(3, 0), d(5, 2)],
  [d(3, 1), d(6, -1)], [d(3, 1), d(6, -1)], [d(3, 2), d(6, 0)], [d(3, 2), d(6, 0)], [d(4, -1), d(6, 1)],
  [d(4, -1), d(6, 1)], [d(4, 0), d(6, 2)], [d(4, 0), d(6, 2)], [d(4, 1), d(7, -1)], [d(4, 1), d(7, -1)],
];
export const basicDamage = (st: number): [Dice, Dice] => TABLE[Math.min(40, Math.max(1, Math.floor(st))) - 1];
export const diceStr = (x: Dice) => (x.adds === 0 ? `${x.dice}d` : x.adds > 0 ? `${x.dice}d+${x.adds}` : `${x.dice}d${x.adds}`);

export type DamageExpr = { kind: 'thr' | 'sw'; adds: number } | { kind: 'fixed'; dice: Dice };
export function parseDamage(src: string): DamageExpr | null {
  const s = src.replace(/\s+/g, '').toLowerCase();
  const adds = (r: string) => (r === '' ? 0 : /^[+-]?\d+$/.test(r) ? parseInt(r, 10) : null);
  for (const k of ['thr', 'sw'] as const) if (s.startsWith(k)) { const a = adds(s.slice(k.length)); return a === null ? null : { kind: k, adds: a }; }
  const m = /^(\d+)d(.*)$/.exec(s); if (!m) return null;
  const a = adds(m[2]); return a === null ? null : { kind: 'fixed', dice: d(+m[1], a) };
}
export function parseDb(stats: string): number | null { const m = /DB\s*(\d+)/.exec(stats); return m ? +m[1] : null; }
export function parseDr(stats: string): [string, number] | null {
  const i = stats.indexOf('DR'); if (i < 0) return null;
  const text = /^[^\s,;]*/.exec(stats.slice(i + 2).trimStart())![0]; const n = /^\d+/.exec(text);
  return n ? [text, +n[0]] : null;
}

const sg = (v: number) => (v >= 0 ? `+${v}` : String(v));

export interface SkillLv { id: string; name: string; attribute: string; level: number | null }
export interface TechLv { id: string; name: string; level: number | null }
export interface Owned { id: string; name: string; carried: boolean; stats?: string | null }
export interface CombatCtx {
  skills: SkillLv[]; techniques: TechLv[]; attr: (k: string) => number; thrust: Dice; swing: Dice;
  /** Flattened equipment with effective carried flag. */
  owned: Owned[];
  /** [dodge, parry, block] bonus and trait names. */
  bonus: [number, number, number]; bonusFrom: string[];
}

const best = <T extends { id: string; level: number | null }>(rows: T[], id: string) =>
  rows.filter((r) => r.id === id && r.level !== null).reduce<T | null>((b, r) => (!b || r.level! > b.level! ? r : b), null);

export function computeWeapons(weapons: WeaponDef[], c: CombatCtx): WeaponRow[] {
  const a = (k: string) => Math.floor(c.attr(k));
  return weapons.map((w, index) => {
    const mine = w.equipment ? c.owned.filter((o) => o.id === w.equipment) : null;
    const available = !mine || mine.length > 0; const ready = !mine || mine.some((o) => o.carried);
    const attacks = w.attacks.map((at): AttackRow => {
      const skillId = at.skill ?? w.skill; const skill = best(c.skills, skillId);
      const tech = at.technique ? best(c.techniques, at.technique) : null;
      const explain: string[] = [];
      let level: number | null; let used: string; let usedId: string | null; let viaTech = false;
      if (tech) { explain.push(`technique ${tech.name} ${tech.level}`); [level, used, usedId, viaTech] = [tech.level, tech.name, tech.id, true]; }
      else if (skill) {
        const m = at.skill_modifier ?? 0; const l = skill.level! + m;
        explain.push(m ? `skill ${skill.name}${sg(m)} = ${l}` : `skill ${skill.name} ${l}`); [level, used, usedId] = [l, skill.name, skill.id];
      } else {
        let b: [number, string, string | null] | null = null;
        for (const df of at.defaults ?? []) {
          const x: [number, string, string | null] | null = 'attribute' in df ? [a(df.attribute) + df.modifier, `${df.attribute}${sg(df.modifier)}`, null]
            : ((r) => (r ? [r.level! + df.modifier, `${r.name}${sg(df.modifier)}`, df.skill] as [number, string, string] : null))(best(c.skills, df.skill));
          if (x && (!b || x[0] > b[0])) b = x;
        }
        if (b) { explain.push(`default ${b[1]} = ${b[0]}`); [level, used, usedId] = b; }
        else { explain.push(`no skill ${skillId} and no default`); [level, used, usedId] = [null, skillId, skillId]; }
      }
      const e = parseDamage(at.damage);
      let dice: Dice; let base: string;
      if (!e) [dice, base] = [d(0, 0), 'unparsed'];
      else if (e.kind === 'thr') [dice, base] = [d(c.thrust.dice, c.thrust.adds + e.adds), `thr ${diceStr(c.thrust)}`];
      else if (e.kind === 'sw') [dice, base] = [d(c.swing.dice, c.swing.adds + e.adds), `sw ${diceStr(c.swing)}`];
      else if (e.kind === 'fixed') [dice, base] = [{ ...e.dice }, 'fixed'];
      else [dice, base] = [d(0, 0), 'unparsed'];
      explain.push(`damage ${at.damage} = ${base} -> ${diceStr(dice)}`);
      const sb = at.skill_damage_bonus;
      if (sb && skill) {
        const rel = skill.level! - a(skill.attribute);
        if (rel >= sb.min_relative) {
          dice = d(dice.dice, dice.adds + sb.per_die * dice.dice);
          explain.push(`${skill.name} at ${skill.attribute}${sg(rel)} (>= ${sg(sb.min_relative)}): ${sg(sb.per_die)} per die -> ${diceStr(dice)}`);
        } else explain.push(`no ${skill.name} damage bonus: ${skill.attribute}${sg(rel)} < ${skill.attribute}${sg(sb.min_relative)}`);
      }
      const pm = at.parry ?? null; let parry: number | null = null;
      if (pm === null) explain.push('cannot parry');
      else if (level !== null) {
        parry = Math.floor(level / 2) + 3 + pm + c.bonus[1];
        explain.push(`Parry floor(${level}/2) + 3${pm ? ` ${sg(pm)}` : ''}${c.bonus[1] ? ` ${sg(c.bonus[1])}` : ''} = ${parry}`);
      }
      return { mode: at.mode, skill: used, skill_id: usedId, via_technique: viaTech, level, damage_expr: at.damage, damage: diceStr(dice),
        damage_type: at.damage_type, reach: at.reach ?? '', parry, can_parry: pm !== null, parry_modifier: pm ?? 0, explain };
    });
    return { index, id: w.id, name: w.name, equipment: w.equipment ?? null, available, ready, attacks };
  });
}

export function computeDefenses(weapons: WeaponRow[], c: CombatCtx, enc: { base_dodge: number; dodge_penalty: number; name: string }): Defenses {
  const from = c.bonusFrom.length ? ` (${c.bonusFrom.join(', ')})` : '';
  const bt = (b: number) => (b ? ` ${sg(b)}${from}` : '');
  const dodge = enc.base_dodge + c.bonus[0] + enc.dodge_penalty;
  const dex = [`floor(Basic Speed) + 3 = ${enc.base_dodge}${bt(c.bonus[0])}`];
  if (enc.dodge_penalty) dex.push(`encumbrance ${enc.dodge_penalty} (${enc.name})`);
  dex.push(`= ${dodge}`);
  const armed = weapons.some((w) => w.ready && w.equipment && w.attacks.some((x) => x.parry != null));
  let parry: Defense | null = null;
  for (const w of weapons) if (w.ready && (!armed || w.equipment)) for (const x of w.attacks)
    if (x.parry != null && (!parry || x.parry > parry.value))
      parry = { value: x.parry, from: w.name, explain: x.explain.filter((l) => /^(Parry|skill|default|technique)/.test(l)) };
  let db = 0; let dbFrom: string | null = null; const armor: ArmorRow[] = [];
  for (const o of c.owned) {
    if (!o.carried || !o.stats) continue;
    const v = parseDb(o.stats); if (v !== null && v > db) { db = v; dbFrom = o.name; }
    const r = parseDr(o.stats); if (r) armor.push({ item: o.name, dr: r[0], value: r[1] });
  }
  let block: Defense | null = null;
  const sh = db > 0 ? best(c.skills, 'shield') : null;
  if (sh) {
    const v = Math.floor(sh.level! / 2) + 3 + c.bonus[2];
    block = { value: v, from: sh.name, explain: [`skill ${sh.name} ${sh.level}`, `Block floor(${sh.level}/2) + 3${bt(c.bonus[2])} = ${v}`] };
  }
  return { dodge: { value: dodge, from: 'Basic Speed', explain: dex }, parry, block, db, db_from: dbFrom, bonus: Math.max(...c.bonus), armor };
}
