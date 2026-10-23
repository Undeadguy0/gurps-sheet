// TS mirror of gurps_rules::encumbrance (used by the browser mock and tests).
// The Rust crate is the source of truth.
import type { Encumbrance, EquipmentItem } from './types';

export const BL_MULTIPLES = [1, 2, 3, 6, 10];
export const MOVE_MULTIPLIERS = [1, 0.8, 0.6, 0.4, 0.2];
export const LEVEL_NAMES = ['None', 'Light', 'Medium', 'Heavy', 'Extra-Heavy'];
export const LEVEL_LABEL_RU = ['Нет', 'Лёгкая', 'Средняя', 'Тяжёлая', 'Очень тяжёлая'];
const EPS = 1e-9;

export function levelFor(weight: number, bl: number): [number, boolean] {
  for (let i = 0; i < 5; i++) if (weight <= bl * BL_MULTIPLES[i] + EPS) return [i, false];
  return [4, true];
}

export function moveFor(baseMove: number, level: number): number {
  const m = Math.floor(baseMove * MOVE_MULTIPLIERS[level] + EPS);
  return baseMove >= 1 ? Math.max(m, 1) : Math.max(m, 0);
}

export const totalWeight = (e: EquipmentItem): number => e.weight * e.quantity + (e.contents ?? []).reduce((a, c) => a + totalWeight(c), 0);
export const totalCost = (e: EquipmentItem): number => e.cost * e.quantity + (e.contents ?? []).reduce((a, c) => a + totalCost(c), 0);
export const carriedWeight = (e: EquipmentItem): number =>
  e.carried === false ? 0 : e.weight * e.quantity + (e.contents ?? []).reduce((a, c) => a + carriedWeight(c), 0);
const carriedCost = (e: EquipmentItem): number =>
  e.carried === false ? 0 : e.cost * e.quantity + (e.contents ?? []).reduce((a, c) => a + carriedCost(c), 0);

const fmt = (v: number) => (Number.isInteger(v) ? String(v) : String(Math.round(v * 100) / 100));

export function computeEncumbrance(items: EquipmentItem[], bl: number, baseMove: number, speed: number): Encumbrance {
  const sum = (f: (e: EquipmentItem) => number) => items.reduce((a, e) => a + f(e), 0);
  const carried = sum(carriedWeight);
  const [level, overloaded] = levelFor(carried, bl);
  const baseDodge = Math.floor(speed + EPS) + 3;
  const move = overloaded ? 0 : moveFor(baseMove, level);
  const explain = [`carried ${fmt(carried)} lb, BL ${fmt(bl)} lb`];
  if (overloaded) explain.push(`> 10×BL (${fmt(bl * 10)} lb): too heavy to move`);
  else {
    explain.push(`≤ ${BL_MULTIPLES[level]}×BL (${fmt(bl * BL_MULTIPLES[level])} lb) -> ${LEVEL_NAMES[level]} (${level})`);
    explain.push(`Move floor(${fmt(baseMove)} × ${MOVE_MULTIPLIERS[level]}) = ${move}`);
  }
  explain.push(`Dodge floor(${fmt(speed)}) + 3 ${level > 0 ? `- ${level}` : ''} = ${baseDodge - level}`);
  return {
    level, name: LEVEL_NAMES[level], carried_weight: carried, total_weight: sum(totalWeight), total_cost: sum(totalCost),
    carried_cost: sum(carriedCost), basic_lift: bl, base_move: baseMove, move_value: move, dodge_penalty: -level,
    base_dodge: baseDodge, dodge: baseDodge - level, overloaded,
    table: [0, 1, 2, 3, 4].map((l) => ({ level: l, name: LEVEL_NAMES[l], max_weight: bl * BL_MULTIPLES[l], move_value: moveFor(baseMove, l), dodge: baseDodge - l })),
    explain,
  };
}

/** "46 lb" / "5,5 lb" — weights are kept in pounds as in the data. */
export const lb = (v: number) => `${fmt(v)} lb`;
export const money = (v: number) => `$${fmt(v)}`;
