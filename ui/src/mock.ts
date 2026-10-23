// Browser-preview backend. Used ONLY when the UI runs outside Tauri (plain Vite dev),
// so designers can work on the UI without the Rust app. The source of truth for all
// rules is the Rust crate `gurps-rules`; this is a simplified mirror of it.
import type {
  LibraryEntry, LocaleInfo, LocalizedText, RollResult, SheetView, StatRow, TraitDef, TraitView, Value, Outcome,
  SkillDef, SkillLibrary, SkillView, TechniqueDef, TechniqueView,
} from './types';
import { SKILL_CATALOG_RU, SKILL_DEFS, TALENT_DEF, TECHNIQUE_DEFS } from './mockSkills';
import { computeSkills, computeTechniques, signedInt } from './skills';

interface MockSkill { def: SkillDef; points: number; specialization?: string | null; explanation?: string }
interface MockTech { def: TechniqueDef; points: number; explanation?: string }
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
    description: "A sample trait used to test the sheet: each level simply adds +1 to ST, and everything calculated from ST follows automatically (Hit Points, Basic Lift, thrust and swing damage).\n\nUse it to see how the sheet explains where a number comes from: the ST row will list the trait as a separate contribution next to the purchased levels." },
  { plugin: 'basic-set-sample', id: 'lifting-st', name: 'Lifting ST', category: 'advantage', cost: 0, cost_per_level: 3, page: 'B65', features: [{ type: 'lifting_st_bonus', amount: 1, per_level: true }],
    description: "Extra muscle that only counts when you lift, carry, push or drag something. Each level raises the ST used to work out Basic Lift, so the character can haul heavier loads before being slowed down. It does not make blows hit harder and does not add Hit Points.\n\nIn play it suits porters, labourers, climbers who carry gear and anyone who needs to move a lot of weight cheaply. Combine it with ordinary ST if you also want stronger attacks." },
  { plugin: 'basic-set-sample', id: 'combat-reflexes', name: 'Combat Reflexes', category: 'advantage', cost: 15, cost_per_level: 0, page: 'B43', features: [],
    description: "The character reacts to danger almost instantly. Surprise rarely freezes them, they recover from being stunned faster than most, and they are a little better at dodging, blocking and parrying. Fear and shock also have a weaker grip on them.\n\nIn play this is the classic trait of veterans, bodyguards and duellists: they act first in an ambush and keep their head when things go wrong. The defence and fear bonuses are not calculated by the sheet yet; apply them at the table." },
  { plugin: 'basic-set-sample', id: 'overconfidence', name: 'Overconfidence', category: 'disadvantage', cost: -5, cost_per_level: 0, page: 'B148', features: [],
    description: "The character believes they can handle anything and consistently underestimates risk. When a sensible person would back off, plan or ask for help, they tend to charge ahead instead, unless they manage to resist the urge.\n\nIt makes for bold, memorable characters (young heroes, braggarts, star pilots), but the player should actually play the recklessness. Others may find them inspiring or insufferable." },
];
RAW_LIBRARY.push({ plugin: 'basic-set-sample', ...TALENT_DEF } as Omit<LibraryEntry, 'explanation'>);
const LIBRARY = RAW_LIBRARY;

/** Mirror of plugins/lang-ru/locales/ru/basic-set-sample.ftl (subset). Keys: `<kind>-<id>`. */
export const CATALOG: Record<string, Record<string, { name?: string; description?: string }>> = {
  ru: {
    'trait-increased-st-example': { name: 'Повышенная ST (пример)', description: "Пробная черта для проверки листа: каждый уровень просто добавляет +1 к ST, а всё, что считается от ST, меняется само (HP, базовый груз, урон толчком и замахом).\n\nУдобна, чтобы посмотреть, как лист объясняет происхождение чисел: в строке ST черта будет отдельным слагаемым рядом с купленными уровнями." },
    'trait-lifting-st': { name: 'Подъёмная сила', description: "Дополнительная мускульная сила, которая работает только при подъёме, переноске, толкании или волочении. Каждый уровень повышает ST, от которой считается базовый груз, поэтому персонаж может тащить больше, не замедляясь. На силу ударов и на HP она не влияет.\n\nПодходит носильщикам, грузчикам, альпинистам со снаряжением и всем, кому нужно дёшево таскать тяжести. Если нужны и сильные удары — сочетайте с обычной ST." },
    'trait-combat-reflexes': { name: 'Боевые рефлексы', description: "Персонаж реагирует на опасность почти мгновенно. Внезапность редко сбивает его с толку, из оглушения он выходит быстрее других и чуть лучше уклоняется, блокирует и парирует. Страх и шок тоже действуют на него слабее.\n\nЭто типичная черта ветеранов, телохранителей и дуэлянтов: в засаде они действуют первыми и не теряют голову, когда всё идёт плохо. Бонусы к защите и против страха лист пока не считает — учитывайте их за столом." },
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

export class MockBackend {
  name = 'Пример (браузер)';
  levels: Record<string, number> = {};
  traits: MockTrait[] = [];
  locale = 'ru';
  originalOnly = false;

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
  skillLibrary(): SkillLibrary {
    return {
      skills: SKILL_DEFS.map((d) => ({ ...d, plugin: 'basic-set-sample', explanation: this.text('skill', d.id, d.description), translated_name: this.tr(d.id, 'skill')?.name ?? null })),
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
    const [skills, techniques, skillPts] = this.skillViews(tot);
    p.skills = skillPts;
    p.total = p.attributes + p.secondary + p.advantages + p.perks + p.disadvantages + p.quirks + p.skills;
    p.unspent = p.budget - p.total;
    const traits: TraitView[] = this.traits.map((t, index) => ({ index, id: t.def.id, name: this.shown(t) ?? this.name_(t),
      original_name: this.shown(t) ? this.name_(t) : null, category: t.def.category,
      levels: t.levels, leveled: !!t.def.cost_per_level, min_level: t.def.min_levels ?? (t.def.cost_per_level ? 1 : 0), cost: this.cost(t), page: t.def.page, source: t.source,
      explanation: t.explanation?.trim() ? t.explanation : this.plugText(t.def),
      default_explanation: this.plugText(t.def), custom_explanation: !!t.explanation?.trim() }));
    return {
      sheet: { name: this.name, attributes, secondary, basic_lift: this.row('BasicLift', blv),
        damage: { thrust: '—', swing: '—', based_on_st: tot.ST }, points: p },
      traits, skills, techniques,
      issues: [...(p.unspent < 0 ? [{ severity: 'warning' as const, message: `over budget by ${-p.unspent} points` }] : []),
        ...this.skills.filter((s) => s.def.specialization_required && !s.specialization?.trim())
          .map((s) => ({ severity: 'warning' as const, message: `skill '${s.def.name}' needs a specialization` }))],
      path: null, budget: 150,
    };
  }

  private skill(a: Record<string, unknown>) { const s = this.skills[a.index as number]; if (!s) throw new Error('no such skill'); return s; }
  private tech(a: Record<string, unknown>) { const t = this.techniques[a.index as number]; if (!t) throw new Error('no such technique'); return t; }

  invoke(cmd: string, a: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'new_character': this.levels = {}; this.traits = []; this.skills = []; this.techniques = []; this.name = (a.name as string) ?? 'New character'; return this.view();
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
      case 'list_skill_library': return this.skillLibrary();
      case 'add_skill': {
        const def = SKILL_DEFS.find((d) => d.id === a.id); if (!def) throw new Error('unknown skill');
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
      case 'plugin_status': case 'reload_plugins': return { dir: null, plugins: ['basic-set-sample (встроенная копия)', 'lang-ru (встроенная копия)'], errors: [], traits: LIBRARY.length, skills: SKILL_DEFS.length + TECHNIQUE_DEFS.length };
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
