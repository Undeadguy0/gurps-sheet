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

/** Attack mode of an item (gurps_core::AttackMode). */
export interface AttackMode {
  sid: string; name?: string | null; natural?: boolean; skill?: string | null;
  damage: { base?: string | null; dice?: number | null; mod?: number; type?: string | null };
  reach?: number[]; parry?: number | null; skill_modifier?: number | null; technique?: string | null;
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
  available: boolean; ready: boolean; attacks: AttackView[];
  /** Item's explanation (edited on the item) or the natural attack's text. */
  explanation?: string | null;
}

export interface EncumbranceLevel { level: number; name: string; max_weight: number; move_value: number; dodge: number }
export interface Encumbrance {
  level: number; name: string; carried_weight: number; total_weight: number; total_cost: number; carried_cost: number;
  basic_lift: number; base_move: number; move_value: number; dodge_penalty: number; base_dodge: number; dodge: number;
  overloaded: boolean; table: EncumbranceLevel[]; explain: string[];
}

/** Library common fields: own sid, pack code, texts for the UI language. */
interface LibMeta { plugin: string; id: string; explanation?: string | null; translated_name?: string | null }

export interface ItemDef {
  sid: string; name: string; weight: number; cost: number; container?: boolean;
  armor?: { loc: string; dr?: number | null }[]; shield?: { db: number; block_skill?: string | null } | null;
  attack_modes?: AttackMode[]; page?: string | null; description?: LocalizedText | null;
}
export type EquipmentLibEntry = ItemDef & LibMeta;
export interface EquipmentView {
  path: number[]; depth: number; id: string; name: string; original_name?: string | null;
  quantity: number; weight: number; cost: number; total_weight: number; total_cost: number;
  carried: boolean; effectively_carried: boolean; container: boolean; stats?: string | null;
  page?: string | null; source?: string | null; explanation?: string | null; default_explanation?: string | null; custom_explanation: boolean;
}

/** Trait kind code (ref_trait_kind). */
export type TraitKind = 'adv' | 'disadv' | 'perk' | 'quirk';

export interface TraitCost { type: string; base: number; per_level?: number | null; max_level?: number | null; min_level?: number | null }
export interface TraitDef {
  sid: string; name: string; kind: TraitKind; cost: TraitCost; features?: unknown[]; prereqs?: unknown;
  page?: string | null;
  /** Markdown explanation: plain string or a locale table ({ ru, en }). */
  description?: LocalizedText | null;
}
export type LocalizedText = string | Record<string, string>;
/** `translated_name` is the name in the UI language (null when untranslated); `name` is the original. */
export type LibraryEntry = TraitDef & LibMeta;
export interface LocaleInfo { available: string[]; locale: string; original_only: boolean }

export interface TraitView {
  index: number; id: string;
  /** Display name in the UI language (with level). */
  name: string;
  /** Original plugin name when `name` is a translation. */
  original_name?: string | null;
  kind: TraitKind; level: number;
  leveled: boolean; min_level: number; max_level?: number | null; cost: number; page?: string | null; source?: string | null;
  /** Effective explanation (user text, else plugin text), Markdown. */
  explanation?: string | null;
  /** Plugin text, used by "reset to default". */
  default_explanation?: string | null;
  custom_explanation: boolean;
  /** Cost before modifiers; total modifier %; applied modifiers; cost lines (engine text). */
  base_cost: number; modifier_percent: number; modifiers: ModifierView[]; cost_explain: string[];
}

export interface ModifierDef { sid: string; name: string; type: string; value_kind: string; amount: number; page?: string | null; description?: LocalizedText | null }
export type ModifierLibEntry = ModifierDef & LibMeta;
export interface ModifierView {
  index: number; id: string; name: string; original_name?: string | null; cost_percent: number; page?: string | null; source?: string | null;
  explanation?: string | null; default_explanation?: string | null; custom_explanation: boolean;
}

export interface SpellDef {
  sid: string; name: string; difficulty: Difficulty; colleges: string[]; prereqs?: unknown;
  page?: string | null; description?: LocalizedText | null;
}
export type SpellLibEntry = SpellDef & LibMeta;
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
export interface SkillDefault { attribute?: string | null; skill?: string | null; mod: number }
export interface SkillDef {
  sid: string; name: string; attribute: string; difficulty: Difficulty; defaults?: SkillDefault[];
  spec_required?: boolean; specializations?: string[]; page?: string | null; description?: LocalizedText | null;
}
export interface TechniqueDef {
  sid: string; name: string; parent_skill: string; difficulty: Difficulty; default_mod: number; max_mod?: number | null;
  page?: string | null; description?: LocalizedText | null;
}
export type SkillLibEntry = SkillDef & LibMeta;
export type TechniqueLibEntry = TechniqueDef & LibMeta & { skill_name: string };
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
export interface CustomContent { traits?: TraitDef[]; skills?: SkillDef[]; spells?: SpellDef[]; items?: ItemDef[] }

