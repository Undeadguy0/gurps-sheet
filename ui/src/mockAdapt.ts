// Boundary between the preview backend (legacy shapes, mockTypes.ts) and the app types
// (types.ts, mirrors the v2 model: sid + kind/cost{...}, catalog-style fields).
import type * as L from './mockTypes';
import type * as N from './types';

const KIND: Record<L.TraitCategory, N.TraitKind> = { advantage: 'adv', disadvantage: 'disadv', perk: 'perk', quirk: 'quirk' };
const CATEGORY: Record<N.TraitKind, L.TraitCategory> = { adv: 'advantage', disadv: 'disadvantage', perk: 'perk', quirk: 'quirk' };
type Meta = { plugin: string; explanation?: string | null; translated_name?: string | null };
const meta = (e: Meta, id: string) => ({ plugin: e.plugin, id, explanation: e.explanation, translated_name: e.translated_name });

export const trait = (d: L.TraitDef): N.TraitDef => ({
  sid: d.id, name: d.name, kind: KIND[d.category],
  cost: { type: d.cost_per_level ? 'base_plus_level' : 'fixed', base: d.cost, per_level: d.cost_per_level || null, max_level: d.max_levels, min_level: d.min_levels },
  features: d.features, page: d.page, description: d.description,
});
export const skill = (d: L.SkillDef): N.SkillDef => ({
  sid: d.id, name: d.name, attribute: d.attribute, difficulty: d.difficulty,
  defaults: (d.defaults ?? []).map((x) => ({ attribute: 'attribute' in x ? x.attribute : null, skill: 'skill' in x ? x.skill : null, mod: x.modifier })),
  spec_required: d.specialization_required, specializations: d.specializations, page: d.page, description: d.description,
});
export const technique = (d: L.TechniqueDef): N.TechniqueDef => ({
  sid: d.id, name: d.name, parent_skill: d.skill, difficulty: d.difficulty, default_mod: d.modifier, max_mod: d.max, page: d.page, description: d.description,
});
export const spell = (d: L.SpellDef): N.SpellDef => ({
  sid: d.id, name: d.name, difficulty: d.difficulty ?? 'H', colleges: d.college ?? [], prereqs: d.prerequisites, page: d.page, description: d.description,
});
export const modifier = (d: L.ModifierDef): N.ModifierDef => ({
  sid: d.id, name: d.name, type: d.cost_percent >= 0 ? 'enh' : 'lim', value_kind: '%', amount: Math.abs(d.cost_percent), page: d.page, description: d.description,
});
export const item = (d: L.EquipmentDef): N.ItemDef => ({
  sid: d.id, name: d.name, weight: d.weight, cost: d.cost, container: d.container, page: d.page, description: d.description,
});

/** Converts one preview-backend result to the app shape. */
export function out(cmd: string, r: unknown): unknown {
  switch (cmd) {
    case 'list_library': return (r as L.LibraryEntry[]).map((e) => ({ ...trait(e), ...meta(e, e.id) }));
    case 'list_spell_library': return (r as L.SpellLibEntry[]).map((e) => ({ ...spell(e), ...meta(e, e.id) }));
    case 'list_modifier_library': return (r as L.ModifierLibEntry[]).map((e) => ({ ...modifier(e), ...meta(e, e.id) }));
    case 'list_equipment_library': return (r as L.EquipmentLibEntry[]).map((e) => ({ ...item(e), ...meta(e, e.id) }));
    case 'list_skill_library': {
      const s = r as L.SkillLibrary;
      return {
        skills: s.skills.map((e) => ({ ...skill(e), ...meta(e, e.id) })),
        techniques: s.techniques.map((e) => ({ ...technique(e), ...meta(e, e.id), skill_name: e.skill_name })),
      };
    }
  }
  if (r && typeof r === 'object' && 'sheet' in r && 'traits' in r) {
    const v = r as L.SheetView;
    return {
      ...v,
      traits: v.traits.map(({ category, levels, ...t }) => ({ ...t, kind: KIND[category], level: levels })),
      custom: v.custom && {
        traits: v.custom.traits?.map(trait), skills: v.custom.skills?.map(skill),
        spells: v.custom.spells?.map(spell), items: v.custom.equipment?.map(item),
      },
    };
  }
  return r;
}

/** Converts a `save_custom` definition from the app shape to the preview backend's. */
export function customIn(kind: string, d: Record<string, unknown>): Record<string, unknown> {
  const { sid, ...rest } = d;
  const base = { ...rest, id: sid ?? '' };
  if (kind === 'trait') {
    const c = (d.cost ?? {}) as N.TraitCost;
    return { ...base, category: CATEGORY[(d.kind as N.TraitKind) ?? 'adv'], cost: c.base ?? 0, cost_per_level: c.per_level ?? 0, max_levels: c.max_level ?? null, features: d.features ?? [] };
  }
  if (kind === 'skill') return { ...base, specialization_required: !!d.spec_required };
  if (kind === 'spell') return { ...base, college: d.colleges ?? [] };
  return base;
}
