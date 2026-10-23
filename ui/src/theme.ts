import { ref, watch } from 'vue';

export type ThemeId = 'tool' | 'gm-table' | 'paper';
export type Scheme = 'light' | 'dark';

export const THEMES: { id: ThemeId; label: string }[] = [
  { id: 'tool', label: 'Инструмент' },
  { id: 'gm-table', label: 'Стол мастера' },
  { id: 'paper', label: 'Бумажный лист' },
];

const KEY_T = 'gurps.theme';
const KEY_S = 'gurps.scheme';

function read<T extends string>(k: string, fallback: T, allowed: readonly string[]): T {
  try {
    const v = localStorage.getItem(k);
    return v && allowed.includes(v) ? (v as T) : fallback;
  } catch {
    return fallback;
  }
}

export const theme = ref<ThemeId>(read(KEY_T, 'tool', THEMES.map((t) => t.id)));
export const scheme = ref<Scheme>(read(KEY_S, 'light', ['light', 'dark']));

export function applyTheme() {
  const el = document.documentElement;
  el.dataset.theme = theme.value;
  el.dataset.scheme = scheme.value;
  try {
    localStorage.setItem(KEY_T, theme.value);
    localStorage.setItem(KEY_S, scheme.value);
  } catch {
    /* storage unavailable */
  }
}

watch([theme, scheme], applyTheme, { immediate: false });
