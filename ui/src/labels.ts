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
