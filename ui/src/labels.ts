import type { ItemDef, ModifierDef, TraitCost } from './types';
// UI strings (Russian) and stat names. GURPS abbreviations are kept as in the Basic Set.
export const STAT_LABEL: Record<string, string> = {
  ST: 'ST — Сила', DX: 'DX — Ловкость', IQ: 'IQ — Интеллект', HT: 'HT — Здоровье',
  HP: 'HP — Единицы жизни', Will: 'Will — Воля', Per: 'Per — Восприятие', FP: 'FP — Усталость',
  BasicSpeed: 'Basic Speed — Скорость', BasicMove: 'Basic Move — Движение', BasicLift: 'Basic Lift — Базовый груз',
};
export const CATEGORY_LABEL: Record<string, string> = {
  adv: 'Преимущество', perk: 'Перк', disadv: 'Недостаток', quirk: 'Причуда',
};
export const OUTCOME_LABEL: Record<string, string> = {
  critical_success: 'Крит. успех', success: 'Успех', failure: 'Провал', critical_failure: 'Крит. провал',
};
/** Stats that can be rolled against with 3d6. */
export const ROLLABLE = new Set(['ST', 'DX', 'IQ', 'HT', 'Will', 'Per']);
export const fmt = (v: number) => (Number.isInteger(v) ? String(v) : String(+v.toFixed(2)));
export const signed = (v: number) => (v > 0 ? `+${fmt(v)}` : fmt(v));

const FEATURE_LABEL: Record<string, string> = {
  attribute_bonus: 'Атрибут', skill_bonus: 'Навык', spell_bonus: 'Заклинания', dr_bonus: 'DR',
  damage_bonus: 'Урон', defense_bonus: 'Активные защиты', lifting_st_bonus: 'ST для подъёма', striking_st_bonus: 'ST для урона',
};
/** Readable description of a catalog feature (gurps-calc JSON), e.g. "Навык (outdoor_a): +1 за уровень". */
export function featureText(raw: unknown): string {
  const f = (raw ?? {}) as Record<string, unknown>;
  const kind = String(f.kind ?? f.type ?? '?');
  const target = ['attribute', 'target', 'skill', 'skills', 'college', 'colleges', 'attack_mode', 'locations', 'defenses']
    .map((k) => f[k]).filter((v) => v != null && v !== '').map((v) => (Array.isArray(v) ? v.join(', ') : STAT_LABEL[String(v)] ?? String(v)));
  const amt = typeof f.amount === 'number' ? `: ${signed(f.amount)}` : f.damage_type ? `: ${String(f.damage_type)}` : '';
  return `${FEATURE_LABEL[kind] ?? kind}${target.length ? ` (${target.join('; ')})` : ''}${amt}${f.per_level ? ' за уровень' : ''}`;
}

/** Trait cost as text: "10", "5/ур.", "2 + 3/ур.". */
export function costText(c: TraitCost): string {
  return c.per_level ? `${c.base ? c.base + ' + ' : ''}${c.per_level}/ур.` : String(c.base);
}

/** Short item summary: DR, DB, attack modes. */
export function itemStats(d: ItemDef): string {
  const parts: string[] = [];
  const dr = Math.max(...(d.armor ?? []).map((a) => a.dr ?? 0), 0);
  if (dr) parts.push(`DR ${dr}`);
  if (d.shield) parts.push(`DB ${d.shield.db}`);
  for (const m of d.attack_modes ?? []) {
    const dm = m.damage; const base = dm.base ?? (dm.dice != null ? `${dm.dice}d` : '');
    parts.push(`${m.name ?? ''} ${base}${dm.mod ? signed(dm.mod) : ''} ${dm.type ?? ''}`.trim());
  }
  return parts.join('; ');
}

/** Modifier percent (as gurps_core::ModifierDef::percent): limitations are negative. */
export function modPercent(d: ModifierDef): number {
  if (d.value_kind !== '%') return 0;
  const a = Math.round(Math.abs(d.amount));
  return d.type === 'lim' ? -a : a;
}
