// Russian rendering of the engine's English explanation and validation lines
// (gurps-rules keeps them in English so the CLI and tests stay stable).
// Ordered regex rules; numbers with "pt/pts" go through the plural helper.
// Names inside lines (traits, skills, items) are plugin originals unless `name` maps them.
import { ref } from 'vue';
import { plural, POUNDS, pts, type Forms } from './plural';
import { LEVEL_LABEL_RU, LEVEL_NAMES } from './encumbrance';

const LEVELS_GEN: Forms = { one: 'уровня', few: 'уровней', many: 'уровней' };
// Decimals keep the dot, like the numbers elsewhere on the sheet.
const num = (s: string) => s;
const pounds = (s: string) => plural(+s, POUNDS).replace(/^[\d.]+/, num(s));
const level = (n: string) => LEVEL_LABEL_RU[LEVEL_NAMES.indexOf(n)] ?? n;

const ATTRS = new Set(['ST', 'DX', 'IQ', 'HT', 'Per', 'Will']);
type Name = (s: string) => string;
type Rule = [RegExp, (m: RegExpExecArray, name: Name, self: (l: string) => string) => string];

const RULES: Rule[] = [
  // values (attributes)
  [/^default$/, () => 'по умолчанию'],
  [/^base (\S+) \((.*)\)$/, (m, _n, t) => `база ${num(m[1])} (${t(m[2])})`],
  [/^([+-]\S+ )?purchased \((.+)\)$/, (m) => `${m[1] ?? ''}куплено (${m[2]})`],
  [/^([+-]\S+ )?from trait (.+?)(?: \(ST (\S+) for lifting\))?$/, (m, n) =>
    `${m[1] ?? ''}от черты «${n(m[2])}»${m[3] ? ` (ST ${m[3]} для подъёма)` : ''}`],
  // skills and techniques
  [/^no points and no default$/, () => 'нет очков и нет умолчания'],
  [/^default (.+?)([+-]\d+) = (-?\d+)$/, (m, n) =>
    `по умолчанию ${ATTRS.has(m[1]) ? m[1] : `«${n(m[1])}»`}${m[2]} = ${m[3]}`],
  [/^capped at (.+) = (-?\d+)$/, (m) => `не выше ${m[1]} = ${m[2]}`],
  [/^needs skill (.+)$/, (m, n) => `нужен навык «${n(m[1])}»`],
  // encumbrance
  [/^carried (\S+) lb, BL (\S+) lb$/, (m) => `при себе ${pounds(m[1])}, базовый груз ${pounds(m[2])}`],
  [/^≤ (\d+)×BL \((\S+) lb\) -> (\S+) \((\d)\)$/, (m) => `≤ ${m[1]}×БГ (${pounds(m[2])}) → ${level(m[3])} (${m[4]})`],
  [/^> 10×BL \((\S+) lb\): too heavy to move$/, (m) => `> 10×БГ (${pounds(m[1])}): слишком тяжело, двигаться нельзя`],
  [/^Move floor\((.+)\) = (\d+)$/, (m) => `Движение floor(${m[1]}) = ${m[2]}`],
  [/^Dodge floor\((.+?)\) \+ 3 +(?:- (\d+) )?= (-?\d+)$/, (m) => `Уклонение floor(${num(m[1])}) + 3${m[2] ? ` − ${m[2]}` : ''} = ${m[3]}`],
  // weapons and defences
  [/^floor\(Basic Speed\) \+ 3 = (-?\d+)(?: ([+-]\d+) \((.+)\))?$/, (m, n) =>
    `floor(Скорость) + 3 = ${m[1]}${m[2] ? ` ${m[2]} (${m[3].split(', ').map(n).join(', ')})` : ''}`],
  [/^encumbrance (-\d+) \((.+)\)$/, (m) => `нагрузка ${m[1]} (${level(m[2])})`],
  [/^Parry (.+)$/, (m, _n, t) => `Парирование ${t(m[1])}`],
  [/^Block (.+)$/, (m, _n, t) => `Блок ${t(m[1])}`],
  [/^(floor\(-?\d+\/2\) \+ 3.*) ([+-]\d+) \((.+)\)( = -?\d+)$/, (m, n) => `${m[1]} ${m[2]} (${m[3].split(', ').map(n).join(', ')})${m[4]}`],
  [/^technique (.+) (-?\d+)$/, (m, n) => `приём «${n(m[1])}» ${m[2]}`],
  [/^skill (.+?)([+-]\d+) = (-?\d+)$/, (m, n) => `навык «${n(m[1])}»${m[2]} = ${m[3]}`],
  [/^skill ([^']+) (-?\d+)$/, (m, n) => `навык «${n(m[1])}» ${m[2]}`],
  [/^no skill (.+) and no default$/, (m, n) => `нет навыка «${n(m[1])}» и умолчания`],
  [/^damage (\S+) = (thr|sw) (\S+) -> (\S+)$/, (m) => `урон ${m[1]} = ${m[2] === 'thr' ? 'толчок' : 'замах'} ${m[3]} → ${m[4]}`],
  [/^damage (\S+) = fixed -> (\S+)$/, (m) => `урон ${m[1]} (фиксированный) → ${m[2]}`],
  [/^damage (\S+) = unparsed -> (\S+)$/, (m) => `урон «${m[1]}» не разобран`],
  [/^no (.+) damage bonus: (\S+) < (\S+)$/, (m, n) => `нет бонуса к урону от «${n(m[1])}»: ${m[2]} < ${m[3]}`],
  [/^(.+) at (\S+) \(>= ([+-]\d+)\): ([+-]\d+) per die -> (\S+)$/, (m, n) =>
    `«${n(m[1])}» на уровне ${m[2]} (≥ ${m[3]}): ${m[4]} за кубик → ${m[5]}`],
  [/^cannot parry$/, () => 'парировать нельзя'],
  // validation
  [/^over budget by (.+)$/, (m) => `Бюджет превышен на ${m[1]}`],
  [/^carried (\S+) lb is more than 10×BL \((\S+) lb\): the character cannot move$/, (m) =>
    `при себе ${pounds(m[1])} — больше 10×БГ (${pounds(m[2])}): персонаж не может двигаться`],
  [/^trait '(.+)' needs at least (\d+) level\(s\)$/, (m, n) => `черте «${n(m[1])}» нужно не меньше ${plural(+m[2], LEVELS_GEN)}`],
  [/^trait '(.+)' exceeds max level (\d+)$/, (m, n) => `черта «${n(m[1])}» выше максимального уровня ${m[2]}`],
  [/^skill '(.+)' has negative points$/, (m, n) => `у навыка «${n(m[1])}» отрицательные очки`],
  [/^skill '(.+)': (.+) is not on the cost table \(1, 2, 4, 8, 12\.\.\.\); extra points are wasted$/, (m, n) =>
    `навык «${n(m[1])}»: ${m[2]} нет в таблице стоимости (1, 2, 4, 8, 12…); лишние очки пропадают`],
  [/^skill '(.+)' needs a specialization$/, (m, n) => `навыку «${n(m[1])}» нужна специализация`],
  [/^technique '(.+)' has negative points$/, (m, n) => `у приёма «${n(m[1])}» отрицательные очки`],
  [/^technique '(.+)' needs skill '(.+)'$/, (m, n) => `приёму «${n(m[1])}» нужен навык «${n(m[2])}»`],
];

const POST: [RegExp, (m: string, ...g: string[]) => string][] = [
  [/(-?\d+) pts?\b/g, (_m, n) => pts(+n)],
  [/ \((E|A|H|VH)\) -> /g, (_m, d) => ` (${d}) → `],
  [/ -> /g, () => ' → '],
  [/Basic Speed/g, () => 'Скорость'],
];

/** Translates one engine line to Russian; unknown lines are returned unchanged. */
export function engineRu(line: string, name: Name = (s) => s): string {
  const self = (l: string) => engineRu(l, name);
  let out = line;
  for (const [re, f] of RULES) {
    const m = re.exec(line);
    if (m) { out = f(m, name, self); break; }
  }
  for (const [re, f] of POST) out = out.replace(re, f as (m: string, ...g: string[]) => string);
  return out;
}

/** Current mode for engine lines; App sets it from the UI locale (originals -> English). */
export const engineLocale = ref<'ru' | 'en'>('ru');
/** Original plugin name -> name in the UI language (traits, skills, techniques, items, weapons). */
export const engineNames = ref<Map<string, string>>(new Map());
export const tx = (line: string) =>
  engineLocale.value === 'ru' ? engineRu(line, (n) => engineNames.value.get(n) ?? n) : line;

/** Builds the name map from a sheet view (rows carry `name` + `original_name`). */
export function namesFrom(groups: ({ name: string; original_name?: string | null; levels?: number }[] | undefined)[]): Map<string, string> {
  const m = new Map<string, string>();
  for (const g of groups) for (const r of g ?? []) {
    if (!r.original_name) continue;
    m.set(r.original_name, r.name);
    // trait reasons carry the level: "Striking ST 2"
    if (r.levels) m.set(`${r.original_name} ${r.levels}`, `${r.name} ${r.levels}`);
  }
  return m;
}
