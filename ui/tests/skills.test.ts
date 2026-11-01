import { describe, expect, it } from 'vitest';
import { MockBackend } from '../src/mock';
import { levelSteps, nextPoints, prevPoints, relativeLevel, techniqueLevels, nextTechniquePoints } from '../src/skills';
import { matchesLibrary } from '../src/i18n';
import type { SheetView, SkillLibrary } from '../src/mockTypes';

describe('skill maths (mirror of gurps-rules)', () => {
  it('cost table 1/2/4/8 then +4', () => {
    expect([1, 2, 3, 4, 8, 12, 16].map(levelSteps)).toEqual([0, 1, 1, 2, 3, 4, 5]);
    expect(levelSteps(0)).toBeNull();
    expect(relativeLevel('A', 2)).toBe(0);
    expect(relativeLevel('H', 4)).toBe(0);
    expect(relativeLevel('VH', 1)).toBe(-3);
    expect([0, 1, 2, 4, 8].map(nextPoints)).toEqual([1, 2, 4, 8, 12]);
    expect([12, 4, 1, 0].map(prevPoints)).toEqual([8, 2, 0, 0]);
  });
  it('technique costs', () => {
    expect(techniqueLevels('H', 2)).toBe(1);
    expect(techniqueLevels('A', 2)).toBe(2);
    expect(nextTechniquePoints('H', 0)).toBe(2);
  });
});

describe('mock backend skills (GM warrior data)', () => {
  function warrior() {
    const m = new MockBackend();
    m.invoke('set_attribute', { key: 'DX', level: 2 });
    m.invoke('add_trait', { id: 'outdoorsman', levels: 1 });
    m.invoke('add_skill', { id: 'broadsword', points: 8 });
    m.invoke('add_skill', { id: 'brawling', points: 1 });
    m.invoke('add_skill', { id: 'survival', specialization: 'Woodlands', points: 2 });
    m.invoke('add_skill', { id: 'stealth', points: 0 });
    return m.invoke('add_technique', { id: 'kicking', points: 2 }) as SheetView;
  }
  it('levels, talent, default, technique', () => {
    const v = warrior();
    expect(v.skills.map((s) => s.level)).toEqual([14, 12, 11, 7]);
    expect(v.skills[2].name).toBe('Выживание (Woodlands)');
    expect(v.skills[2].original_name).toBe('Survival (Woodlands)');
    expect(v.skills[2].bonus).toBe(1);
    expect(v.skills[3].from_default).toBe(true);
    expect(v.techniques[0].level).toBe(11);
    expect(v.techniques[0].name).toBe('Пинок (Драка)');
    expect(v.techniques[0].max_level).toBe(12);
    expect(v.sheet.points.skills).toBe(13);
  });
  it('technique cap and explanations', () => {
    const m = new MockBackend();
    m.invoke('add_skill', { id: 'brawling', points: 1 });
    let v = m.invoke('add_technique', { id: 'kicking', points: 6 }) as SheetView;
    expect(v.techniques[0].level).toBe(10);
    expect(v.techniques[0].explain.some((l) => l.startsWith('capped'))).toBe(true);
    v = m.invoke('set_skill_explanation', { index: 0, text: 'моё' }) as SheetView;
    expect(v.skills[0].custom_explanation).toBe(true);
    v = m.invoke('set_skill_explanation', { index: 0, text: ' ' }) as SheetView;
    expect(v.skills[0].explanation).toContain('Грубый');
    v = m.invoke('remove_skill', { index: 0 }) as SheetView;
    expect(v.techniques[0].level).toBeNull();
  });
  it('library: bilingual search, original-only mode', () => {
    const m = new MockBackend();
    const lib = m.invoke('list_skill_library', {}) as SkillLibrary;
    expect(lib.skills.length).toBeGreaterThanOrEqual(10);
    expect(lib.skills.filter((e) => matchesLibrary(e, 'выжив')).map((e) => e.id)).toEqual(['survival']);
    expect(lib.skills.filter((e) => matchesLibrary(e, 'surv')).map((e) => e.id)).toEqual(['survival']);
    m.invoke('set_locale', { locale: 'ru', originalOnly: true });
    const v = m.invoke('add_skill', { id: 'survival', points: 2 }) as SheetView;
    expect(v.skills[0].name).toBe('Survival');
    expect(v.issues.some((i) => i.message.includes('specialization'))).toBe(true);
  });
});
