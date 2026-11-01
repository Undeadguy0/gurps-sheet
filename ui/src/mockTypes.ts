// Legacy (stage-3) shapes used only inside the in-browser preview backend (mock.ts and its
// TS rule mirrors). The app's real types are in types.ts; mock.ts converts at its boundary.

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
  disadvantages: number; quirks: number; skills: number; spells?: number; total: number; budget: number; unspent: number;
}

export interface Sheet {
  name: string;
  attributes: StatRow[];
  secondary: StatRow[];
  basic_lift: StatRow;
  damage: { thrust: string; swing: string; based_on_st: number; st_value?: Value; explain?: string[] };
  points: PointsBreakdown;
  encumbrance?: Encumbrance;
  weapons?: WeaponRow[];
  defenses?: Defenses;
}

export interface AttackDef {
  mode: string; damage: string; damage_type: string; reach?: string; parry?: number | null;
  skill?: string | null; skill_modifier?: number; technique?: string | null; defaults?: SkillDefault[];
  skill_damage_bonus?: { min_relative: number; per_die: number } | null;
}
export interface WeaponDef {
  id: string; name: string; equipment?: string | null; skill: string; attacks: AttackDef[];
  page?: string | null; description?: LocalizedText | null;
}
export interface AttackRow {
  mode: string; skill: string; skill_id?: string | null; via_technique: boolean; level?: number | null;
  damage_expr: string; damage: string; damage_type: string; reach: string;
  parry?: number | null; can_parry: boolean; parry_modifier: number; explain: string[];
}
export interface WeaponRow {
  index: number; id: string; name: string; equipment?: string | null; available: boolean; ready: boolean; attacks: AttackRow[];
}
export interface Defense { value: number; from: string; explain: string[] }
export interface ArmorRow { item: string; dr: string; value: number }
export interface Defenses {
  dodge: Defense; parry?: Defense | null; block?: Defense | null; db: number; db_from?: string | null; bonus: number; armor: ArmorRow[];
}
export interface AttackView extends AttackRow { original_skill?: string | null }
export interface WeaponView {
  index: number; id: string; name: string; original_name?: string | null; equipment?: string | null;
  available: boolean; ready: boolean; attacks: AttackView[]; page?: string | null; source?: string | null;
  explanation?: string | null; default_explanation?: string | null; custom_explanation: boolean;
}
export interface WeaponLibEntry extends WeaponDef {
  plugin: string; explanation?: string | null; translated_name?: string | null; equipment_name?: string | null; skill_name: string;
}

export interface EncumbranceLevel { level: number; name: string; max_weight: number; move_value: number; dodge: number }
export interface Encumbrance {
  level: number; name: string; carried_weight: number; total_weight: number; total_cost: number; carried_cost: number;
  basic_lift: number; base_move: number; move_value: number; dodge_penalty: number; base_dodge: number; dodge: number;
  overloaded: boolean; table: EncumbranceLevel[]; explain: string[];
}

export interface EquipmentDef {
  id: string; name: string; weight: number; cost: number; container?: boolean; stats?: string | null;
  page?: string | null; description?: LocalizedText | null;
}
/** Character's item (mock backend + tests); mirrors gurps_core::Equipment with the def flattened. */
export interface EquipmentItem extends EquipmentDef {
  quantity: number; carried?: boolean; contents?: EquipmentItem[]; source?: string | null; explanation?: string | null;
}
export interface EquipmentLibEntry extends EquipmentDef { plugin: string; explanation?: string | null; translated_name?: string | null }
export interface EquipmentView {
  path: number[]; depth: number; id: string; name: string; original_name?: string | null;
  quantity: number; weight: number; cost: number; total_weight: number; total_cost: number;
  carried: boolean; effectively_carried: boolean; container: boolean; stats?: string | null;
  page?: string | null; source?: string | null; explanation?: string | null; default_explanation?: string | null; custom_explanation: boolean;
}

export type TraitCategory = 'advantage' | 'perk' | 'disadvantage' | 'quirk';

export type Feature =
  | { type: 'attribute_bonus'; target: string; amount: number; per_level?: boolean }
  | { type: 'lifting_st_bonus'; amount: number; per_level?: boolean }
  | { type: 'striking_st_bonus'; amount: number; per_level?: boolean }
  | { type: 'skill_bonus'; skills: string[]; amount: number; per_level?: boolean }
  | { type: 'defense_bonus'; amount: number; defenses?: string[]; per_level?: boolean }
  | { type: 'spell_bonus'; amount: number; colleges?: string[]; per_level?: boolean };

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
  /** Cost before modifiers; total modifier %; applied modifiers; cost lines (engine text). */
  base_cost: number; modifier_percent: number; modifiers: ModifierView[]; cost_explain: string[];
}

export interface ModifierDef { id: string; name: string; cost_percent: number; page?: string | null; description?: LocalizedText | null }
export interface ModifierLibEntry extends ModifierDef { plugin: string; explanation?: string | null; translated_name?: string | null }
export interface ModifierView {
  index: number; id: string; name: string; original_name?: string | null; cost_percent: number; page?: string | null; source?: string | null;
  explanation?: string | null; default_explanation?: string | null; custom_explanation: boolean;
}

export interface SpellDef {
  id: string; name: string; attribute?: string; difficulty?: Difficulty; college?: string[]; prerequisites?: string | null;
  page?: string | null; description?: LocalizedText | null;
}
export interface SpellLibEntry extends SpellDef { plugin: string; explanation?: string | null; translated_name?: string | null }
export interface SpellView {
  index: number; id: string; name: string; original_name?: string | null; college: string[]; attribute: string; difficulty: Difficulty;
  points: number; level?: number | null; relative?: string | null; bonus: number; prerequisites?: string | null; explain: string[];
  page?: string | null; source?: string | null; explanation?: string | null; default_explanation?: string | null; custom_explanation: boolean;
}

export type NegativeRounding = 'toward_positive' | 'by_magnitude';
/** Per-character rules options. */
export interface RulesOptions { negative_rounding: NegativeRounding }

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

export interface SheetView { sheet: Sheet; traits: TraitView[]; skills: SkillView[]; techniques: TechniqueView[]; equipment?: EquipmentView[]; weapons?: WeaponView[]; spells?: SpellView[]; rules?: RulesOptions; notes?: string; pool?: PoolView; custom?: CustomContent; session?: SessionView; history?: HistoryView; issues: Issue[]; path?: string | null; budget: number }

export type Outcome = 'critical_success' | 'success' | 'failure' | 'critical_failure';
export interface RollResult {
  target: number; modifier: number; effective: number; dice: [number, number, number];
  roll: number; margin: number; outcome: Outcome;
}

/** Where the library came from (plugin_status command). */
export interface PluginStatus { dir?: string | null; plugins: string[]; errors: string[]; traits: number; skills?: number; equipment?: number; weapons?: number; spells?: number; modifiers?: number }

/** Current HP/FP at the table (session state, separate from build points). */
export interface SessionView { hp: number; hp_max: number; fp: number; fp_max: number }
/** Undo/redo state; labels name the action in Russian. */
export interface HistoryView {
  can_undo: boolean; can_redo: boolean; undo_label?: string | null; redo_label?: string | null; undo_depth: number; dirty: boolean;
}
export interface DamageRoll { expr: string; dice: number[]; adds: number; total: number }

export type LedgerKind = 'starting' | 'session-award' | 'gm-bonus' | 'training' | 'new-disadvantage' | 'correction'
  | 'influence' | 'impulse-buy' | 'buy-off' | 'other';
export interface LedgerEntry { id: number; date?: string; amount: number; kind: LedgerKind; note?: string; skill?: string | null }
export type LedgerInput = Omit<LedgerEntry, 'id'>;
export interface PoolView {
  entries: LedgerEntry[]; starting: number; income: number; earned: number; spent_outside: number; available: number;
  character_cost: number; unspent: number; disadvantage_limit: number; disadvantage_limit_custom: boolean;
  disadvantages: number; quirks: number; quirk_points: number;
}
export type CustomKind = 'trait' | 'skill' | 'spell' | 'equipment';
export interface CustomContent { traits?: TraitDef[]; skills?: SkillDef[]; spells?: SpellDef[]; equipment?: EquipmentDef[] }

