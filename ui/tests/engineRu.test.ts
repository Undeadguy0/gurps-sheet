import { describe, expect, it } from 'vitest';
import { engineRu, namesFrom } from '../src/engineRu';
import lines from './engine-lines.json';
import { MockBackend } from '../src/mock';
import type { SheetView } from '../src/types';

// Lines that are language-neutral formulas and may stay as they are.
const NEUTRAL = /^(= .*|(ST|DX|IQ|HT|Per|Will) -?\d+)$/;
// English words allowed in the output: formula names and attribute abbreviations.
const ALLOWED = new Set(['floor', 'thr', 'sw', 'cut', 'imp', 'cr', 'Per', 'Will']);

const RU_NAMES = new Map([
  ['Combat Reflexes', 'Боевые рефлексы'], ['Brawling', 'Драка'], ['Broadsword', 'Палаш'], ['Knife', 'Нож'], ['Shield', 'Щит'],
  ['Kicking (Brawling)', 'Пинок (Драка)'], ['Lifting ST 2', 'Подъёмная ST 2'], ['Striking ST 2', 'Ударная ST 2'],
  ['Increased ST (example) 1', 'Повышенная ST (пример) 1'], ['Outdoorsman 1', 'Природник 1'],
  ['Magery 2', 'Магические способности 2'], ['Area Effect', 'Действие по площади'],
]);
const n = (s: string) => RU_NAMES.get(s) ?? s;

describe('engine lines -> ru (snapshot of real gurps-rules output, tests/engine-lines.json)', () => {
  it('every non-formula line is translated and has no English words left', () => {
    const bad: string[] = [];
    for (const l of lines as string[]) {
      if (NEUTRAL.test(l)) continue;
      const r = engineRu(l, n);
      const words = (r.match(/[A-Za-z]{3,}/g) ?? []).filter((w) => !ALLOWED.has(w));
      if (r === l || words.length) bad.push(`${l} => ${r}`);
    }
    expect(bad).toEqual([]);
  });
  it('known lines', () => {
    expect(engineRu('floor(Basic Speed) + 3 = 9 +1 (Combat Reflexes)', n)).toBe('floor(Скорость) + 3 = 9 +1 (Боевые рефлексы)');
    expect(engineRu('Block floor(14/2) + 3 +1 (Combat Reflexes) = 11', n)).toBe('Блок floor(14/2) + 3 +1 (Боевые рефлексы) = 11');
    expect(engineRu('Parry floor(15/2) + 3 -1 = 9')).toBe('Парирование floor(15/2) + 3 -1 = 9');
    expect(engineRu('damage sw-3 = sw 2d-1 -> 2d-4')).toBe('урон sw-3 = замах 2d-1 → 2d-4');
    expect(engineRu('no Brawling damage bonus: DX+0 < DX+2', n)).toBe('нет бонуса к урону от «Драка»: DX+0 < DX+2');
    expect(engineRu('Brawling at DX+2 (>= +2): +1 per die -> 1d', n)).toBe('«Драка» на уровне DX+2 (≥ +2): +1 за кубик → 1d');
    expect(engineRu('technique Kicking (Brawling) 11', n)).toBe('приём «Пинок (Драка)» 11');
    expect(engineRu('skill Brawling-2 = 10', n)).toBe('навык «Драка»-2 = 10');
    expect(engineRu('default Brawling-2 = 10', n)).toBe('по умолчанию «Драка»-2 = 10');
    expect(engineRu('encumbrance -1 (Light)')).toBe('нагрузка -1 (Лёгкая)');
    expect(engineRu('carried 5.5 lb, BL 24 lb')).toBe('при себе 5.5 фунта, базовый груз 24 фунта');
  });
  it('plural agreement in point counts (1, 2, 5, 11, 21, 22, 25)', () => {
    expect([1, 2, 5, 11, 21, 22, 25].map((k) => engineRu(`purchased (${k} ${k === 1 ? 'pt' : 'pts'})`)))
      .toEqual(['куплено (1 очко)', 'куплено (2 очка)', 'куплено (5 очков)', 'куплено (11 очков)', 'куплено (21 очко)', 'куплено (22 очка)', 'куплено (25 очков)']);
  });
  it('validation messages (gurps_rules::validate formats)', () => {
    expect(engineRu('over budget by 21 pts')).toBe('Бюджет превышен на 21 очко');
    expect(engineRu("skill 'Survival' needs a specialization", (s) => (s === 'Survival' ? 'Выживание' : s))).toBe('навыку «Выживание» нужна специализация');
    expect(engineRu("skill 'Knife': 3 pts is not on the cost table (1, 2, 4, 8, 12...); extra points are wasted"))
      .toBe('навык «Knife»: 3 очка нет в таблице стоимости (1, 2, 4, 8, 12…); лишние очки пропадают');
    expect(engineRu("trait 'X' needs at least 1 level(s)")).toBe('черте «X» нужно не меньше 1 уровня');
    expect(engineRu("trait 'X' needs at least 2 level(s)")).toBe('черте «X» нужно не меньше 2 уровней');
    expect(engineRu("technique 'Kicking' needs skill 'brawling'")).toBe('приёму «Kicking» нужен навык «brawling»');
    expect(engineRu('carried 460 lb is more than 10×BL (450 lb): the character cannot move'))
      .toBe('при себе 460 фунтов — больше 10×БГ (450 фунтов): персонаж не может двигаться');
    expect(engineRu('something new')).toBe('something new');
  });
  it('mock backend lines are covered too', () => {
    const m = new MockBackend();
    m.invoke('add_trait', { id: 'combat-reflexes' });
    m.invoke('add_skill', { id: 'brawling', points: 1 });
    m.invoke('add_technique', { id: 'kicking', points: 2 });
    const v = m.invoke('add_equipment', { id: 'small-knife' }) as SheetView;
    const names = namesFrom([v.traits, v.skills, v.techniques, v.weapons, v.equipment]);
    const all = [...v.sheet.attributes, ...v.sheet.secondary, v.sheet.basic_lift].flatMap((r) => r.explain)
      .concat(v.skills.flatMap((s) => s.explain), v.techniques.flatMap((t) => t.explain), v.sheet.encumbrance!.explain,
        v.sheet.weapons!.flatMap((w) => w.attacks.flatMap((a) => a.explain)), v.sheet.defenses!.dodge.explain, v.issues.map((i) => i.message));
    const left = all.filter((l) => !NEUTRAL.test(l))
      .map((l) => engineRu(l, (s) => names.get(s) ?? s))
      .filter((r) => (r.match(/[A-Za-z]{3,}/g) ?? []).some((w) => !ALLOWED.has(w)));
    expect(left).toEqual([]);
  });
});
