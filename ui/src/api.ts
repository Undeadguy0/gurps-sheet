// Backend facade: Tauri commands when running inside the app, an in-browser
// preview backend (mock.ts) when opened with plain `npm run dev`.
import type { LibraryEntry, RollResult, SheetView } from './types';
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
  removeTrait: (index: number) => call<SheetView>('remove_trait', { index }),
  getSheet: () => call<SheetView>('get_sheet'),
  listLibrary: () => call<LibraryEntry[]>('list_library'),
  roll: (target: number, modifier = 0) => call<RollResult>('roll', { target, modifier }),
};
