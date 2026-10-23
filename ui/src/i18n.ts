// Helpers for translated plugin data ("Перевод (Оригинал)").
import type { LibraryEntry } from './types';

/** Library search: matches the translated name, the original name and the id. */
export function matchesLibrary(e: LibraryEntry, q: string): boolean {
  const s = q.trim().toLowerCase();
  return !s || [e.translated_name ?? '', e.name, e.id].some((n) => n.toLowerCase().includes(s));
}

const KEY = 'gurps-sheet.locale';
export interface LocalePref { locale: string; originalOnly: boolean }
export function loadLocalePref(): LocalePref | null {
  try { const v = JSON.parse(localStorage.getItem(KEY) ?? 'null'); return v && typeof v.locale === 'string' ? v : null; } catch { return null; }
}
export function saveLocalePref(p: LocalePref) {
  try { localStorage.setItem(KEY, JSON.stringify(p)); } catch { /* private mode */ }
}
