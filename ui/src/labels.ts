import type { Feature } from './types';
// UI strings (Russian) and stat names. GURPS abbreviations are kept as in the Basic Set.
export const STAT_LABEL: Record<string, string> = {
  ST: 'ST — Сила', DX: 'DX — Ловкость', IQ: 'IQ — Интеллект', HT: 'HT — Здоровье',
  HP: 'HP — Единицы жизни', Will: 'Will — Воля', Per: 'Per — Восприятие', FP: 'FP — Усталость',
  BasicSpeed: 'Basic Speed — Скорость', BasicMove: 'Basic Move — Движение', BasicLift: 'Basic Lift — Базовый груз',
};
export const CATEGORY_LABEL: Record<string, string> = {
  advantage: 'Преимущество', perk: 'Перк', disadvantage: 'Недостаток', quirk: 'Причуда',
};
export const OUTCOME_LABEL: Record<string, string> = {
  critical_success: 'Крит. успех', success: 'Успех', failure: 'Провал', critical_failure: 'Крит. провал',
};
/** Stats that can be rolled against with 3d6. */
export const ROLLABLE = new Set(['ST', 'DX', 'IQ', 'HT', 'Will', 'Per']);
export const fmt = (v: number) => (Number.isInteger(v) ? String(v) : String(+v.toFixed(2)));
export const signed = (v: number) => (v > 0 ? `+${fmt(v)}` : fmt(v));

/** Readable description of a modelled trait effect, e.g. "ST +1 за уровень". */
export function featureText(f: Feature): string {
  const per = f.per_level ? ' за уровень' : '';
  const amt = signed(f.amount);
  switch (f.type) {
    case 'attribute_bonus': return `${STAT_LABEL[f.target] ?? f.target}: ${amt}${per}`;
    case 'lifting_st_bonus': return `ST для подъёма и переноски (базовый груз): ${amt}${per}`;
    case 'striking_st_bonus': return `ST для урона (толчок/замах): ${amt}${per}`;
    case 'skill_bonus': return `Навыки (${f.skills.join(', ')}): ${amt}${per}`;
    case 'defense_bonus': {
      const ru: Record<string, string> = { dodge: 'уклонение', parry: 'парирование', block: 'блок' };
      const which = f.defenses?.length ? f.defenses.map((d) => ru[d] ?? d).join(', ') : 'уклонение, парирование и блок';
      return `Активные защиты (${which}): ${amt}${per}`;
    }
  }
}
