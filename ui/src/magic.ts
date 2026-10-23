// Spell levels and trait-modifier costs. Mirror of crates/gurps-rules/src/spells.rs,
// crates/gurps-rules/src/modifiers.rs and gurps_core::modifier (source of truth).
import type { Difficulty, NegativeRounding, SpellDef } from './types';
import { enPts, relativeLevel, signedInt } from './skills';

/** Limitations cannot reduce a cost by more than 80%. */
export const MIN_TOTAL_PERCENT = -80;
const pct = (v: number) => (v >= 0 ? `+${v}%` : `${v}%`);
const num = (v: number) => (Number.isInteger(v) ? String(v) : String(+v.toFixed(2)));

/** base × (100% + total%), total floored at -80%, rounded up (towards +∞: -7.5 -> -7, GM-confirmed);
 * `by_magnitude` rounds negative fractions away from zero (-7.5 -> -8). */
export function modifiedCost(base: number, totalPercent: number, r: NegativeRounding = 'toward_positive'): number {
  const x = (base * (100 + Math.max(totalPercent, MIN_TOTAL_PERCENT))) / 100;
  return x < 0 && r === 'by_magnitude' ? Math.floor(x) : Math.ceil(x);
}

export function traitCostExplain(base: number, mods: { name: string; cost_percent: number }[], r: NegativeRounding = 'toward_positive'): string[] {
  if (!mods.length) return [];
  const total = mods.reduce((a, m) => a + m.cost_percent, 0);
  const out = [`base cost ${base}`, ...mods.map((m) => `${pct(m.cost_percent)} ${m.name}`)];
  out.push(total < MIN_TOTAL_PERCENT ? `total ${pct(total)} floored at ${pct(MIN_TOTAL_PERCENT)}` : `total ${pct(total)}`);
  const eff = 100 + Math.max(total, MIN_TOTAL_PERCENT);
  const exact = (base * eff) / 100;
  const how = exact < 0 && r === 'by_magnitude' && !Number.isInteger(exact) ? 'rounded away from zero' : 'rounded up';
  out.push(`${base} × ${eff}% = ${num(exact)} -> ${modifiedCost(base, total, r)} (${how})`);
  return out;
}

export interface SpellIn { def: SpellDef; points: number }
export interface SpellOut { level: number | null; relative: string | null; bonus: number; explain: string[] }
/** `bonuses`: [trait display name, bonus, colleges (empty = all)]. */
export function computeSpells(spells: SpellIn[], attr: (k: string) => number, bonuses: [string, number, string[]][]): SpellOut[] {
  return spells.map((s) => {
    const a = s.def.attribute ?? 'IQ'; const d: Difficulty = s.def.difficulty ?? 'H';
    const base = Math.floor(attr(a)); const explain = [`${a} ${base}`];
    const rel = relativeLevel(d, s.points);
    if (rel === null) return { level: null, relative: null, bonus: 0, explain: [...explain, 'spells have no default: buy at least 1 pt'] };
    explain.push(`${enPts(s.points)}, ${a}/${d} -> ${a}${signedInt(rel)} = ${base + rel}`);
    let bonus = 0;
    for (const [name, b, colleges] of bonuses) {
      if (b !== 0 && (!colleges.length || colleges.some((c) => (s.def.college ?? []).includes(c)))) { bonus += b; explain.push(`${signedInt(b)} from trait ${name}`); }
    }
    const level = base + rel + bonus; explain.push(`= ${level}`);
    return { level, relative: `${a}${signedInt(level - base)}`, bonus, explain };
  });
}
