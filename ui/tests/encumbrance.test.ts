import { describe, expect, it } from 'vitest';
import { computeEncumbrance, levelFor, moveFor } from '../src/encumbrance';
import { MockBackend } from '../src/mock';
import { EQUIPMENT_DEFS } from '../src/mockEquipment';
import type { EquipmentItem, SheetView } from '../src/types';

const item = (id: string, quantity = 1, contents?: EquipmentItem[]): EquipmentItem => {
  const d = EQUIPMENT_DEFS.find((x) => x.id === id)!;
  return { ...d, quantity, contents };
};

describe('encumbrance maths (mirror of gurps_rules::encumbrance)', () => {
  it('levels are inclusive at BL multiples', () => {
    expect([20, 20.5, 40, 40.1, 60, 120, 200].map((w) => levelFor(w, 20)[0])).toEqual([0, 1, 1, 2, 2, 3, 4]);
    expect(levelFor(200.5, 20)).toEqual([4, true]);
  });
  it('move multipliers round down, minimum 1', () => {
    expect([0, 1, 2, 3, 4].map((l) => moveFor(7, l))).toEqual([7, 5, 4, 2, 1]);
    expect(moveFor(10, 1)).toBe(8);
  });
  it('GM warrior: 46 lb, BL 45 -> Light, Move 5, Dodge -1', () => {
    const eq = [item('broadsword'), item('medium-shield'), item('mail-shirt'),
      item('backpack-frame', 1, [item('personal-basics'), item('small-knife', 2)])];
    const e = computeEncumbrance(eq, 45, 7, 6);
    expect([e.carried_weight, e.total_cost, e.level, e.name, e.move_value, e.dodge_penalty, e.dodge]).toEqual([46, 875, 1, 'Light', 5, -1, 8]);
  });
  it('GM rogue: 5.5 lb -> None, Move 7, Dodge floor(6.75)+3 = 9', () => {
    const e = computeEncumbrance([item('small-knife', 3), item('leather-jacket')], 24, 7, 6.75);
    expect([e.carried_weight, e.total_cost, e.level, e.move_value, e.dodge]).toEqual([5.5, 140, 0, 7, 9]);
  });
});

describe('mock backend equipment', () => {
  it('containers, carried toggle, ru names, remove', () => {
    const m = new MockBackend();
    m.invoke('add_equipment', { id: 'backpack-frame' });
    m.invoke('add_equipment', { id: 'small-knife', quantity: 2, parent: [0] });
    expect(() => m.invoke('add_equipment', { id: 'torch', parent: [0, 0] })).toThrow();
    let v = m.invoke('add_equipment', { id: 'medium-shield' }) as SheetView;
    expect(v.equipment!.map((e) => [e.name, e.depth])).toEqual([['Рюкзак на раме', 0], ['Малый нож', 1], ['Средний щит', 0]]);
    expect(v.sheet.encumbrance!.carried_weight).toBe(26);
    expect(v.sheet.encumbrance!.level).toBe(1); // BL 20
    v = m.invoke('set_equipment_carried', { path: [0], carried: false }) as SheetView;
    expect(v.equipment![1].effectively_carried).toBe(false);
    expect([v.sheet.encumbrance!.carried_weight, v.sheet.encumbrance!.total_weight, v.sheet.encumbrance!.level]).toEqual([15, 26, 0]);
    v = m.invoke('set_equipment_explanation', { path: [1], text: 'мой' }) as SheetView;
    expect(v.equipment![2].custom_explanation).toBe(true);
    v = m.invoke('remove_equipment', { path: [0, 0] }) as SheetView;
    expect(v.equipment!.length).toBe(2);
    const lib = m.invoke('list_equipment_library', {}) as { translated_name: string; name: string }[];
    expect(lib.length).toBeGreaterThanOrEqual(10);
    expect(lib.every((e) => e.translated_name)).toBe(true);
  });
});
