// Russian plural agreement via Intl.PluralRules('ru'): one / few / many (+ 'other' for fractions).
const rules = new Intl.PluralRules('ru');

export type Forms = { one: string; few: string; many: string; other?: string };

export function pluralForm(n: number, f: Forms): string {
  const cat = rules.select(n);
  if (cat === 'one') return f.one;
  if (cat === 'few') return f.few;
  if (cat === 'many') return f.many;
  return f.other ?? f.few; // fractions: "1,5 очка"
}

/** "5 очков" */
export const plural = (n: number, f: Forms) => `${n} ${pluralForm(n, f)}`;

export const POINTS: Forms = { one: 'очко', few: 'очка', many: 'очков' };
export const STEPS: Forms = { one: 'шаг', few: 'шага', many: 'шагов' };
export const SOURCES: Forms = { one: 'источник', few: 'источника', many: 'источников' };
export const POUNDS: Forms = { one: 'фунт', few: 'фунта', many: 'фунтов' };
export const ITEMS: Forms = { one: 'предмет', few: 'предмета', many: 'предметов' };

export const pts = (n: number) => plural(n, POINTS);
/** Past-tense neuter verb agrees with any count of "очко": "Осталось 1 очко / 2 очка / 5 очков". */
export const remaining = (n: number) => n < 0 ? `Перерасход: ${pts(-n)}` : `Осталось ${pts(n)}`;
