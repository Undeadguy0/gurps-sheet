import { describe, expect, it } from 'vitest';
import { MockBackend } from '../src/mock';
import { normalize, totals } from '../src/pool';
import { tx } from '../src/engineRu';
import type { SheetView } from '../src/types';

const v = (m: MockBackend, cmd: string, a: Record<string, unknown> = {}) => m.invoke(cmd, a) as SheetView;

describe('point pool ledger (mirror of gurps_core::ledger)', () => {
  it('normalizes signs and requires a note for other', () => {
    expect(normalize({ kind: 'influence', amount: 3 }).amount).toBe(-3);
    expect(normalize({ kind: 'session-award', amount: -2 }).amount).toBe(2);
    expect(normalize({ kind: 'correction', amount: -2 }).amount).toBe(-2);
    expect(() => normalize({ kind: 'other', amount: 1 })).toThrow();
    expect(() => normalize({ kind: 'gm-bonus', amount: 0 })).toThrow();
    expect(normalize({ kind: 'gm-bonus', amount: 1, skill: 'x' }).skill).toBeNull();
    expect(totals([{ id: 1, kind: 'starting', amount: 100 }, { id: 2, kind: 'session-award', amount: 5 }, { id: 3, kind: 'buy-off', amount: -2 }]))
      .toEqual({ starting: 100, income: 5, earned: 105, spent_outside: 2, available: 103 });
  });

  it('pool: live cost, undoable entries, header budget, warnings', () => {
    const m = new MockBackend();
    let s = v(m, 'new_character', { name: 'P' });
    expect(s.pool).toMatchObject({ starting: 150, earned: 150, available: 150, unspent: 150, disadvantage_limit: 75 });
    s = v(m, 'add_ledger_entry', { entry: { kind: 'session-award', amount: 3, note: 'подарок от Мастера', date: '2026-10-23' } });
    expect(s.history?.undo_label).toBe('запись в журнале очков');
    s = v(m, 'add_ledger_entry', { entry: { kind: 'impulse-buy', amount: 1 } });
    expect(s.pool).toMatchObject({ earned: 153, spent_outside: 1, available: 152 });
    expect(s.budget).toBe(152);
    s = v(m, 'set_attribute', { key: 'ST', level: 12 });
    expect(s.pool!.character_cost).toBe(s.sheet.points.total);
    expect(s.pool!.unspent).toBe(152 - s.sheet.points.total);
    s = v(m, 'update_ledger_entry', { id: 2, entry: { kind: 'session-award', amount: 4, note: 'x' } });
    expect(s.pool!.income).toBe(4);
    s = v(m, 'remove_ledger_entry', { id: 3 });
    expect(s.pool!.spent_outside).toBe(0);
    s = v(m, 'undo');
    expect(s.pool!.spent_outside).toBe(1);
    s = v(m, 'set_disadvantage_limit', { limit: 4 });
    s = v(m, 'add_trait', { id: 'overconfidence' });
    const w = s.issues.find((i) => i.message.startsWith('disadvantages total'))!;
    expect(w.message).toBe('disadvantages total 5 pts which is over the limit of 4 pts');
    expect(tx(w.message)).toBe('недостатков на 5 очков — больше лимита 4 очка');
    expect(s.pool).toMatchObject({ disadvantages: 5, disadvantage_limit_custom: true });
    expect(() => m.invoke('set_disadvantage_limit', { limit: -1 })).toThrow();
  });
});

describe('regression: replace the starting entry', () => {
  it('delete starting 150, add starting 200 -> one entry, budget 200', () => {
    const m = new MockBackend();
    v(m, 'new_character', {});
    v(m, 'remove_ledger_entry', { id: 1 });
    const s = v(m, 'add_ledger_entry', { entry: { kind: 'starting', amount: 200 } });
    expect(s.pool!.entries).toHaveLength(1);
    expect([s.pool!.starting, s.budget]).toEqual([200, 200]);
  });
});

describe('custom definitions', () => {
  it('saves, lists in libraries with plugin custom, edits and removes (undoable)', () => {
    const m = new MockBackend();
    v(m, 'new_character', {});
    let s = v(m, 'save_custom', { kind: 'spell', def: { name: 'Искра', difficulty: 'VH', colleges: ['fire'], description: '**жжёт**' } });
    expect(s.custom?.spells?.[0].sid).toBe('custom_spell_1');
    const lib = m.invoke('list_spell_library', {}) as { id: string; plugin: string }[];
    expect(lib.find((e) => e.id === 'custom_spell_1')?.plugin).toBe('custom');
    s = v(m, 'add_spell', { id: 'custom_spell_1' });
    expect(s.spells?.[0].difficulty).toBe('VH');
    s = v(m, 'save_custom', { kind: 'trait', def: { name: 'Удача мастера', kind: 'adv', cost: { type: 'fixed', base: 7 } } });
    expect(s.custom?.traits?.[0].sid).toBe('custom_trait_1');
    s = v(m, 'save_custom', { kind: 'spell', def: { sid: 'custom_spell_1', name: 'Искра', difficulty: 'H', colleges: ['fire'] } });
    expect(s.spells?.[0].difficulty).toBe('H');
    expect(() => m.invoke('save_custom', { kind: 'spell', def: { name: ' ' } })).toThrow();
    s = v(m, 'remove_custom', { kind: 'trait', id: 'custom_trait_1' });
    expect(s.custom?.traits).toHaveLength(0);
    expect(v(m, 'undo').custom?.traits).toHaveLength(1);
    expect(() => m.invoke('export_custom_plugin', {})).toThrow(/Tauri/);
  });
});
