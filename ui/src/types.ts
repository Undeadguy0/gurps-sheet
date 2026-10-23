// Mirrors the Rust types serialised by the Tauri commands (gurps-rules / src-tauri).

export type SourceKind =
  | { kind: 'purchased' }
  | { kind: 'trait'; ref: string }
  | { kind: 'derived'; ref: string }
  | { kind: 'other'; ref: string };

export interface Contribution { source: SourceKind; amount: number; reason: string }
export interface Value { base: number; base_reason: string; contributions: Contribution[] }

export interface StatRow {
  key: string;
  label: string;
  /** Purchased steps (Basic Speed: quarter steps). */
  level: number;
  /** Stat units per step (Basic Speed 0.25, else 1). */
  step: number;
  /** Character points per step; 0 = not buyable. */
  cost_per_step: number;
  points: number;
  total: number;
  value: Value;
  explain: string[];
}

export interface PointsBreakdown {
  attributes: number; secondary: number; advantages: number; perks: number;
  disadvantages: number; quirks: number; skills: number; total: number; budget: number; unspent: number;
}

export interface Sheet {
  name: string;
  attributes: StatRow[];
  secondary: StatRow[];
  basic_lift: StatRow;
  damage: { thrust: string; swing: string; based_on_st: number; st_value?: Value; explain?: string[] };
  points: PointsBreakdown;
}

export type TraitCategory = 'advantage' | 'perk' | 'disadvantage' | 'quirk';

export type Feature =
  | { type: 'attribute_bonus'; target: string; amount: number; per_level?: boolean }
  | { type: 'lifting_st_bonus'; amount: number; per_level?: boolean }
  | { type: 'striking_st_bonus'; amount: number; per_level?: boolean }
  | { type: 'skill_bonus'; skills: string[]; amount: number; per_level?: boolean };

export interface TraitDef {
  id: string; name: string; category: TraitCategory; cost: number; cost_per_level: number;
  max_levels?: number | null; min_levels?: number | null; page?: string | null; features: Feature[];
  /** Plugin Markdown explanation: plain string or a locale table ({ ru, en }). */
  description?: LocalizedText | null;
}
export type LocalizedText = string | Record<string, string>;
/** `explanation` is the plugin text resolved for the UI locale. */
export interface LibraryEntry extends TraitDef {
  plugin: string; explanation?: string | null;
  /** Name in the UI language; null when showing originals or untranslated. `name` is the original. */
  translated_name?: string | null;
}
export interface LocaleInfo { available: string[]; locale: string; original_only: boolean }

export interface TraitView {
  index: number; id: string;
  /** Display name in the UI language (with level). */
  name: string;
  /** Original plugin name when `name` is a translation. */
  original_name?: string | null;
  category: TraitCategory; levels: number;
  leveled: boolean; min_level: number; cost: number; page?: string | null; source?: string | null;
  /** Effective explanation (user text, else plugin text), Markdown. */
  explanation?: string | null;
  /** Plugin text, used by "reset to default". */
  default_explanation?: string | null;
  custom_explanation: boolean;
}

export interface Issue { severity: 'error' | 'warning'; message: string }

export type Difficulty = 'E' | 'A' | 'H' | 'VH';
export type SkillDefault = { attribute: string; modifier: number } | { skill: string; modifier: number };
export interface SkillDef {
  id: string; name: string; attribute: string; difficulty: Difficulty; defaults?: SkillDefault[];
  specialization_required?: boolean; specializations?: string[]; page?: string | null; description?: LocalizedText | null;
}
export interface TechniqueDef {
  id: string; name: string; skill: string; difficulty: Difficulty; modifier: number; max: number;
  page?: string | null; description?: LocalizedText | null;
}
export interface SkillLibEntry extends SkillDef { plugin: string; explanation?: string | null; translated_name?: string | null }
export interface TechniqueLibEntry extends TechniqueDef {
  plugin: string; explanation?: string | null; translated_name?: string | null; skill_name: string;
}
export interface SkillLibrary { skills: SkillLibEntry[]; techniques: TechniqueLibEntry[] }

interface ExplainedView {
  index: number; id: string; name: string; original_name?: string | null; difficulty: string; points: number;
  level?: number | null; relative?: string | null; explain: string[]; page?: string | null; source?: string | null;
  explanation?: string | null; default_explanation?: string | null; custom_explanation: boolean;
}
export interface SkillView extends ExplainedView {
  attribute: string; from_default: boolean; bonus: number; defaults: string[];
  specialization?: string | null; specialization_required: boolean; specializations: string[];
}
export interface TechniqueView extends ExplainedView { skill: string; modifier: number; max: number; max_level?: number | null }

export interface SheetView { sheet: Sheet; traits: TraitView[]; skills: SkillView[]; techniques: TechniqueView[]; issues: Issue[]; path?: string | null; budget: number }

export type Outcome = 'critical_success' | 'success' | 'failure' | 'critical_failure';
export interface RollResult {
  target: number; modifier: number; effective: number; dice: [number, number, number];
  roll: number; margin: number; outcome: Outcome;
}

/** Where the library came from (plugin_status command). */
export interface PluginStatus { dir?: string | null; plugins: string[]; errors: string[]; traits: number; skills?: number }
