// Skill/technique level maths. Mirror of crates/gurps-rules/src/skills.rs (source of truth),
// used by the browser preview backend and for UI hints (cost of the next level).
import type { Difficulty, SkillDef, TechniqueDef } from './mockTypes';

export const DIFF_OFFSET: Record<Difficulty, number> = { E: 0, A: -1, H: -2, VH: -3 };
export const DIFF_LABEL: Record<string, string> = { E: 'Лёгкий', A: 'Средний', H: 'Трудный', VH: 'Очень трудный' };

/** 1 pt -> 0, 2 -> 1, 4 -> 2, 8 -> 3, then +1 per 4 points; null for 0 points. */
export function levelSteps(points: number): number | null {
  if (points <= 0) return null;
  if (points === 1) return 0;
  if (points < 4) return 1;
  if (points < 8) return 2;
  return 3 + Math.floor((points - 8) / 4);
}
export const relativeLevel = (d: Difficulty, points: number) => {
  const s = levelSteps(points); return s === null ? null : s + DIFF_OFFSET[d];
};
export function pointsForSteps(steps: number): number {
  return steps <= 0 ? 1 : steps === 1 ? 2 : steps === 2 ? 4 : 8 + 4 * (steps - 3);
}
/** Next / previous point value on the cost table (for the +/- buttons). */
export function nextPoints(points: number): number { const s = levelSteps(points); return s === null ? 1 : pointsForSteps(s + 1); }
export function prevPoints(points: number): number { const s = levelSteps(points); return s === null || s === 0 ? 0 : pointsForSteps(s - 1); }
export const techniqueLevels = (d: Difficulty, points: number) => (d === 'H' || d === 'VH' ? Math.max(points - 1, 0) : Math.max(points, 0));
/** Technique points for the next level above the current points. */
export const nextTechniquePoints = (d: Difficulty, points: number) => ((d === 'H' || d === 'VH') && points === 0 ? 2 : points + 1);
export const prevTechniquePoints = (d: Difficulty, points: number) => ((d === 'H' || d === 'VH') && points <= 2 ? 0 : Math.max(points - 1, 0));

export const signedInt = (v: number) => (v >= 0 ? `+${v}` : String(v));

export interface SkillIn { def: SkillDef; points: number; specialization?: string | null }
export interface TechIn { def: TechniqueDef; points: number }
export interface SkillOut {
  name: string; level: number | null; relative: string | null; from_default: boolean; bonus: number; explain: string[];
}
export interface TechOut { name: string; level: number | null; max_level: number | null; parent: number | null; explain: string[] }

export const displayName = (name: string, spec?: string | null) => (spec && spec.trim() ? `${name} (${spec.trim()})` : name);

/** `attr` returns attribute totals; `talents` maps skill id -> [trait name, bonus][]. */
export function computeSkills(skills: SkillIn[], attr: (k: string) => number, talents: Record<string, [string, number][]>): SkillOut[] {
  const a = (k: string) => Math.floor(attr(k));
  const bought: Record<string, [number, string]> = {};
  for (const s of skills) {
    const rel = relativeLevel(s.def.difficulty, s.points); if (rel === null) continue;
    const lv = a(s.def.attribute) + rel;
    if (!bought[s.def.id] || lv > bought[s.def.id][0]) bought[s.def.id] = [lv, displayName(s.def.name, s.specialization)];
  }
  return skills.map((s) => {
    const d = s.def.difficulty; const k = s.def.attribute; const base = a(k);
    const explain = [`${k} ${base}`];
    const rel = relativeLevel(d, s.points);
    const purchased = rel === null ? null : base + rel;
    if (rel !== null) explain.push(`${enPts(s.points)}, ${k}/${d} -> ${k}${signedInt(rel)} = ${base + rel}`);
    let best: [number, string] | null = null;
    for (const df of s.def.defaults ?? []) {
      const c: [number, string] | null = 'attribute' in df ? [a(df.attribute) + df.modifier, `${df.attribute}${signedInt(df.modifier)}`]
        : df.skill !== s.def.id && bought[df.skill] ? [bought[df.skill][0] + df.modifier, `${bought[df.skill][1]}${signedInt(df.modifier)}`] : null;
      if (c && (!best || c[0] > best[0])) best = c;
    }
    if (best) explain.push(`default ${best[1]} = ${best[0]}`);
    let raw: number | null = purchased; let fromDefault = false;
    if (best && (purchased === null || best[0] > purchased)) { raw = best[0]; fromDefault = true; }
    let bonus = 0;
    if (raw !== null) for (const [n, b] of talents[s.def.id] ?? []) { bonus += b; explain.push(`${signedInt(b)} from trait ${n}`); }
    const level = raw === null ? null : raw + bonus;
    explain.push(level === null ? 'no points and no default' : `= ${level}`);
    return { name: displayName(s.def.name, s.specialization), level, relative: level === null ? null : `${k}${signedInt(level - base)}`, from_default: fromDefault, bonus, explain };
  });
}

export function computeTechniques(techs: TechIn[], skills: SkillIn[], rows: SkillOut[]): TechOut[] {
  return techs.map((t) => {
    let pi: number | null = null;
    skills.forEach((s, i) => { if (s.def.id === t.def.skill && rows[i].level !== null && (pi === null || rows[i].level! > rows[pi].level!)) pi = i; });
    const pname = pi !== null ? rows[pi].name : skills.find((s) => s.def.id === t.def.skill)?.def.name ?? t.def.skill;
    const name = `${t.def.name} (${pname})`;
    if (pi === null) return { name, level: null, max_level: null, parent: null, explain: [`needs skill ${pname}`] };
    const pl = rows[pi].level!; const def = pl + t.def.modifier; const cap = pl + t.def.max; const up = techniqueLevels(t.def.difficulty, t.points);
    const explain = [`default ${pname}${signedInt(t.def.modifier)} = ${def}`];
    if (up > 0) explain.push(`${enPts(t.points)} (${t.def.difficulty}) -> +${up}`);
    if (def + up > cap) explain.push(`capped at ${pname}${signedInt(t.def.max)} = ${cap}`);
    const level = Math.min(def + up, cap); explain.push(`= ${level}`);
    return { name, level, max_level: cap, parent: pl, explain };
  });
}

export const enPts = (n: number) => `${n} ${Math.abs(n) === 1 ? 'pt' : 'pts'}`;
