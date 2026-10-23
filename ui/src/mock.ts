// Browser-preview backend. Used ONLY when the UI runs outside Tauri (plain Vite dev),
// so designers can work on the UI without the Rust app. The source of truth for all
// rules is the Rust crate `gurps-rules`; this is a simplified mirror of it.
import type {
  LibraryEntry, LocaleInfo, LocalizedText, RollResult, SheetView, StatRow, TraitDef, TraitView, Value, Outcome,
} from './types';

interface MockTrait { def: TraitDef; levels: number; source: string; explanation?: string }

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
    description: 'Example trait: each level gives **+1 ST** (and everything derived from ST).' },
  { plugin: 'basic-set-sample', id: 'lifting-st', name: 'Lifting ST', category: 'advantage', cost: 0, cost_per_level: 3, page: 'B65', features: [{ type: 'lifting_st_bonus', amount: 1, per_level: true }],
    description: 'Extra strength **only for lifting and carrying**.\nRaises Basic Lift, but not damage or HP.' },
  { plugin: 'basic-set-sample', id: 'combat-reflexes', name: 'Combat Reflexes', category: 'advantage', cost: 15, cost_per_level: 0, page: 'B43', features: [],
    description: 'Fast reactions in a fight:\n- rarely caught off guard;\n- small bonuses to defence and readiness.' },
  { plugin: 'basic-set-sample', id: 'overconfidence', name: 'Overconfidence', category: 'disadvantage', cost: -5, cost_per_level: 0, page: 'B148', features: [],
    description: 'Character believes they can handle anything and *underestimates risk*.' },
];
const LIBRARY = RAW_LIBRARY;

/** Mirror of plugins/lang-ru/locales/ru/basic-set-sample.ftl (subset). Keys: `<kind>-<id>`. */
export const CATALOG: Record<string, Record<string, { name?: string; description?: string }>> = {
  ru: {
    'trait-increased-st-example': { name: 'Повышенная ST (пример)', description: 'Пример черты: каждый уровень даёт **+1 ST** (и всё, что от ST зависит).' },
    'trait-lifting-st': { name: 'Подъёмная сила', description: 'Добавочная сила **только для подъёма и переноски** грузов.\nВлияет на базовый груз, но не на урон и не на HP.' },
    'trait-combat-reflexes': { name: 'Боевые рефлексы', description: 'Быстрая реакция в бою:\n- реже теряется от неожиданности;\n- небольшие бонусы к защите и готовности.' },
    'trait-overconfidence': { name: 'Самоуверенность', description: 'Персонаж уверен, что справится с чем угодно, и *недооценивает риск*.' },
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

export class MockBackend {
  name = 'Пример (браузер)';
  levels: Record<string, number> = {};
  traits: MockTrait[] = [];
  locale = 'ru';
  originalOnly = false;

  private tr(id: string) {
    if (this.originalOnly) return undefined;
    return (CATALOG[this.locale] ?? CATALOG[this.locale.split(/[-_]/)[0]])?.[`trait-${id}`];
  }
  private plugText(def: TraitDef) { return this.tr(def.id)?.description ?? localize(def.description, this.originalOnly ? 'en' : this.locale); }
  private shown(t: MockTrait) { const n = this.tr(t.def.id)?.name; return n ? (t.def.cost_per_level ? `${n} ${t.levels}` : n) : undefined; }
  library(): LibraryEntry[] {
    return LIBRARY.map((e) => ({ ...e, explanation: this.plugText(e), translated_name: this.tr(e.id)?.name ?? null }));
  }
  locales(): LocaleInfo { return { available: [...new Set([...Object.keys(CATALOG), 'en'])].sort(), locale: this.locale, original_only: this.originalOnly }; }

  private name_(t: MockTrait) { return t.def.cost_per_level ? `${t.def.name} ${t.levels}` : t.def.name; }
  private cost(t: MockTrait) { return t.def.cost + t.def.cost_per_level * t.levels; }

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
    if (lv) v.contributions.push({ source: { kind: 'purchased' }, amount: lv * step, reason: `purchased (${lv * COST[key]} pts)` });
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
      disadvantages: cat('disadvantage'), quirks: cat('quirk'), skills: 0, total: 0, budget: 150, unspent: 0 };
    p.total = p.attributes + p.secondary + p.advantages + p.perks + p.disadvantages + p.quirks;
    p.unspent = p.budget - p.total;
    const traits: TraitView[] = this.traits.map((t, index) => ({ index, id: t.def.id, name: this.shown(t) ?? this.name_(t),
      original_name: this.shown(t) ? this.name_(t) : null, category: t.def.category,
      levels: t.levels, leveled: !!t.def.cost_per_level, min_level: t.def.min_levels ?? (t.def.cost_per_level ? 1 : 0), cost: this.cost(t), page: t.def.page, source: t.source,
      explanation: t.explanation?.trim() ? t.explanation : this.plugText(t.def),
      default_explanation: this.plugText(t.def), custom_explanation: !!t.explanation?.trim() }));
    return {
      sheet: { name: this.name, attributes, secondary, basic_lift: this.row('BasicLift', blv),
        damage: { thrust: '—', swing: '—', based_on_st: tot.ST }, points: p },
      traits, issues: p.unspent < 0 ? [{ severity: 'warning', message: `over budget by ${-p.unspent} points` }] : [],
      path: null, budget: 150,
    };
  }

  invoke(cmd: string, a: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'new_character': this.levels = {}; this.traits = []; this.name = (a.name as string) ?? 'New character'; return this.view();
      case 'get_sheet': return this.view();
      case 'set_name': this.name = a.name as string; return this.view();
      case 'set_attribute': this.levels[a.key as string] = a.level as number; return this.view();
      case 'add_trait': {
        const def = LIBRARY.find((l) => l.id === a.id); if (!def) throw new Error('unknown trait');
        this.traits.push({ def, levels: (a.levels as number) ?? (def.cost_per_level ? 1 : 0), source: def.plugin }); return this.view();
      }
      case 'set_trait_levels': this.traits[a.index as number].levels = a.levels as number; return this.view();
      case 'set_trait_explanation': {
        const t = this.traits[a.index as number]; if (!t) throw new Error('no such trait');
        const text = a.text as string | null; t.explanation = text && text.trim() ? text : undefined; return this.view();
      }
      case 'remove_trait': this.traits.splice(a.index as number, 1); return this.view();
      case 'list_library': return this.library();
      case 'get_locales': return this.locales();
      case 'set_locale': if ((a.locale as string)?.trim()) this.locale = a.locale as string; this.originalOnly = !!a.originalOnly; return this.view();
      case 'roll': {
        const dice = [0, 0, 0].map(() => 1 + Math.floor(Math.random() * 6)) as [number, number, number];
        const roll = dice[0] + dice[1] + dice[2]; const effective = (a.target as number) + ((a.modifier as number) ?? 0);
        const r: RollResult = { target: a.target as number, modifier: (a.modifier as number) ?? 0, effective, dice, roll, margin: effective - roll, outcome: outcome(roll, effective) };
        return r;
      }
      case 'load': case 'save': throw new Error('Файлы доступны только в приложении Tauri');
      default: throw new Error(`unknown command ${cmd}`);
    }
  }
}
