<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { api, inTauri } from './api';
import type { LibraryEntry, LocaleInfo, RollResult, SheetView } from './types';
import { loadLocalePref, saveLocalePref } from './i18n';
import AppHeader from './components/AppHeader.vue';
import NavRail, { type Section } from './components/NavRail.vue';
import AttributesPanel from './components/AttributesPanel.vue';
import TraitsPanel from './components/TraitsPanel.vue';
import LibraryDialog from './components/LibraryDialog.vue';
import ProvenancePanel from './components/ProvenancePanel.vue';
import { ROLLABLE } from './labels';

const sections: Section[] = [
  { id: 'sheet', label: 'Лист', icon: '▦', ready: true },
  { id: 'traits', label: 'Черты', icon: '✦', ready: true },
  { id: 'skills', label: 'Навыки', icon: '◎', ready: false },
  { id: 'equipment', label: 'Снаряж.', icon: '⚒', ready: false },
  { id: 'notes', label: 'Заметки', icon: '✎', ready: false },
  { id: 'table', label: 'За столом', icon: '⚄', ready: false },
];

const view = ref<SheetView | null>(null);
const library = ref<LibraryEntry[]>([]);
const rolls = ref<Record<string, RollResult>>({});
const section = ref('sheet');
const libraryOpen = ref(false);
const helpOpen = ref(false);
const explainKey = ref<string | null>(null);
const focusKey = ref<string | null>(null);
const dirty = ref(false);
const status = ref(inTauri ? 'Не сохранено' : 'Предпросмотр в браузере');
const error = ref<string | null>(null);
const traitsPanel = ref<InstanceType<typeof TraitsPanel> | null>(null);

async function run(p: Promise<SheetView>, changes = true) {
  try {
    view.value = await p;
    error.value = null;
    if (changes) { dirty.value = true; status.value = 'Есть несохранённые изменения'; }
  } catch (e) {
    error.value = String(e);
  }
}

const explainRow = computed(() => {
  if (!view.value || !explainKey.value) return null;
  const s = view.value.sheet;
  return [...s.attributes, ...s.secondary, s.basic_lift].find((r) => r.key === explainKey.value) ?? null;
});

async function roll(key: string, target: number) {
  rolls.value = { ...rolls.value, [key]: await api.roll(Math.floor(target)) };
}
async function addTraits(ids: string[]) {
  for (const id of ids) await run(api.addTrait(id));
  libraryOpen.value = false;
}
async function save(path?: string) {
  try {
    const p = await api.save(path);
    dirty.value = false;
    status.value = `Сохранено ${new Date().toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' })} · ${p}`;
    view.value = await api.getSheet();
  } catch (e) { error.value = String(e); }
}
async function load(path: string) {
  await run(api.load(path), false);
  if (!error.value) { dirty.value = false; status.value = `Открыт ${path}`; }
}

const rename = (n: string) => run(api.setName(n));
const newChar = () => run(api.newCharacter(), false);
const setLevel = (k: string, l: number) => run(api.setAttribute(k, l));
const removeTrait = (i: number) => run(api.removeTrait(i));
const setTraitLevels = (i: number, l: number) => run(api.setTraitLevels(i, l));
const locales = ref<LocaleInfo | null>(null);
async function setLocale(locale: string, originalOnly: boolean) {
  await run(api.setLocale(locale, originalOnly), false);
  saveLocalePref({ locale, originalOnly });
  [locales.value, library.value] = await Promise.all([api.getLocales(), api.listLibrary()]);
}
async function reloadLibrary() {
  [library.value, locales.value, view.value] = await Promise.all([api.listLibrary(), api.getLocales(), api.getSheet()]);
}
const setTraitExplanation = (i: number, t: string | null) => run(api.setTraitExplanation(i, t));

function onKey(e: KeyboardEvent) {
  const t = e.target as HTMLElement;
  const typing = t && (t.tagName === 'INPUT' || t.tagName === 'SELECT' || t.tagName === 'TEXTAREA');
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); libraryOpen.value = true; return; }
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') { e.preventDefault(); if (inTauri) save(view.value?.path ?? undefined); return; }
  if (e.key === 'Escape') { libraryOpen.value = false; explainKey.value = null; helpOpen.value = false; return; }
  if (e.altKey && /^[1-6]$/.test(e.key)) { section.value = sections[+e.key - 1].id; e.preventDefault(); return; }
  if (typing) return;
  if (e.key === '?') helpOpen.value = !helpOpen.value;
  else if (e.key === '/') { e.preventDefault(); traitsPanel.value?.focusFilter(); }
  else if (e.key.toLowerCase() === 'p' && focusKey.value) explainKey.value = focusKey.value;
  else if (e.key.toLowerCase() === 'r' && focusKey.value && ROLLABLE.has(focusKey.value) && view.value) {
    const s = view.value.sheet;
    const r = [...s.attributes, ...s.secondary].find((x) => x.key === focusKey.value);
    if (r) roll(r.key, r.total);
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKey);
  const pref = loadLocalePref();
  if (pref) await api.setLocale(pref.locale, pref.originalOnly);
  view.value = await api.getSheet();
  library.value = await api.listLibrary();
  locales.value = await api.getLocales();
});
onBeforeUnmount(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <div v-if="view" class="app-shell" :class="{ 'has-side-panel': explainRow }">
    <AppHeader :view="view" :dirty="dirty" :status="status" :locales="locales" @set-locale="setLocale"
      @rename="rename" @save="save" @load="load"
      @new="newChar" @help="helpOpen = !helpOpen" />
    <NavRail :sections="sections" :active="section" @select="(s) => (section = s)" />

    <main class="app-main" :data-section="section">
      <div v-if="error" class="alert alert--error" role="alert"><span aria-hidden="true">⚠</span> {{ error }}</div>
      <div v-for="(i, n) in view.issues" :key="n" class="alert" :class="`alert--${i.severity}`" role="status">
        <span aria-hidden="true">{{ i.severity === 'error' ? '⚠' : 'ℹ' }}</span> {{ i.message }}
      </div>

      <div v-if="section === 'sheet' || section === 'traits'" class="sheet-grid">
        <AttributesPanel v-if="section === 'sheet'" class="sheet-grid__left" :sheet="view.sheet" :rolls="rolls" :selected="explainKey"
          @set-level="setLevel" @roll="roll" @explain="(k) => (explainKey = k)"
          @focus-row="(k) => (focusKey = k)" />
        <TraitsPanel ref="traitsPanel" class="sheet-grid__right" :traits="view.traits" :points="view.sheet.points"
          @add="libraryOpen = true" @remove="removeTrait" @set-levels="setTraitLevels" @set-explanation="setTraitExplanation" />
      </div>
      <section v-else class="panel empty-state">
        <h2 class="panel__title">{{ sections.find((s) => s.id === section)?.label }}</h2>
        <p class="muted">Раздел появится в следующих версиях.</p>
      </section>
    </main>

    <ProvenancePanel v-if="explainRow" :row="explainRow" @close="explainKey = null" />
    <LibraryDialog v-if="libraryOpen" :library="library" @close="libraryOpen = false" @add="addTraits" @reloaded="reloadLibrary" />

    <div v-if="helpOpen" class="modal-backdrop" @click.self="helpOpen = false">
      <div class="modal help-dialog" role="dialog" aria-modal="true" aria-labelledby="help-title">
        <h2 id="help-title" class="panel__title">Клавиши</h2>
        <dl class="kv">
          <dt><kbd>Ctrl</kbd>+<kbd>K</kbd></dt><dd>Библиотека</dd>
          <dt><kbd>/</kbd></dt><dd>Фильтр черт</dd>
          <dt><kbd>P</kbd></dt><dd>Откуда взялось число (строка в фокусе)</dd>
          <dt><kbd>R</kbd></dt><dd>Бросок 3d6 по строке в фокусе</dd>
          <dt><kbd>Alt</kbd>+<kbd>1..6</kbd></dt><dd>Разделы</dd>
          <dt><kbd>Ctrl</kbd>+<kbd>S</kbd></dt><dd>Сохранить</dd>
          <dt><kbd>Esc</kbd></dt><dd>Закрыть панель</dd>
          <dt><kbd>?</kbd></dt><dd>Эта шпаргалка</dd>
        </dl>
        <button class="btn" type="button" @click="helpOpen = false">Закрыть</button>
      </div>
    </div>
  </div>
  <div v-else class="app-loading" aria-busy="true"><div class="skeleton"></div></div>
</template>
