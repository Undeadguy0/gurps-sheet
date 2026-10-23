// Browser-preview backend. Used ONLY when the UI runs outside Tauri (plain Vite dev),
// so designers can work on the UI without the Rust app. The source of truth for all
// rules is the Rust crate `gurps-rules`; this is a simplified mirror of it.
import { normalize as normalizeLedger, totals as ledgerTotals } from './pool';
import type {
  LibraryEntry, LocaleInfo, LocalizedText, RollResult, DamageRoll, LedgerEntry, LedgerInput, CustomContent, CustomKind, SheetView, StatRow, TraitDef, TraitView, Value, Outcome,
  SkillDef, SkillLibrary, SkillView, TechniqueDef, TechniqueView, EquipmentItem, EquipmentLibEntry, EquipmentView,
  WeaponDef, WeaponLibEntry, WeaponView, ModifierDef, ModifierLibEntry, NegativeRounding, ModifierView, SpellDef, SpellLibEntry, SpellView,
} from './types';
import { WEAPON_CATALOG_RU, WEAPON_DEFS } from './mockWeapons';
import { basicDamage, computeDefenses, computeWeapons, diceStr, type CombatCtx, type Owned } from './combat';
import { EQUIPMENT_CATALOG_RU, EQUIPMENT_DEFS } from './mockEquipment';
import { computeEncumbrance, totalCost, totalWeight } from './encumbrance';
import { SKILL_CATALOG_RU, SKILL_DEFS, TALENT_DEF, TECHNIQUE_DEFS } from './mockSkills';
import { MAGERY_DEF, MAGERY_RU, MODIFIER_CATALOG_RU, MODIFIER_DEFS, SPELL_CATALOG_RU, SPELL_DEFS } from './mockMagic';
import { computeSpells, modifiedCost, traitCostExplain } from './magic';
import { computeSkills, computeTechniques, signedInt, enPts } from './skills';

interface MockSkill { def: SkillDef; points: number; specialization?: string | null; explanation?: string }
interface MockTech { def: TechniqueDef; points: number; explanation?: string }
interface MockMod { def: ModifierDef; explanation?: string }
interface MockTrait { def: TraitDef; levels: number; source: string; explanation?: string; modifiers: MockMod[] }
interface MockSpell { def: SpellDef; points: number; explanation?: string }

/** Points per step (B15-B17); Basic Speed per 0.25. Mirrors gurps_rules::points::cost_per_step. */
export const COST: Record<string, number> = {
  ST: 10, DX: 20, IQ: 20, HT: 10, HP: 2, Will: 5, Per: 5, FP: 3, BasicSpeed: 5, BasicMove: 5,
};
const LABEL: Record<string, string> = { BasicSpeed: 'Basic Speed', BasicMove: 'Basic Move', BasicLift: 'Basic Lift' };
const PRIMARY = ['ST', 'DX', 'IQ', 'HT'];
const SECONDARY = ['HP', 'Will', 'Per', 'FP', 'BasicSpeed', 'BasicMove'];

/** Same fallback order as gurps_core::LocalizedText::get. */
export function localize(t: LocalizedText | null | undefined, locale = 'ru'): string | null {
  if (t == null) return null;
  const ok = (s?: string) => (s && s.trim() ? s : null);
  if (typeof t === 'string') return ok(t);
  const lang = locale.split(/[-_]/)[0];
  return ok(t[locale]) ?? ok(t[lang]) ?? ok(t.en) ?? ok(t.ru) ?? Object.values(t).map(ok).find((x) => x) ?? null;
}

// Own-words sample texts (mirrors plugins/basic-set-sample; not book text).
const RAW_LIBRARY: Omit<LibraryEntry, 'explanation'>[] = [
  { plugin: 'basic-set-sample', id: 'increased-st-example', name: 'Increased ST (example)', category: 'advantage', cost: 0, cost_per_level: 10, page: 'example', features: [{ type: 'attribute_bonus', target: 'ST', amount: 1, per_level: true }],
    description: "A sample trait used to test the sheet: each level simply adds +1 to ST, and everything calculated from ST follows automatically (Hit Points, Basic Lift, thrust and swing damage).\n\nUse it to see how the sheet explains where a number comes from: the ST row will list the trait as a separate contribution next to the purchased levels." },
  { plugin: 'basic-set-sample', id: 'lifting-st', name: 'Lifting ST', category: 'advantage', cost: 0, cost_per_level: 3, page: 'B65', features: [{ type: 'lifting_st_bonus', amount: 1, per_level: true }],
    description: "Extra muscle that only counts when you lift, carry, push or drag something. Each level raises the ST used to work out Basic Lift, so the character can haul heavier loads before being slowed down. It does not make blows hit harder and does not add Hit Points.\n\nIn play it suits porters, labourers, climbers who carry gear and anyone who needs to move a lot of weight cheaply. Combine it with ordinary ST if you also want stronger attacks." },
  { plugin: 'basic-set-sample', id: 'combat-reflexes', name: 'Combat Reflexes', category: 'advantage', cost: 15, cost_per_level: 0, page: 'B43', features: [{ type: 'defense_bonus', amount: 1 }],
    description: "The character reacts to danger almost instantly. Surprise rarely freezes them, they recover from being stunned faster than most, and they are a little better at dodging, blocking and parrying. Fear and shock also have a weaker grip on them.\n\nIn play this is the classic trait of veterans, bodyguards and duellists: they act first in an ambush and keep their head when things go wrong. The sheet adds +1 to Dodge, Parry and Block; the other effects (surprise, fear, stun recovery) are still applied at the table." },
  { plugin: 'basic-set-sample', id: 'overconfidence', name: 'Overconfidence', category: 'disadvantage', cost: -5, cost_per_level: 0, page: 'B148', features: [],
    description: "The character believes they can handle anything and consistently underestimates risk. When a sensible person would back off, plan or ask for help, they tend to charge ahead instead, unless they manage to resist the urge.\n\nIt makes for bold, memorable characters (young heroes, braggarts, star pilots), but the player should actually play the recklessness. Others may find them inspiring or insufferable." },
];
RAW_LIBRARY.push({ plugin: 'basic-set-sample', ...TALENT_DEF } as Omit<LibraryEntry, 'explanation'>);
RAW_LIBRARY.push({ plugin: 'basic-set-sample', ...MAGERY_DEF } as Omit<LibraryEntry, 'explanation'>);
const LIBRARY = RAW_LIBRARY;

/** Mirror of plugins/lang-ru/locales/ru/basic-set-sample.ftl (subset). Keys: `<kind>-<id>`. */
export const CATALOG: Record<string, Record<string, { name?: string; description?: string }>> = {
  ru: {
    'trait-increased-st-example': { name: 'Повышенная ST (пример)', description: "Пробная черта для проверки листа: каждый уровень просто добавляет +1 к ST, а всё, что считается от ST, меняется само (HP, базовый груз, урон толчком и замахом).\n\nУдобна, чтобы посмотреть, как лист объясняет происхождение чисел: в строке ST черта будет отдельным слагаемым рядом с купленными уровнями." },
    'trait-lifting-st': { name: 'Подъёмная сила', description: "Дополнительная мускульная сила, которая работает только при подъёме, переноске, толкании или волочении. Каждый уровень повышает ST, от которой считается базовый груз, поэтому персонаж может тащить больше, не замедляясь. На силу ударов и на HP она не влияет.\n\nПодходит носильщикам, грузчикам, альпинистам со снаряжением и всем, кому нужно дёшево таскать тяжести. Если нужны и сильные удары — сочетайте с обычной ST." },
    'trait-combat-reflexes': { name: 'Боевые рефлексы', description: "Персонаж реагирует на опасность почти мгновенно. Внезапность редко сбивает его с толку, из оглушения он выходит быстрее других и чуть лучше уклоняется, блокирует и парирует. Страх и шок тоже действуют на него слабее.\n\nЭто типичная черта ветеранов, телохранителей и дуэлянтов: в засаде они действуют первыми и не теряют голову, когда всё идёт плохо. Лист добавляет +1 к уклонению, парированию и блоку; остальное (внезапность, страх, оглушение) учитывайте за столом." },
    'trait-overconfidence': { name: 'Самоуверенность', description: "Персонаж уверен, что справится с чем угодно, и постоянно недооценивает риск. Там, где разумный человек отступил бы, подготовился или позвал на помощь, он склонен лезть вперёд, если не сумеет сдержаться.\n\nДаёт смелых и запоминающихся персонажей (юные герои, хвастуны, лихие пилоты), но безрассудство нужно действительно отыгрывать. Окружающих это может как вдохновлять, так и раздражать." },
  },
};

const fmt = (v: number) => (Number.isInteger(v) ? String(v) : String(+v.toFixed(2)));
const bl = (st: number) => { const v = (st * st) / 5; return v >= 10 ? Math.round(v) : Math.round(v * 10) / 10; };

export function outcome(roll: number, skill: number): Outcome {
  if (roll <= 4 || (roll === 5 && skill >= 15) || (roll === 6 && skill >= 16)) return 'critical_success';
  if (roll === 18) return 'critical_failure';
  if (roll === 17) return skill <= 15 ? 'critical_failure' : 'failure';
  if (roll >= skill + 10) return 'critical_failure';
  return roll <= skill ? 'success' : 'failure';
}

Object.assign(CATALOG.ru, SKILL_CATALOG_RU);
for (const [id, v] of Object.entries(EQUIPMENT_CATALOG_RU)) CATALOG.ru[`equipment-${id}`] = v;
for (const [id, v] of Object.entries(WEAPON_CATALOG_RU)) CATALOG.ru[`weapon-${id}`] = v;

interface MockWeapon { def: WeaponDef; explanation?: string }
const unarmed = (): MockWeapon[] => WEAPON_DEFS.filter((w) => !w.equipment).map((def) => ({ def }));

CATALOG.ru['trait-magery'] = MAGERY_RU;
for (const [k, v] of Object.entries(SPELL_CATALOG_RU)) CATALOG.ru[`spell-${k}`] = v;
for (const [k, v] of Object.entries(MODIFIER_CATALOG_RU)) CATALOG.ru[`modifier-${k}`] = v;

/** SplitMix64, bit-identical to gurps-rules (same seed, same dice as the app). */
export class SplitMix {
  private s: bigint;
  constructor(seed: number | bigint) { this.s = BigInt.asUintN(64, BigInt(seed)); }
  next(): bigint {
    const M = (x: bigint) => BigInt.asUintN(64, x);
    this.s = M(this.s + 0x9E3779B97F4A7C15n);
    let z = this.s;
    z = M((z ^ (z >> 30n)) * 0xBF58476D1CE4E5B9n);
    z = M((z ^ (z >> 27n)) * 0x94D049BB133111EBn);
    return z ^ (z >> 31n);
  }
  d6(): number { return Number(this.next() % 6n) + 1; }
}

/** "2d-1" style damage; minimum 0 for crushing, else 1 (B378, # unverified). */
export function damageRoll(expr: string, crushing: boolean, rng: SplitMix): DamageRoll {
  const m = /^\s*(\d+)\s*d\s*([+-]\s*\d+)?\s*$/i.exec(expr);
  if (!m) throw new Error(`cannot parse dice '${expr}'`);
  const dice = Array.from({ length: +m[1] }, () => rng.d6());
  const adds = m[2] ? parseInt(m[2].replace(/\s/g, ''), 10) : 0;
  return { expr, dice, adds, total: Math.max(crushing ? 0 : 1, dice.reduce((x, y) => x + y, 0) + adds) };
}

/** Russian action labels for undoable commands (same as src-tauri/src/lib.rs). */
const ACTION: Record<string, string> = {
  set_negative_rounding: 'правило округления', set_name: 'переименование', set_notes: 'заметки',
  add_ledger_entry: 'запись в журнале очков', update_ledger_entry: 'правка записи журнала очков', remove_ledger_entry: 'удаление записи журнала очков',
  set_disadvantage_limit: 'лимит недостатков', save_custom: 'своё определение', remove_custom: 'удаление своего определения',
  add_trait: 'добавление черты', set_trait_levels: 'уровень черты', remove_trait: 'удаление черты', set_trait_explanation: 'пояснение черты',
  add_skill: 'добавление навыка', set_skill_points: 'очки навыка', set_skill_specialization: 'специализация навыка',
  set_skill_explanation: 'пояснение навыка', remove_skill: 'удаление навыка',
  add_technique: 'добавление приёма', set_technique_points: 'очки приёма', set_technique_explanation: 'пояснение приёма', remove_technique: 'удаление приёма',
  add_equipment: 'добавление снаряжения', set_equipment_quantity: 'количество снаряжения', set_equipment_carried: 'при себе / оставлено',
  set_equipment_explanation: 'пояснение снаряжения', remove_equipment: 'удаление снаряжения',
  add_weapon: 'добавление оружия', remove_weapon: 'удаление оружия', set_weapon_explanation: 'пояснение оружия',
  add_spell: 'добавление заклинания', set_spell_points: 'очки заклинания', set_spell_explanation: 'пояснение заклинания', remove_spell: 'удаление заклинания',
  add_trait_modifier: 'добавление модификатора', remove_trait_modifier: 'удаление модификатора', set_modifier_explanation: 'пояснение модификатора',
};
export const HISTORY_LIMIT = 200;
type Snap = Record<string, unknown>;
const SNAP_KEYS = ['name', 'notes', 'ledger', 'disLimit', 'custom', 'levels', 'traits', 'skills', 'techniques', 'spells', 'equipment', 'weapons', 'rounding', 'session'] as const;

export class MockBackend {
  session: { hp: number | null; fp: number | null } = { hp: null, fp: null };
  notes = '';
  ledger: LedgerEntry[] = [{ id: 1, amount: 150, kind: 'starting' }];
  disLimit: number | null = null;
  custom: CustomContent = {};
  private disLimitValue() { return this.disLimit ?? Math.trunc(ledgerTotals(this.ledger).starting / 2); }
  private customList<T>(kind: CustomKind): T[] {
    const k = kind === 'equipment' ? 'equipment' : (`${kind}s` as 'traits' | 'skills' | 'spells');
    return ((this.custom[k] ??= []) as unknown) as T[];
  }
  rng = new SplitMix(Date.now());
  private undoStack: [string, Snap][] = [];
  private redoStack: [string, Snap][] = [];
  private saved: string = '';
  private snap(): Snap { return structuredClone(Object.fromEntries(SNAP_KEYS.map((k) => [k, (this as unknown as Snap)[k]]))); }
  private restore(s: Snap) { Object.assign(this, structuredClone(s)); }
  private key(s: Snap) { return JSON.stringify(s); }
  /** «name» of the list item an edit touched (added/removed/changed). */
  private changedItem(before: Snap, after: Snap): string | null {
    for (const k of ['traits', 'skills', 'techniques', 'spells', 'equipment', 'weapons']) {
      type It = { id?: string; name?: string; def?: { id: string; name: string } };
      const a = before[k] as It[]; const b = after[k] as It[];
      for (let i = 0; i < Math.max(a.length, b.length); i++) {
        if (JSON.stringify(a[i]) === JSON.stringify(b[i])) continue;
        const it = a.length > b.length ? a[i] : (b[i] ?? a[i]);
        const d = it.def ?? { id: it.id ?? '', name: it.name ?? '' };
        return this.tr(d.id, k === 'equipment' ? k : k.slice(0, -1))?.name ?? d.name;
      }
    }
    return null;
  }
  historyView() {
    return { can_undo: this.undoStack.length > 0, can_redo: this.redoStack.length > 0,
      undo_label: this.undoStack[this.undoStack.length - 1]?.[0] ?? null, redo_label: this.redoStack[this.redoStack.length - 1]?.[0] ?? null,
      undo_depth: this.undoStack.length, dirty: this.key(this.snap()) !== this.saved };
  }
  private resetHistory() { this.undoStack = []; this.redoStack = []; this.saved = this.key(this.snap()); }
  private withMeta(r: unknown) {
    if (r && typeof r === 'object' && 'sheet' in r) (r as SheetView).history = this.historyView();
    return r;
  }

  invoke(cmd: string, a: Record<string, unknown>): unknown {
    if (!this.saved) this.saved = this.key(this.snap());
    if (cmd === 'undo' || cmd === 'redo') {
      const [from, to] = cmd === 'undo' ? [this.undoStack, this.redoStack] : [this.redoStack, this.undoStack];
      const e = from.pop(); if (!e) throw new Error(cmd === 'undo' ? 'nothing to undo' : 'nothing to redo');
      to.push([e[0], this.snap()]); this.restore(e[1]);
      return this.withMeta(this.view());
    }
    let label = ACTION[cmd] ?? (cmd === 'set_attribute' ? `изменение ${a.key}` : null);
    if (cmd === 'set_current') label = `${a.value == null ? 'сброс' : 'текущие'} ${a.stat}`;
    if (!label) { const r = this.exec(cmd, a); if (cmd === 'new_character') this.resetHistory(); return this.withMeta(r); }
    const before = this.snap();
    try { this.exec(cmd, a); } catch (e) { this.restore(before); throw e; }
    const after = this.snap();
    if (this.key(after) !== this.key(before)) {
      const item = this.changedItem(before, after);
      this.undoStack.push([item ? `${label} «${item}»` : label, before]);
      if (this.undoStack.length > HISTORY_LIMIT) this.undoStack.shift();
      this.redoStack = [];
    }
    return this.withMeta(this.view());
  }

  name = 'Пример (браузер)';
  levels: Record<string, number> = {};
  traits: MockTrait[] = [];
  locale = 'ru';
  originalOnly = false;

  spells: MockSpell[] = [];
  skills: MockSkill[] = [];
  techniques: MockTech[] = [];
  private tr(id: string, kind = 'trait') {
    if (this.originalOnly) return undefined;
    return (CATALOG[this.locale] ?? CATALOG[this.locale.split(/[-_]/)[0]])?.[`${kind}-${id}`];
  }
  private text(kind: string, id: string, own?: LocalizedText | null) {
    return this.tr(id, kind)?.description ?? localize(own, this.originalOnly ? 'en' : this.locale);
  }
  private skillName(id: string) {
    return this.tr(id, 'skill')?.name ?? SKILL_DEFS.find((d) => d.id === id)?.name ?? id;
  }
  equipment: EquipmentItem[] = [];
  weapons: MockWeapon[] = unarmed();
  weaponLibrary(): WeaponLibEntry[] {
    return WEAPON_DEFS.map((d) => ({ ...d, plugin: 'basic-set-sample', explanation: this.text('weapon', d.id, d.description),
      translated_name: this.tr(d.id, 'weapon')?.name ?? null, skill_name: this.skillName(d.skill),
      equipment_name: d.equipment ? this.tr(d.equipment, 'equipment')?.name ?? EQUIPMENT_DEFS.find((e) => e.id === d.equipment)?.name ?? d.equipment : null }));
  }
  private combat(tot: Record<string, number>, skills: SkillView[], techniques: TechniqueView[], enc: { base_dodge: number; dodge_penalty: number; name: string }) {
    let strikeSt = tot.ST;
    const bonus: [number, number, number] = [0, 0, 0]; const bonusFrom: string[] = [];
    for (const t of this.traits) for (const f of t.def.features) {
      const amt = f.per_level ? f.amount * Math.max(1, t.levels) : f.amount;
      if (f.type === 'striking_st_bonus') strikeSt += amt;
      if (f.type === 'defense_bonus') {
        ['dodge', 'parry', 'block'].forEach((k, i) => { if (!f.defenses?.length || f.defenses.includes(k)) bonus[i] += Math.trunc(amt); });
        bonusFrom.push(this.name_(t));
      }
    }
    const [thrust, swing] = basicDamage(strikeSt);
    const owned: Owned[] = [];
    const walk = (l: EquipmentItem[], c: boolean) => l.forEach((e) => { const cc = c && e.carried !== false; owned.push({ id: e.id, name: e.name, carried: cc, stats: e.stats }); walk(e.contents ?? [], cc); });
    walk(this.equipment, true);
    const ctx: CombatCtx = {
      skills: skills.map((r) => ({ id: r.id, name: r.original_name ?? r.name, attribute: r.attribute, level: r.level ?? null })),
      techniques: techniques.map((r) => ({ id: r.id, name: r.original_name ?? r.name, level: r.level ?? null })),
      attr: (k) => tot[k] ?? 10, thrust, swing, owned, bonus, bonusFrom,
    };
    const rows = computeWeapons(this.weapons.map((w) => w.def), ctx);
    const views: WeaponView[] = rows.map((r) => {
      const w = this.weapons[r.index]; const n = this.tr(r.id, 'weapon')?.name; const dflt = this.text('weapon', r.id, w.def.description);
      return { index: r.index, id: r.id, name: n ?? r.name, original_name: n ? r.name : null, equipment: r.equipment, available: r.available, ready: r.ready,
        attacks: r.attacks.map((x) => { const t = x.skill_id ? this.tr(x.skill_id, x.via_technique ? 'technique' : 'skill')?.name : undefined;
          return { ...x, skill: t ?? x.skill, original_skill: t ? x.skill : null }; }),
        page: w.def.page, source: 'basic-set-sample', explanation: w.explanation?.trim() ? w.explanation : dflt, default_explanation: dflt, custom_explanation: !!w.explanation?.trim() };
    });
    return { rows, views, defenses: computeDefenses(rows, ctx, enc), damage: { thrust: diceStr(thrust), swing: diceStr(swing), based_on_st: strikeSt } };
  }
  private pruneWeapons() {
    const ids = new Set<string>(); const walk = (l: EquipmentItem[]) => l.forEach((e) => { ids.add(e.id); walk(e.contents ?? []); }); walk(this.equipment);
    this.weapons = this.weapons.filter((w) => !w.def.equipment || ids.has(w.def.equipment));
  }
  equipmentLibrary(): EquipmentLibEntry[] {
    return [...EQUIPMENT_DEFS.filter((d) => !this.custom.equipment?.some((c) => c.id === d.id)).map((d) => ({ ...d, plugin: 'basic-set-sample', explanation: this.text('equipment', d.id, d.description), translated_name: this.tr(d.id, 'equipment')?.name ?? null })),
      ...(this.custom.equipment ?? []).map((d) => ({ ...d, plugin: 'custom', explanation: localize(d.description), translated_name: null }))];
  }
  private equipmentViews(): EquipmentView[] {
    const out: EquipmentView[] = [];
    const walk = (list: EquipmentItem[], path: number[], parentCarried: boolean) => list.forEach((e, i) => {
      const p = [...path, i]; const n = this.tr(e.id, 'equipment')?.name; const lib = EQUIPMENT_DEFS.find((d) => d.id === e.id);
      const dflt = this.text('equipment', e.id, lib?.description ?? e.description); const carried = parentCarried && e.carried !== false;
      out.push({ path: p, depth: path.length, id: e.id, name: n ?? e.name, original_name: n ? e.name : null, quantity: e.quantity,
        weight: e.weight, cost: e.cost, total_weight: totalWeight(e), total_cost: totalCost(e), carried: e.carried !== false,
        effectively_carried: carried, container: !!e.container, stats: e.stats ?? null, page: e.page, source: e.source,
        explanation: e.explanation?.trim() ? e.explanation : dflt, default_explanation: dflt, custom_explanation: !!e.explanation?.trim() });
      walk(e.contents ?? [], p, carried);
    });
    walk(this.equipment, [], true);
    return out;
  }
  private item(a: Record<string, unknown>): EquipmentItem {
    let list = this.equipment; let e: EquipmentItem | undefined;
    for (const i of a.path as number[]) { e = list[i]; if (!e) throw new Error('no such item'); list = e.contents ?? []; }
    if (!e) throw new Error('no such item');
    return e;
  }
  skillLibrary(): SkillLibrary {
    return {
      skills: [...SKILL_DEFS.map((d) => ({ ...d, plugin: 'basic-set-sample', explanation: this.text('skill', d.id, d.description), translated_name: this.tr(d.id, 'skill')?.name ?? null })),
        ...(this.custom.skills ?? []).map((d) => ({ ...d, plugin: 'custom', explanation: localize(d.description), translated_name: null }))],
      techniques: TECHNIQUE_DEFS.map((d) => ({ ...d, plugin: 'basic-set-sample', explanation: this.text('technique', d.id, d.description),
        translated_name: this.tr(d.id, 'technique')?.name ?? null, skill_name: this.skillName(d.skill) })),
    };
  }
  private skillViews(tot: Record<string, number>): [SkillView[], TechniqueView[], number] {
    const talents: Record<string, [string, number][]> = {};
    for (const t of this.traits) for (const f of t.def.features) if (f.type === 'skill_bonus')
      for (const id of f.skills) (talents[id] ??= []).push([this.name_(t), Math.floor(f.per_level ? f.amount * Math.max(1, t.levels) : f.amount)]);
    const rows = computeSkills(this.skills, (k) => tot[k] ?? 10, talents);
    const trows = computeTechniques(this.techniques, this.skills, rows);
    const sv: SkillView[] = this.skills.map((s, index) => {
      const r = rows[index]; const n = this.tr(s.def.id, 'skill')?.name;
      const shown = n ? (s.specialization?.trim() ? `${n} (${s.specialization.trim()})` : n) : null;
      const dflt = this.text('skill', s.def.id, s.def.description);
      return { index, id: s.def.id, name: shown ?? r.name, original_name: shown ? r.name : null, attribute: s.def.attribute, difficulty: s.def.difficulty,
        points: s.points, level: r.level, relative: r.relative, from_default: r.from_default, bonus: r.bonus, explain: r.explain,
        defaults: (s.def.defaults ?? []).map((d) => ('attribute' in d ? `${d.attribute}${signedInt(d.modifier)}` : `${this.skillName(d.skill)}${signedInt(d.modifier)}`)),
        specialization: s.specialization ?? null, specialization_required: !!s.def.specialization_required, specializations: s.def.specializations ?? [],
        page: s.def.page, source: 'basic-set-sample', explanation: s.explanation?.trim() ? s.explanation : dflt, default_explanation: dflt, custom_explanation: !!s.explanation?.trim() };
    });
    const tv: TechniqueView[] = this.techniques.map((t, index) => {
      const r = trows[index]; const n = this.tr(t.def.id, 'technique')?.name; const sk = this.skillName(t.def.skill);
      const dflt = this.text('technique', t.def.id, t.def.description);
      return { index, id: t.def.id, name: n ? `${n} (${sk})` : r.name, original_name: n ? r.name : null, skill: t.def.skill, difficulty: t.def.difficulty,
        modifier: t.def.modifier, max: t.def.max, points: t.points, level: r.level, max_level: r.max_level,
        relative: r.level === null || r.parent === null ? null : `${sk}${signedInt(r.level - r.parent)}`, explain: r.explain, page: t.def.page, source: 'basic-set-sample',
        explanation: t.explanation?.trim() ? t.explanation : dflt, default_explanation: dflt, custom_explanation: !!t.explanation?.trim() };
    });
    const pts = this.skills.reduce((a, s) => a + Math.max(0, s.points), 0) + this.techniques.reduce((a, t) => a + Math.max(0, t.points), 0);
    return [sv, tv, pts];
  }
  private plugText(def: TraitDef) { return this.tr(def.id)?.description ?? localize(def.description, this.originalOnly ? 'en' : this.locale); }
  private shown(t: MockTrait) { const n = this.tr(t.def.id)?.name; return n ? (t.def.cost_per_level ? `${n} ${t.levels}` : n) : undefined; }
  library(): LibraryEntry[] {
    return [...LIBRARY.map((e) => ({ ...e, explanation: this.plugText(e), translated_name: this.tr(e.id)?.name ?? null })),
      ...(this.custom.traits ?? []).map((d) => ({ ...d, plugin: 'custom', explanation: localize(d.description), translated_name: null }))];
  }
  locales(): LocaleInfo { return { available: [...new Set([...Object.keys(CATALOG), 'en'])].sort(), locale: this.locale, original_only: this.originalOnly }; }

  private name_(t: MockTrait) { return t.def.cost_per_level ? `${t.def.name} ${t.levels}` : t.def.name; }
  private baseCost(t: MockTrait) { return t.def.cost + t.def.cost_per_level * t.levels; }
  private modPct(t: MockTrait) { return t.modifiers.reduce((a, m) => a + m.def.cost_percent, 0); }
  rounding: NegativeRounding = 'toward_positive';
  private cost(t: MockTrait) { return t.modifiers.length ? modifiedCost(this.baseCost(t), this.modPct(t), this.rounding) : this.baseCost(t); }
  spellLibrary(): SpellLibEntry[] {
    return [...SPELL_DEFS.map((d) => ({ ...d, plugin: 'basic-set-sample', explanation: this.text('spell', d.id, d.description), translated_name: this.tr(d.id, 'spell')?.name ?? null })),
      ...(this.custom.spells ?? []).map((d) => ({ ...d, plugin: 'custom', explanation: localize(d.description), translated_name: null }))];
  }
  modifierLibrary(): ModifierLibEntry[] {
    return MODIFIER_DEFS.map((d) => ({ ...d, plugin: 'basic-set-sample', explanation: this.text('modifier', d.id, d.description), translated_name: this.tr(d.id, 'modifier')?.name ?? null }));
  }
  private modViews(t: MockTrait): ModifierView[] {
    return t.modifiers.map((m, index) => {
      const n = this.tr(m.def.id, 'modifier')?.name; const dflt = this.text('modifier', m.def.id, m.def.description);
      return { index, id: m.def.id, name: n ?? m.def.name, original_name: n ? m.def.name : null, cost_percent: m.def.cost_percent, page: m.def.page,
        source: 'basic-set-sample', explanation: m.explanation?.trim() ? m.explanation : dflt, default_explanation: dflt, custom_explanation: !!m.explanation?.trim() };
    });
  }
  private spellViews(tot: Record<string, number>): SpellView[] {
    const bonuses: [string, number, string[]][] = [];
    for (const t of this.traits) for (const f of t.def.features) if (f.type === 'spell_bonus')
      bonuses.push([this.name_(t), Math.floor(f.per_level ? f.amount * t.levels : f.amount), f.colleges ?? []]);
    const rows = computeSpells(this.spells, (k) => tot[k] ?? 10, bonuses);
    return this.spells.map((sp, index) => {
      const r = rows[index]; const n = this.tr(sp.def.id, 'spell')?.name; const dflt = this.text('spell', sp.def.id, sp.def.description);
      return { index, id: sp.def.id, name: n ?? sp.def.name, original_name: n ? sp.def.name : null, college: sp.def.college ?? [], attribute: sp.def.attribute ?? 'IQ',
        difficulty: sp.def.difficulty ?? 'H', points: sp.points, level: r.level, relative: r.relative, bonus: r.bonus, prerequisites: sp.def.prerequisites ?? null,
        explain: r.explain, page: sp.def.page, source: 'basic-set-sample', explanation: sp.explanation?.trim() ? sp.explanation : dflt, default_explanation: dflt,
        custom_explanation: !!sp.explanation?.trim() };
    });
  }
  private spell(a: Record<string, unknown>) { const s = this.spells[a.index as number]; if (!s) throw new Error('no such spell'); return s; }
  private trait(i: unknown) { const t = this.traits[i as number]; if (!t) throw new Error('no such trait'); return t; }

  private row(key: string, v: Value): StatRow {
    const level = this.levels[key] ?? 0;
    const total = v.base + v.contributions.reduce((a, c) => a + c.amount, 0);
    const explain = [`base ${fmt(v.base)} (${v.base_reason})`,
      ...v.contributions.map((c) => `${c.amount >= 0 ? '+' : ''}${fmt(c.amount)} ${c.reason}`), `= ${fmt(total)}`];
    return { key, label: LABEL[key] ?? key, level, step: key === 'BasicSpeed' ? 0.25 : 1, cost_per_step: COST[key] ?? 0, points: level * (COST[key] ?? 0), total, value: v, explain };
  }

  private value(key: string, base: number, reason: string): Value {
    const v: Value = { base, base_reason: reason, contributions: [] };
    const lv = this.levels[key] ?? 0;
    const step = key === 'BasicSpeed' ? 0.25 : 1;
    if (lv) v.contributions.push({ source: { kind: 'purchased' }, amount: lv * step, reason: `purchased (${enPts(lv * COST[key])})` });
    for (const t of this.traits) for (const f of t.def.features)
      if (f.type === 'attribute_bonus' && f.target === key) {
        const n = this.name_(t);
        v.contributions.push({ source: { kind: 'trait', ref: n }, amount: f.per_level ? f.amount * Math.max(1, t.levels) : f.amount, reason: `from trait ${n}` });
      }
    return v;
  }

  view(): SheetView {
    const tot: Record<string, number> = {};
    const attributes = PRIMARY.map((k) => { const r = this.row(k, this.value(k, 10, 'default')); tot[k] = r.total; return r; });
    const secondary: StatRow[] = [];
    for (const k of SECONDARY) {
      const [b, why] = k === 'HP' ? [tot.ST, `= ST ${tot.ST}`] : k === 'Will' || k === 'Per' ? [tot.IQ, `= IQ ${tot.IQ}`]
        : k === 'FP' ? [tot.HT, `= HT ${tot.HT}`] : k === 'BasicSpeed' ? [(tot.DX + tot.HT) / 4, `= (DX ${tot.DX} + HT ${tot.HT}) / 4`]
        : [Math.floor(tot.BasicSpeed), `= floor(Basic Speed ${fmt(tot.BasicSpeed)})`];
      const r = this.row(k, this.value(k, b as number, why as string)); tot[k] = r.total; secondary.push(r);
    }
    let liftSt = tot.ST; const blv: Value = { base: bl(tot.ST), base_reason: `= ST ${tot.ST}² / 5`, contributions: [] };
    for (const t of this.traits) for (const f of t.def.features) if (f.type === 'lifting_st_bonus') {
      const prev = bl(liftSt); liftSt += f.per_level ? f.amount * Math.max(1, t.levels) : f.amount;
      blv.contributions.push({ source: { kind: 'trait', ref: this.name_(t) }, amount: bl(liftSt) - prev, reason: `from trait ${this.name_(t)} (ST ${liftSt} for lifting)` });
    }
    const sum = (ks: string[]) => ks.reduce((a, k) => a + (this.levels[k] ?? 0) * COST[k], 0);
    const cat = (c: string) => this.traits.filter((t) => t.def.category === c).reduce((a, t) => a + this.cost(t), 0);
    const p = { attributes: sum(PRIMARY), secondary: sum(SECONDARY), advantages: cat('advantage'), perks: cat('perk'),
      disadvantages: cat('disadvantage'), quirks: cat('quirk'), skills: 0, spells: 0, total: 0, budget: ledgerTotals(this.ledger).available, unspent: 0 };
    const [skills, techniques, skillPts] = this.skillViews(tot);
    p.skills = skillPts;
    const spells = this.spellViews(tot);
    p.spells = this.spells.reduce((a, sp) => a + Math.max(0, sp.points), 0);
    p.total = p.attributes + p.secondary + p.advantages + p.perks + p.disadvantages + p.quirks + p.skills + p.spells;
    p.unspent = p.budget - p.total;
    const traits: TraitView[] = this.traits.map((t, index) => ({ index, id: t.def.id, name: this.shown(t) ?? this.name_(t),
      original_name: this.shown(t) ? this.name_(t) : null, category: t.def.category,
      levels: t.levels, leveled: !!t.def.cost_per_level, min_level: t.def.min_levels ?? (t.def.cost_per_level ? 1 : 0), cost: this.cost(t), page: t.def.page, source: t.source,
      explanation: t.explanation?.trim() ? t.explanation : this.plugText(t.def),
      default_explanation: this.plugText(t.def), custom_explanation: !!t.explanation?.trim(),
      base_cost: this.baseCost(t), modifier_percent: this.modPct(t), modifiers: this.modViews(t),
      cost_explain: traitCostExplain(this.baseCost(t), t.modifiers.map((m) => m.def), this.rounding) }));
    const encumbrance = computeEncumbrance(this.equipment, bl(liftSt), tot.BasicMove, tot.BasicSpeed);
    const cb = this.combat(tot, skills, techniques, encumbrance);
    return {
      sheet: { name: this.name, attributes, secondary, basic_lift: this.row('BasicLift', blv),
        damage: cb.damage, points: p, encumbrance, weapons: cb.rows, defenses: cb.defenses },
      traits, skills, techniques, spells, equipment: this.equipmentViews(), weapons: cb.views,
      issues: [...(-p.disadvantages > this.disLimitValue() ? [{ severity: 'warning' as const,
          message: `disadvantages total ${enPts(-p.disadvantages)} which is over the limit of ${enPts(this.disLimitValue())}` }] : []),
        ...((() => { const q = this.traits.filter((t) => t.def.category === 'quirk').length;
          return q > 5 || p.quirks < -5 ? [{ severity: 'warning' as const, message: `${q} quirks for ${enPts(p.quirks)}: the usual maximum is 5 quirks at -1` }] : []; })()),
        ...(p.unspent < 0 ? [{ severity: 'warning' as const, message: `over budget by ${enPts(-p.unspent)}` }] : []),
        ...this.skills.filter((s) => s.def.specialization_required && !s.specialization?.trim())
          .map((s) => ({ severity: 'warning' as const, message: `skill '${s.def.name}' needs a specialization` })),
        ...(encumbrance.carried_weight > bl(liftSt) * 10 + 1e-9
          ? [{ severity: 'warning' as const, message: `carried ${encumbrance.carried_weight} lb is more than 10×BL (${bl(liftSt) * 10} lb): the character cannot move` }] : [])],
      pool: (() => { const t = ledgerTotals(this.ledger); return { entries: structuredClone(this.ledger), ...t, character_cost: p.total,
        unspent: t.available - p.total, disadvantage_limit: this.disLimitValue(), disadvantage_limit_custom: this.disLimit !== null,
        disadvantages: -p.disadvantages, quirks: this.traits.filter((x) => x.def.category === 'quirk').length, quirk_points: p.quirks }; })(),
      custom: structuredClone(this.custom),
      path: null, budget: p.budget, rules: { negative_rounding: this.rounding }, notes: this.notes,
      session: { hp: this.session.hp ?? Math.floor(tot.HP), hp_max: Math.floor(tot.HP), fp: this.session.fp ?? Math.floor(tot.FP), fp_max: Math.floor(tot.FP) },
    };
  }

  private skill(a: Record<string, unknown>) { const s = this.skills[a.index as number]; if (!s) throw new Error('no such skill'); return s; }
  private tech(a: Record<string, unknown>) { const t = this.techniques[a.index as number]; if (!t) throw new Error('no such technique'); return t; }

  private exec(cmd: string, a: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'new_character': this.session = { hp: null, fp: null }; this.notes = ''; this.ledger = [{ id: 1, amount: 150, kind: 'starting' }]; this.disLimit = null; this.custom = {}; this.levels = {}; this.traits = []; this.skills = []; this.techniques = []; this.spells = []; this.equipment = []; this.weapons = unarmed(); this.name = (a.name as string) ?? 'New character'; return this.view();
      case 'get_sheet': return this.view();
      case 'set_negative_rounding': {
        if (a.mode !== 'toward_positive' && a.mode !== 'by_magnitude') throw new Error(`unknown rounding mode ${a.mode}`);
        this.rounding = a.mode; return this.view();
      }
      case 'add_ledger_entry': {
        const id = Math.max(0, ...this.ledger.map((x) => x.id)) + 1;
        this.ledger.push({ id, ...normalizeLedger(a.entry as LedgerInput) }); return this.view();
      }
      case 'update_ledger_entry': {
        const i = this.ledger.findIndex((x) => x.id === a.id); if (i < 0) throw new Error('no such ledger entry');
        this.ledger[i] = { id: a.id as number, ...normalizeLedger(a.entry as LedgerInput) }; return this.view();
      }
      case 'remove_ledger_entry': {
        const i = this.ledger.findIndex((x) => x.id === a.id); if (i < 0) throw new Error('no such ledger entry');
        this.ledger.splice(i, 1); return this.view();
      }
      case 'set_disadvantage_limit': {
        const l = a.limit as number | null; if (l !== null && l < 0) throw new Error('limit must not be negative');
        this.disLimit = l; return this.view();
      }
      case 'save_custom': {
        const kind = a.kind as CustomKind; if (!['trait', 'skill', 'spell', 'equipment'].includes(kind)) throw new Error(`unknown custom kind '${kind}'`);
        const d = { ...(a.def as Record<string, unknown>) };
        if (!String(d.name ?? '').trim()) throw new Error('a custom entry needs a name');
        const all = [...(this.custom.traits ?? []), ...(this.custom.skills ?? []), ...(this.custom.spells ?? []), ...(this.custom.equipment ?? [])].map((x) => x.id);
        if (!String(d.id ?? '').trim()) { let n = 1; while (all.includes(`custom-${kind}-${n}`)) n++; d.id = `custom-${kind}-${n}`; }
        if (kind === 'trait') d.features ??= [];
        const list = this.customList<{ id: string }>(kind); const i = list.findIndex((x) => x.id === d.id);
        if (i >= 0) list[i] = d as { id: string }; else list.push(d as { id: string });
        const plain = { ...d, description: null };
        const sheet = kind === 'trait' ? this.traits : kind === 'skill' ? this.skills : kind === 'spell' ? this.spells : [];
        for (const x of sheet as { def: { id: string } }[]) if (x.def.id === d.id) x.def = structuredClone(plain) as unknown as typeof x.def;
        return this.view();
      }
      case 'remove_custom': {
        const list = this.customList<{ id: string }>(a.kind as CustomKind); const i = list.findIndex((x) => x.id === a.id);
        if (i < 0) throw new Error('no such custom entry'); list.splice(i, 1); return this.view();
      }
      case 'export_custom_plugin': throw new Error('Экспорт в плагин доступен только в приложении Tauri');
      case 'set_notes': { const t = a.text as string; this.notes = t.trim() ? t : ''; return this.view(); }
      case 'set_name': this.name = a.name as string; return this.view();
      case 'set_attribute': this.levels[a.key as string] = a.level as number; return this.view();
      case 'add_trait': {
        const def = this.library().find((l) => l.id === a.id); if (!def) throw new Error('unknown trait');
        this.traits.push({ def, levels: (a.levels as number) ?? (def.cost_per_level ? Math.max(1, def.min_levels ?? 1) : 0), source: def.plugin, modifiers: [] }); return this.view();
      }
      case 'set_trait_levels': this.traits[a.index as number].levels = a.levels as number; return this.view();
      case 'set_trait_explanation': {
        const t = this.traits[a.index as number]; if (!t) throw new Error('no such trait');
        const text = a.text as string | null; t.explanation = text && text.trim() ? text : undefined; return this.view();
      }
      case 'remove_trait': if (!this.traits[a.index as number]) throw new Error('no such trait'); this.traits.splice(a.index as number, 1); return this.view();
      case 'list_library': return this.library();
      case 'list_skill_library': return this.skillLibrary();
      case 'list_spell_library': return this.spellLibrary();
      case 'add_spell': {
        const def = this.spellLibrary().find((d) => d.id === a.id); if (!def) throw new Error(`unknown spell '${a.id}'`);
        this.spells.push({ def, points: Math.max(0, (a.points as number) ?? 1) }); return this.view();
      }
      case 'set_spell_points': this.spell(a).points = Math.max(0, a.points as number); return this.view();
      case 'set_spell_explanation': { const t = a.text as string | null; this.spell(a).explanation = t?.trim() ? t : undefined; return this.view(); }
      case 'remove_spell': this.spell(a); this.spells.splice(a.index as number, 1); return this.view();
      case 'list_modifier_library': return this.modifierLibrary();
      case 'add_trait_modifier': {
        const def = MODIFIER_DEFS.find((d) => d.id === a.id); if (!def) throw new Error(`unknown modifier '${a.id}'`);
        this.trait(a.traitIndex).modifiers.push({ def }); return this.view();
      }
      case 'remove_trait_modifier': {
        const t = this.trait(a.traitIndex); if (!t.modifiers[a.index as number]) throw new Error('no such modifier');
        t.modifiers.splice(a.index as number, 1); return this.view();
      }
      case 'set_modifier_explanation': {
        const m = this.trait(a.traitIndex).modifiers[a.index as number]; if (!m) throw new Error('no such modifier');
        const t = a.text as string | null; m.explanation = t?.trim() ? t : undefined; return this.view();
      }
      case 'add_skill': {
        const def = this.skillLibrary().skills.find((d) => d.id === a.id); if (!def) throw new Error('unknown skill');
        const sp = a.specialization as string | null | undefined;
        this.skills.push({ def, points: Math.max(0, (a.points as number) ?? 1), specialization: sp?.trim() ? sp : null }); return this.view();
      }
      case 'set_skill_points': this.skill(a).points = Math.max(0, a.points as number); return this.view();
      case 'set_skill_specialization': { const sp = a.specialization as string | null; this.skill(a).specialization = sp?.trim() ? sp : null; return this.view(); }
      case 'set_skill_explanation': { const t = a.text as string | null; this.skill(a).explanation = t?.trim() ? t : undefined; return this.view(); }
      case 'remove_skill': this.skill(a); this.skills.splice(a.index as number, 1); return this.view();
      case 'add_technique': {
        const def = TECHNIQUE_DEFS.find((d) => d.id === a.id); if (!def) throw new Error('unknown technique');
        this.techniques.push({ def, points: Math.max(0, (a.points as number) ?? 0) }); return this.view();
      }
      case 'set_technique_points': this.tech(a).points = Math.max(0, a.points as number); return this.view();
      case 'set_technique_explanation': { const t = a.text as string | null; this.tech(a).explanation = t?.trim() ? t : undefined; return this.view(); }
      case 'remove_technique': this.tech(a); this.techniques.splice(a.index as number, 1); return this.view();
      case 'list_equipment_library': return this.equipmentLibrary();
      case 'add_equipment': {
        const def = this.equipmentLibrary().find((d) => d.id === a.id); if (!def) throw new Error('unknown item');
        const { description: _d, ...rest } = def; void _d;
        const e: EquipmentItem = { ...rest, quantity: Math.max(1, (a.quantity as number) ?? 1), source: 'basic-set-sample' };
        const parent = a.parent as number[] | null | undefined;
        if (parent?.length) { const c = this.item({ path: parent }); if (!c.container) throw new Error(`${c.name} is not a container`); (c.contents ??= []).push(e); }
        else this.equipment.push(e);
        for (const w of WEAPON_DEFS) if (w.equipment === def.id && !this.weapons.some((x) => x.def.id === w.id)) this.weapons.push({ def: w });
        return this.view();
      }
      case 'set_equipment_quantity': this.item(a).quantity = Math.max(0, a.quantity as number); return this.view();
      case 'set_equipment_carried': this.item(a).carried = !!a.carried; return this.view();
      case 'set_equipment_explanation': { const t = a.text as string | null; this.item(a).explanation = t?.trim() ? t : undefined; return this.view(); }
      case 'remove_equipment': {
        const path = a.path as number[]; this.item(a);
        const list = path.length > 1 ? this.item({ path: path.slice(0, -1) }).contents! : this.equipment;
        list.splice(path[path.length - 1], 1); this.pruneWeapons(); return this.view();
      }
      case 'list_weapon_library': return this.weaponLibrary();
      case 'add_weapon': {
        const def = WEAPON_DEFS.find((d) => d.id === a.id); if (!def) throw new Error('unknown weapon');
        if (this.weapons.some((w) => w.def.id === def.id)) throw new Error(`weapon ${def.id} is already on the sheet`);
        this.weapons.push({ def }); return this.view();
      }
      case 'remove_weapon': if (!this.weapons[a.index as number]) throw new Error('no such weapon'); this.weapons.splice(a.index as number, 1); return this.view();
      case 'set_weapon_explanation': {
        const w = this.weapons[a.index as number]; if (!w) throw new Error('no such weapon');
        const t = a.text as string | null; w.explanation = t?.trim() ? t : undefined; return this.view();
      }
      case 'plugin_status': case 'reload_plugins': return { dir: null, plugins: ['basic-set-sample (встроенная копия)', 'lang-ru (встроенная копия)'], errors: [], traits: LIBRARY.length, skills: SKILL_DEFS.length + TECHNIQUE_DEFS.length, equipment: EQUIPMENT_DEFS.length, weapons: WEAPON_DEFS.length, spells: SPELL_DEFS.length, modifiers: MODIFIER_DEFS.length };
      case 'get_locales': return this.locales();
      case 'set_locale': if ((a.locale as string)?.trim()) this.locale = a.locale as string; this.originalOnly = !!a.originalOnly; return this.view();
      case 'roll': {
        const rng = a.seed != null ? new SplitMix(a.seed as number) : this.rng;
        const dice = [rng.d6(), rng.d6(), rng.d6()] as [number, number, number];
        const roll = dice[0] + dice[1] + dice[2]; const effective = (a.target as number) + ((a.modifier as number) ?? 0);
        const r: RollResult = { target: a.target as number, modifier: (a.modifier as number) ?? 0, effective, dice, roll, margin: effective - roll, outcome: outcome(roll, effective) };
        return r;
      }
      case 'roll_damage': return damageRoll(a.expr as string, !!a.crushing, a.seed != null ? new SplitMix(a.seed as number) : this.rng);
      case 'set_current': {
        const st = a.stat as string; const v = a.value as number | null;
        if (st !== 'HP' && st !== 'FP') throw new Error(`unknown session stat '${st}'`);
        const ss = this.view().session!; const max = st === 'HP' ? ss.hp_max : ss.fp_max;
        const c = v == null ? max : Math.min(max, Math.max(st === 'HP' ? -5 * max : -max, v));
        this.session[st === 'HP' ? 'hp' : 'fp'] = c === max ? null : c; return this.view();
      }
      case 'load': case 'save': throw new Error('Файлы доступны только в приложении Tauri');
      default: throw new Error(`unknown command ${cmd}`);
    }
  }
}
