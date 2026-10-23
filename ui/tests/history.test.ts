import { describe, expect, it } from 'vitest';
import { MockBackend, SplitMix, damageRoll, HISTORY_LIMIT } from '../src/mock';
import type { RollResult, SheetView } from '../src/types';

const v = (m: MockBackend, cmd: string, a: Record<string, unknown> = {}) => m.invoke(cmd, a) as SheetView;

describe('mock undo/redo history', () => {
  it('records named actions, undoes, redoes and tracks dirty', () => {
    const m = new MockBackend();
    v(m, 'new_character', { name: 'Hero' });
    expect(m.historyView()).toMatchObject({ can_undo: false, dirty: false });
    v(m, 'set_attribute', { key: 'ST', level: 2 });
    const added = v(m, 'add_trait', { id: 'combat-reflexes' });
    expect(added.history?.undo_label).toMatch(/^добавление черты «.+»$/);
    expect(added.history?.dirty).toBe(true);
    expect(() => m.invoke('remove_trait', { index: 9 })).toThrow();
    v(m, 'set_attribute', { key: 'ST', level: 2 }); // no-op, not recorded
    expect(m.historyView().undo_depth).toBe(2);
    const u = v(m, 'undo');
    expect(u.traits).toHaveLength(0);
    expect(u.history?.redo_label).toBe(added.history?.undo_label);
    expect(v(m, 'undo').history?.dirty).toBe(false);
    expect(() => m.invoke('undo', {})).toThrow();
    expect(v(m, 'redo').sheet.points.attributes).toBe(20);
    v(m, 'set_name', { name: 'B' });
    expect(m.historyView().can_redo).toBe(false);
    for (let i = 0; i < 250; i++) v(m, 'set_attribute', { key: 'DX', level: (i % 7) + 1 });
    expect(m.historyView().undo_depth).toBe(HISTORY_LIMIT);
    v(m, 'new_character', {});
    expect(m.historyView().undo_depth).toBe(0);
  });

  it('session HP/FP is clamped, undoable and separate from points', () => {
    const m = new MockBackend();
    v(m, 'new_character', {});
    const pts = v(m, 'get_sheet').sheet.points.total;
    const s = v(m, 'set_current', { stat: 'HP', value: 4 });
    expect(s.session).toMatchObject({ hp: 4, hp_max: 10 });
    expect(s.sheet.points.total).toBe(pts);
    expect(s.history?.undo_label).toBe('текущие HP');
    expect(v(m, 'set_current', { stat: 'HP', value: -999 }).session?.hp).toBe(-50);
    expect(v(m, 'set_current', { stat: 'FP', value: 99 }).session?.fp).toBe(10);
    expect(v(m, 'set_current', { stat: 'HP', value: null }).history?.undo_label).toBe('сброс HP');
    v(m, 'undo');
    expect(v(m, 'get_sheet').session?.hp).toBe(-50);
  });
});

describe('seeded rolls', () => {
  it('match the Rust SplitMix64 dice', () => {
    const m = new MockBackend();
    // values from `gurps roll 12 --seed 42` and `gurps roll 10 --mod -2 --seed 7`
    expect(m.invoke('roll', { target: 12, modifier: 0, seed: 42 })).toMatchObject({ dice: [2, 2, 1], margin: 7, outcome: 'success' });
    const r = m.invoke('roll', { target: 10, modifier: -2, seed: 7 }) as RollResult;
    expect(r).toMatchObject({ dice: [4, 1, 1], effective: 8, margin: 2 });
  });
  it('damage rolls are deterministic and respect minimums', () => {
    const a = damageRoll('2d-1', false, new SplitMix(5));
    expect(a).toEqual(damageRoll('2d-1', false, new SplitMix(5)));
    expect(a.dice).toHaveLength(2);
    expect(damageRoll('1d-6', true, new SplitMix(1)).total).toBe(0);
    expect(damageRoll('1d-6', false, new SplitMix(1)).total).toBe(1);
    expect(() => damageRoll('zz', false, new SplitMix(1))).toThrow();
  });
});

describe('mock notes', () => {
  it('one set_notes call is one undo step; whitespace counts as empty', () => {
    const m = new MockBackend();
    v(m, 'new_character', {});
    const s = v(m, 'set_notes', { text: '# Долги\n- гному 5 $' });
    expect(s.notes).toBe('# Долги\n- гному 5 $');
    expect(s.history).toMatchObject({ undo_label: 'заметки', undo_depth: 1, dirty: true });
    expect(v(m, 'undo').notes).toBe('');
    expect(v(m, 'set_notes', { text: '  \n' }).history?.undo_depth).toBe(0);
  });
});
