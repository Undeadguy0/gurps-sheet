// Backend facade: Tauri commands when running inside the app, an in-browser
// preview backend (mock.ts) when opened with plain `npm run dev`.
import type { LibraryEntry, LocaleInfo, PluginStatus, RollResult, SheetView, SkillLibrary, EquipmentLibEntry, WeaponLibEntry, SpellLibEntry, ModifierLibEntry } from './types';
import { MockBackend } from './mock';

export const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (inTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(cmd, args);
  }
  return mock.invoke(cmd, args ?? {}) as T;
}

const mock = new MockBackend();

export const api = {
  newCharacter: (name?: string) => call<SheetView>('new_character', { name }),
  load: (path: string) => call<SheetView>('load', { path }),
  save: (path?: string) => call<string>('save', { path }),
  setAttribute: (key: string, level: number) => call<SheetView>('set_attribute', { key, level }),
  setName: (name: string) => call<SheetView>('set_name', { name }),
  addTrait: (id: string, levels?: number) => call<SheetView>('add_trait', { id, levels }),
  setTraitLevels: (index: number, levels: number) => call<SheetView>('set_trait_levels', { index, levels }),
  /** `null` or blank text resets to the plugin default. */
  setTraitExplanation: (index: number, text: string | null) => call<SheetView>('set_trait_explanation', { index, text }),
  removeTrait: (index: number) => call<SheetView>('remove_trait', { index }),
  getLocales: () => call<LocaleInfo>('get_locales'),
  setLocale: (locale: string, originalOnly: boolean) => call<SheetView>('set_locale', { locale, originalOnly }),
  pluginStatus: () => call<PluginStatus>('plugin_status'),
  reloadPlugins: () => call<PluginStatus>('reload_plugins'),
  getSheet: () => call<SheetView>('get_sheet'),
  listLibrary: () => call<LibraryEntry[]>('list_library'),
  listSkillLibrary: () => call<SkillLibrary>('list_skill_library'),
  addSkill: (id: string, specialization?: string | null, points?: number) => call<SheetView>('add_skill', { id, specialization, points }),
  setSkillPoints: (index: number, points: number) => call<SheetView>('set_skill_points', { index, points }),
  setSkillSpecialization: (index: number, specialization: string | null) => call<SheetView>('set_skill_specialization', { index, specialization }),
  setSkillExplanation: (index: number, text: string | null) => call<SheetView>('set_skill_explanation', { index, text }),
  removeSkill: (index: number) => call<SheetView>('remove_skill', { index }),
  addTechnique: (id: string, points?: number) => call<SheetView>('add_technique', { id, points }),
  setTechniquePoints: (index: number, points: number) => call<SheetView>('set_technique_points', { index, points }),
  setTechniqueExplanation: (index: number, text: string | null) => call<SheetView>('set_technique_explanation', { index, text }),
  removeTechnique: (index: number) => call<SheetView>('remove_technique', { index }),
  listEquipmentLibrary: () => call<EquipmentLibEntry[]>('list_equipment_library'),
  addEquipment: (id: string, quantity?: number, parent?: number[] | null) => call<SheetView>('add_equipment', { id, quantity, parent }),
  setEquipmentQuantity: (path: number[], quantity: number) => call<SheetView>('set_equipment_quantity', { path, quantity }),
  setEquipmentCarried: (path: number[], carried: boolean) => call<SheetView>('set_equipment_carried', { path, carried }),
  setEquipmentExplanation: (path: number[], text: string | null) => call<SheetView>('set_equipment_explanation', { path, text }),
  removeEquipment: (path: number[]) => call<SheetView>('remove_equipment', { path }),
  listWeaponLibrary: () => call<WeaponLibEntry[]>('list_weapon_library'),
  addWeapon: (id: string) => call<SheetView>('add_weapon', { id }),
  removeWeapon: (index: number) => call<SheetView>('remove_weapon', { index }),
  setWeaponExplanation: (index: number, text: string | null) => call<SheetView>('set_weapon_explanation', { index, text }),
  listSpellLibrary: () => call<SpellLibEntry[]>('list_spell_library'),
  addSpell: (id: string, points?: number) => call<SheetView>('add_spell', { id, points }),
  setSpellPoints: (index: number, points: number) => call<SheetView>('set_spell_points', { index, points }),
  setSpellExplanation: (index: number, text: string | null) => call<SheetView>('set_spell_explanation', { index, text }),
  removeSpell: (index: number) => call<SheetView>('remove_spell', { index }),
  listModifierLibrary: () => call<ModifierLibEntry[]>('list_modifier_library'),
  addTraitModifier: (traitIndex: number, id: string) => call<SheetView>('add_trait_modifier', { traitIndex, id }),
  removeTraitModifier: (traitIndex: number, index: number) => call<SheetView>('remove_trait_modifier', { traitIndex, index }),
  setModifierExplanation: (traitIndex: number, index: number, text: string | null) =>
    call<SheetView>('set_modifier_explanation', { traitIndex, index, text }),
  roll: (target: number, modifier = 0) => call<RollResult>('roll', { target, modifier }),
};
