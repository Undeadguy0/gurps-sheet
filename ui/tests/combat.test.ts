import { describe, expect, it } from 'vitest';
import { basicDamage, computeDefenses, computeWeapons, diceStr, parseDamage, parseDb, parseDr, type CombatCtx } from '../src/combat';
import { MockBackend } from '../src/mock';
import { WEAPON_DEFS } from '../src/mockWeapons';
import type { SheetView, WeaponView } from '../src/mockTypes';

const W = (id: string) => WEAPON_DEFS.find((w) => w.id === id)!;
const dmg = (v: SheetView, id: string, mode: string) => {
  const a = v.sheet.weapons!.find((w) => w.id === id)!.attacks.find((x) => x.mode === mode)!;
  return `${a.damage} ${a.damage_type}`;
};

describe('damage expressions (mirror of gurps_core::weapon)', () => {
  it('parses and resolves with adds normalised', () => {
    expect(parseDamage('sw+1')).toEqual({ kind: 'sw', adds: 1 });
    expect(parseDamage('thr - 1')).toEqual({ kind: 'thr', adds: -1 });
    expect(parseDamage('2d+1')).toEqual({ kind: 'fixed', dice: { dice: 2, adds: 1 } });
    expect(parseDamage('x')).toBeNull();
    const [thr, sw] = basicDamage(13);
    expect([diceStr(thr), diceStr(sw)]).toEqual(['1d', '2d-1']);
  });
  it('parses DB and DR from stats', () => {
    expect([parseDb('DB 2'), parseDb('DR 4')]).toEqual([2, null]);
    expect(parseDr('DR 4/2*')).toEqual(['4/2*', 4]);
  });
});

describe('GM rogue (direct): Striking ST 2 -> damage as ST 13', () => {
  const [thrust, swing] = basicDamage(13);
  const ctx: CombatCtx = {
    skills: [{ id: 'knife', name: 'Knife', attribute: 'DX', level: 15 }], techniques: [],
    attr: (k) => ({ ST: 11, DX: 14 } as Record<string, number>)[k] ?? 10, thrust, swing,
    owned: [{ id: 'small-knife', name: 'Small Knife', carried: true }, { id: 'leather-jacket', name: 'Leather Jacket', carried: true, stats: 'DR 1' }],
    bonus: [0, 0, 0], bonusFrom: [],
  };
  const rows = computeWeapons([W('small-knife'), W('punch'), W('kick')], ctx);
  it('Small Knife 1d-1 imp / 2d-4 cut, Parry 9', () => {
    expect(rows[0].attacks.map((a) => `${a.damage} ${a.damage_type}`)).toEqual(['1d-1 imp', '2d-4 cut']);
    expect(rows[0].attacks[0].parry).toBe(9);
  });
  it('defences: Parry 9 (armed preferred over DX punch 10), Dodge 9, no Block, DR 1', () => {
    const d = computeDefenses(rows, ctx, { base_dodge: 9, dodge_penalty: 0, name: 'None' });
    expect([d.parry?.value, d.parry?.from, d.dodge.value, d.block, d.db]).toEqual([9, 'Small Knife', 9, null, 0]);
    expect(rows[1].attacks[0].parry).toBe(10);
    expect(d.armor).toEqual([{ item: 'Leather Jacket', dr: '1', value: 1 }]);
  });
});

describe('GM warrior through the mock backend', () => {
  const m = new MockBackend();
  m.invoke('new_character', { name: 'Warrior' });
  for (const [k, l] of [['ST', 3], ['DX', 2], ['HT', 2]] as const) m.invoke('set_attribute', { key: k, level: l });
  m.invoke('add_trait', { id: 'combat-reflexes' });
  m.invoke('add_trait', { id: 'lifting-st', levels: 2 });
  m.invoke('add_skill', { id: 'broadsword', points: 8 });
  m.invoke('add_skill', { id: 'shield', points: 4 });
  m.invoke('add_skill', { id: 'brawling', points: 1 });
  m.invoke('add_technique', { id: 'kicking', points: 2 });
  for (const id of ['broadsword', 'medium-shield', 'mail-shirt', 'backpack-frame']) m.invoke('add_equipment', { id });
  m.invoke('add_equipment', { id: 'personal-basics', parent: [3] });
  const v = m.invoke('add_equipment', { id: 'small-knife', quantity: 2, parent: [3] }) as SheetView;
  it('damage: Broadsword 2d cut / 1d+1 cr, Kick 1d cr, Punch 1d-1 cr', () => {
    expect(v.sheet.damage).toMatchObject({ thrust: '1d', swing: '2d-1' });
    expect([dmg(v, 'broadsword', 'swing'), dmg(v, 'broadsword', 'thrust'), dmg(v, 'kick', 'kick'), dmg(v, 'punch', 'punch')])
      .toEqual(['2d cut', '1d+1 cr', '1d cr', '1d-1 cr']);
  });
  it('defences with CR: Parry 11, Block 11, Dodge 9 encumbered, DB +2', () => {
    const d = v.sheet.defenses!;
    expect(v.sheet.encumbrance?.level).toBe(1);
    expect([d.parry?.value, d.block?.value, d.dodge.value, d.db, d.bonus]).toEqual([11, 11, 9, 2, 1]);
    expect(d.dodge.explain[0]).toBe('floor(Basic Speed) + 3 = 9 +1 (Combat Reflexes)');
  });
  it('linked weapon rows follow equipment; translated names', () => {
    const ws = v.weapons as WeaponView[];
    expect(ws.map((w) => w.id).sort()).toEqual(['broadsword', 'kick', 'punch', 'small-knife']);
    expect(ws.find((w) => w.id === 'broadsword')!.attacks[0].skill).toBe('Палаш');
    expect(ws.find((w) => w.id === 'kick')!.attacks[0].original_skill).toBe('Kicking (Brawling)');
    const after = m.invoke('remove_equipment', { path: [0] }) as SheetView;
    expect(after.weapons!.some((w) => w.id === 'broadsword')).toBe(false);
  });
});
