import { describe, expect, it } from 'vitest';
import { engineRu } from '../src/engineRu';
import { computeSpells, modifiedCost, traitCostExplain } from '../src/magic';
import { MockBackend } from '../src/mock';
import { SPELL_DEFS } from '../src/mockMagic';
import type { ModifierLibEntry, SheetView, SpellLibEntry } from '../src/types';

// Own expectations (no GM data for spells/modifiers); mirrors crates/gurps-cli/tests/spells_modifiers.rs.
describe('trait modifiers', () => {
  it('total floored at -80%, final cost rounded up', () => {
    expect(modifiedCost(5, -30)).toBe(4); // 3.5 -> 4
    expect(modifiedCost(25, -110)).toBe(5);
    expect(modifiedCost(25, 45)).toBe(37); // 36.25 -> 37
    expect(modifiedCost(-15, -50)).toBe(-7); // -7.5 -> -7 (towards +inf; flagged)
  });
  it('cost explanation lines and their translation', () => {
    const l = traitCostExplain(5, [{ name: 'Emergencies Only', cost_percent: -30 }]);
    expect(l).toEqual(['base cost 5', '-30% Emergencies Only', 'total -30%', '5 × 70% = 3.5 -> 4 (rounded up)']);
    expect(traitCostExplain(25, [{ name: 'A', cost_percent: -60 }, { name: 'B', cost_percent: -50 }])[3]).toBe('total -110% floored at -80%');
    const ru = (s: string) => engineRu(s, (x) => (x === 'Emergencies Only' ? 'Только в крайнем случае' : x));
    expect(l.map(ru)).toEqual(['базовая стоимость 5', '-30% Только в крайнем случае', 'итого -30%', '5 × 70% = 3.5 → 4 (с округлением вверх)']);
    expect(ru('total -110% floored at -80%')).toBe('итого -110%, ограничено до -80%');
    expect(ru('spells have no default: buy at least 1 pt')).toBe('у заклинаний нет умолчания: купите хотя бы 1 очко');
  });
});

describe('spells', () => {
  it('IQ/H and IQ/VH + Magery, no default', () => {
    const fb = SPELL_DEFS.find((s) => s.id === 'fireball')!; const tp = SPELL_DEFS.find((s) => s.id === 'teleport')!;
    const r = computeSpells([{ def: fb, points: 4 }, { def: tp, points: 8 }, { def: fb, points: 0 }], () => 14, [['Magery 2', 2, []]]);
    expect(r.map((x) => x.level)).toEqual([16, 16, null]);
    expect(r[0].explain).toEqual(['IQ 14', '4 pts, IQ/H -> IQ+0 = 14', '+2 from trait Magery 2', '= 16']);
    expect(r[2].explain[1]).toBe('spells have no default: buy at least 1 pt');
    expect(computeSpells([{ def: fb, points: 1 }], () => 10, [['Magery 0', 0, []]])[0].bonus).toBe(0);
  });
  it('mock backend: spells, modifiers, explanations everywhere, translated', () => {
    const m = new MockBackend();
    m.invoke('set_attribute', { key: 'IQ', level: 4 });
    m.invoke('add_trait', { id: 'magery', levels: 2 });
    let v = m.invoke('add_spell', { id: 'fireball', points: 4 }) as SheetView;
    expect(v.spells![0].level).toBe(16);
    expect(v.spells![0].name).toBe('Огненный шар');
    expect(v.spells![0].explanation).toBeTruthy();
    expect(v.sheet.points.spells).toBe(4);
    v = m.invoke('add_trait_modifier', { traitIndex: 0, id: 'emergencies-only' }) as SheetView;
    expect(v.traits[0].base_cost).toBe(25);
    expect(v.traits[0].cost).toBe(18); // 17.5 -> 18
    expect(v.traits[0].modifiers[0].explanation).toBeTruthy();
    v = m.invoke('set_modifier_explanation', { traitIndex: 0, index: 0, text: 'своё' }) as SheetView;
    expect(v.traits[0].modifiers[0]).toMatchObject({ explanation: 'своё', custom_explanation: true });
    v = m.invoke('set_modifier_explanation', { traitIndex: 0, index: 0, text: null }) as SheetView;
    expect(v.traits[0].modifiers[0].custom_explanation).toBe(false);
    v = m.invoke('set_spell_explanation', { index: 0, text: 'x' }) as SheetView;
    expect(v.spells![0].custom_explanation).toBe(true);
    const sl = m.invoke('list_spell_library', {}) as SpellLibEntry[];
    const ml = m.invoke('list_modifier_library', {}) as ModifierLibEntry[];
    expect(sl.length).toBe(6); expect(ml.length).toBe(6);
    expect([...sl, ...ml].every((e) => e.explanation && e.translated_name)).toBe(true);
    m.invoke('set_locale', { locale: 'ru', originalOnly: true });
    expect((m.invoke('list_spell_library', {}) as SpellLibEntry[]).every((e) => !e.translated_name && e.explanation)).toBe(true);
    v = m.invoke('remove_trait_modifier', { traitIndex: 0, index: 0 }) as SheetView;
    expect(v.traits[0].cost).toBe(25);
  });
});
