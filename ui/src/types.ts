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
  level: number;
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
  | { type: 'striking_st_bonus'; amount: number; per_level?: boolean };

export interface TraitDef {
  id: string; name: string; category: TraitCategory; cost: number; cost_per_level: number;
  max_levels?: number | null; min_levels?: number | null; page?: string | null; features: Feature[];
  /** Plugin Markdown explanation: plain string or a locale table ({ ru, en }). */
  description?: LocalizedText | null;
}
export type LocalizedText = string | Record<string, string>;
/** `explanation` is the plugin text resolved for the UI locale. */
export interface LibraryEntry extends TraitDef { plugin: string; explanation?: string | null }

export interface TraitView {
  index: number; id: string; name: string; category: TraitCategory; levels: number;
  leveled: boolean; min_level: number; cost: number; page?: string | null; source?: string | null;
  /** Effective explanation (user text, else plugin text), Markdown. */
  explanation?: string | null;
  /** Plugin text, used by "reset to default". */
  default_explanation?: string | null;
  custom_explanation: boolean;
}

export interface Issue { severity: 'error' | 'warning'; message: string }

export interface SheetView { sheet: Sheet; traits: TraitView[]; issues: Issue[]; path?: string | null; budget: number }

export type Outcome = 'critical_success' | 'success' | 'failure' | 'critical_failure';
export interface RollResult {
  target: number; modifier: number; effective: number; dice: [number, number, number];
  roll: number; margin: number; outcome: Outcome;
}
