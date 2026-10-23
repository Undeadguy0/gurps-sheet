import { describe, expect, it } from 'vitest';
import { COST, MockBackend, localize, outcome } from '../src/mock';
import { matchesLibrary } from '../src/i18n';
import { featureText } from '../src/labels';
import type { LibraryEntry, LocaleInfo } from '../src/types';
import type { SheetView } from '../src/types';

describe('browser preview backend', () => {
  it('acceptance: ST 12 bought + trait +1', () => {
    const m = new MockBackend();
    m.invoke('set_attribute', { key: 'ST', level: 2 });
    const v = m.invoke('add_trait', { id: 'increased-st-example', levels: 1 }) as SheetView;
    const st = v.sheet.attributes[0];
    expect(st.total).toBe(13);
    expect(st.value.base).toBe(10);
    expect(st.value.contributions.map((c) => c.amount)).toEqual([2, 1]);
    expect(v.sheet.points.total).toBe(30);
    expect(v.sheet.secondary[0].total).toBe(13); // HP follows ST
  });
  it('criticals match B348', () => {
    expect(outcome(4, 3)).toBe('critical_success');
    expect(outcome(5, 15)).toBe('critical_success');
    expect(outcome(6, 15)).toBe('success');
    expect(outcome(17, 15)).toBe('critical_failure');
    expect(outcome(17, 16)).toBe('failure');
    expect(outcome(15, 5)).toBe('critical_failure');
  });
  it('explanation: plugin default, user override, reset', () => {
    const m = new MockBackend();
    let v = m.invoke('add_trait', { id: 'combat-reflexes' }) as SheetView;
    const def = v.traits[0].explanation;
    expect(def).toContain('Быстрая реакция');
    expect(v.traits[0].custom_explanation).toBe(false);
    v = m.invoke('set_trait_explanation', { index: 0, text: 'Моё **пояснение**' }) as SheetView;
    expect(v.traits[0].explanation).toBe('Моё **пояснение**');
    expect(v.traits[0].default_explanation).toBe(def);
    expect(v.traits[0].custom_explanation).toBe(true);
    v = m.invoke('set_trait_explanation', { index: 0, text: null }) as SheetView;
    expect(v.traits[0].explanation).toBe(def);
    expect(v.traits[0].custom_explanation).toBe(false);
  });
  it('localize falls back between locales', () => {
    expect(localize({ ru: 'р', en: 'e' }, 'en')).toBe('e');
    expect(localize({ ru: 'р' }, 'en')).toBe('р');
    expect(localize({ de: 'd', en: ' ' }, 'ru')).toBe('d');
    expect(localize('plain')).toBe('plain');
    expect(localize('  ')).toBeNull();
    expect(localize(undefined)).toBeNull();
  });
  it.each([
    ['ST', 10, 1], ['DX', 20, 1], ['IQ', 20, 1], ['HT', 10, 1], ['HP', 2, 1], ['Will', 5, 1],
    ['Per', 5, 1], ['FP', 3, 1], ['BasicSpeed', 5, 0.25], ['BasicMove', 5, 1],
  ] as const)('attribute cost %s = %i per %f', (key, cost, step) => {
    expect(COST[key]).toBe(cost);
    for (const lv of [1, 3, -2]) {
      const m = new MockBackend();
      const v = m.invoke('set_attribute', { key, level: lv }) as SheetView;
      const r = [...v.sheet.attributes, ...v.sheet.secondary].find((x) => x.key === key)!;
      expect(r.cost_per_step).toBe(cost);
      expect(r.step).toBe(step);
      expect(r.points).toBe(cost * lv);
      expect(v.sheet.points.total).toBe(cost * lv);
      expect(r.value.contributions[0].amount).toBe(lv * step);
    }
  });
  it('localization: translation (original), original-only, search both', () => {
    const m = new MockBackend();
    expect((m.invoke('get_locales', {}) as LocaleInfo).available).toEqual(['en', 'ru']);
    let v = m.invoke('add_trait', { id: 'lifting-st', levels: 2 }) as SheetView;
    expect(v.traits[0].name).toBe('Подъёмная сила 2');
    expect(v.traits[0].original_name).toBe('Lifting ST 2');
    expect(v.traits[0].explanation).toContain('только для подъёма');
    const lib = m.invoke('list_library', {}) as LibraryEntry[];
    const cr = lib.find((e) => e.id === 'combat-reflexes')!;
    expect(cr.translated_name).toBe('Боевые рефлексы');
    expect(matchesLibrary(cr, 'рефлекс')).toBe(true);
    expect(matchesLibrary(cr, 'reflexes')).toBe(true);
    expect(matchesLibrary(cr, 'nope')).toBe(false);
    v = m.invoke('set_locale', { locale: 'ru', originalOnly: true }) as SheetView;
    expect(v.traits[0].name).toBe('Lifting ST 2');
    expect(v.traits[0].original_name).toBeNull();
    expect(v.traits[0].explanation).toContain('only for lifting');
    expect((m.invoke('list_library', {}) as LibraryEntry[]).every((e) => !e.translated_name)).toBe(true);
    v = m.invoke('set_locale', { locale: 'en', originalOnly: false }) as SheetView;
    expect(v.traits[0].name).toBe('Lifting ST 2');
  });
  it('featureText is readable', () => {
    expect(featureText({ type: 'attribute_bonus', target: 'ST', amount: 1, per_level: true })).toBe('ST — Сила: +1 за уровень');
    expect(featureText({ type: 'lifting_st_bonus', amount: 2 })).toContain('базовый груз): +2');
    expect(featureText({ type: 'striking_st_bonus', amount: -1, per_level: true })).toContain('-1 за уровень');
  });
});
