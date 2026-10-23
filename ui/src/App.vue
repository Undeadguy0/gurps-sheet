<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { api, inTauri } from './api';
import type { LibraryEntry, LocaleInfo, RollResult, SheetView, SkillLibrary, EquipmentLibEntry, WeaponLibEntry } from './types';
import { loadLocalePref, saveLocalePref } from './i18n';
import AppHeader from './components/AppHeader.vue';
import NavRail, { type Section } from './components/NavRail.vue';
import AttributesPanel from './components/AttributesPanel.vue';
import TraitsPanel from './components/TraitsPanel.vue';
import LibraryDialog from './components/LibraryDialog.vue';
import SkillsPanel from './components/SkillsPanel.vue';
import EquipmentPanel from './components/EquipmentPanel.vue';
import EquipmentLibraryDialog from './components/EquipmentLibraryDialog.vue';
import SkillLibraryDialog from './components/SkillLibraryDialog.vue';
import ProvenancePanel from './components/ProvenancePanel.vue';
import WeaponsPanel from './components/WeaponsPanel.vue';
import DefensesPanel from './components/DefensesPanel.vue';
import WeaponLibraryDialog from './components/WeaponLibraryDialog.vue';
import { ROLLABLE } from './labels';
import { engineLocale, engineNames, namesFrom, tx } from './engineRu';
import { watchEffect } from 'vue';

const sections: Section[] = [
  { id: 'sheet', label: 'Лист', icon: '▦', ready: true },
  { id: 'traits', label: 'Черты', icon: '✦', ready: true },
  { id: 'skills', label: 'Навыки', icon: '◎', ready: true },
  { id: 'equipment', label: 'Снаряж.', icon: '⚒', ready: true },
  { id: 'combat', label: 'Оружие', icon: '⚔', ready: true },
  { id: 'notes', label: 'Заметки', icon: '✎', ready: false },
  { id: 'table', label: 'За столом', icon: '⚄', ready: false },
];

const view = ref<SheetView | null>(null);
const library = ref<LibraryEntry[]>([]);
const rolls = ref<Record<string, RollResult>>({});
const section = ref('sheet');
const libraryOpen = ref(false);
const skillLibraryOpen = ref(false);
const equipmentLibrary = ref<EquipmentLibEntry[]>([]);
const weaponLibrary = ref<WeaponLibEntry[]>([]);
const weaponLibraryOpen = ref(false);
/** null = closed; [] = top level; path = into that container. */
const equipmentTarget = ref<number[] | null>(null);
const skillLibrary = ref<SkillLibrary>({ skills: [], techniques: [] });
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
// Engine lines are English; show them in Russian unless the user wants originals.
watchEffect(() => {
  const l = locales.value;
  engineLocale.value = l && (!l.locale.startsWith('ru') || l.original_only) ? 'en' : 'ru';
  const v = view.value;
  engineNames.value = v ? namesFrom([v.traits, v.skills, v.techniques, v.equipment, v.weapons]) : new Map();
});
async function setLocale(locale: string, originalOnly: boolean) {
  await run(api.setLocale(locale, originalOnly), false);
  saveLocalePref({ locale, originalOnly });
  [locales.value, library.value, skillLibrary.value, equipmentLibrary.value, weaponLibrary.value] = await Promise.all([api.getLocales(), api.listLibrary(), api.listSkillLibrary(), api.listEquipmentLibrary(), api.listWeaponLibrary()]);
}
async function reloadLibrary() {
  [library.value, locales.value, view.value] = await Promise.all([api.listLibrary(), api.getLocales(), api.getSheet()]);
}
const setTraitExplanation = (i: number, t: string | null) => run(api.setTraitExplanation(i, t));
type Kind = 'skill' | 'technique';
const setSkillPoints = (k: Kind, i: number, p: number) => run(k === 'skill' ? api.setSkillPoints(i, p) : api.setTechniquePoints(i, p));
const removeSkill = (k: Kind, i: number) => run(k === 'skill' ? api.removeSkill(i) : api.removeTechnique(i));
const setSkillExplanation = (k: Kind, i: number, t: string | null) => run(k === 'skill' ? api.setSkillExplanation(i, t) : api.setTechniqueExplanation(i, t));
const setSpecialization = (i: number, s: string | null) => run(api.setSkillSpecialization(i, s));
async function openEquipmentLibrary(parent: number[] | null) { equipmentLibrary.value = await api.listEquipmentLibrary(); equipmentTarget.value = parent ?? []; }
async function addEquipment(id: string, qty: number) {
  const parent = equipmentTarget.value?.length ? equipmentTarget.value : null;
  await run(api.addEquipment(id, qty, parent)); equipmentTarget.value = null;
}
const equipmentTargetName = computed(() => {
  const t = equipmentTarget.value; if (!t?.length) return null;
  return view.value?.equipment?.find((e) => e.path.join('.') === t.join('.'))?.name ?? null;
});
const setEquipmentQuantity = (p: number[], q: number) => run(api.setEquipmentQuantity(p, q));
const setEquipmentCarried = (p: number[], c: boolean) => run(api.setEquipmentCarried(p, c));
const removeEquipment = (p: number[]) => run(api.removeEquipment(p));
const setEquipmentExplanation = (p: number[], t: string | null) => run(api.setEquipmentExplanation(p, t));
async function openWeaponLibrary() { weaponLibrary.value = await api.listWeaponLibrary(); weaponLibraryOpen.value = true; }
async function addWeapon(id: string) { await run(api.addWeapon(id)); weaponLibraryOpen.value = false; }
const removeWeapon = (i: number) => run(api.removeWeapon(i));
const setWeaponExplanation = (i: number, t: string | null) => run(api.setWeaponExplanation(i, t));
async function openSkillLibrary() { skillLibrary.value = await api.listSkillLibrary(); skillLibraryOpen.value = true; }
async function addSkill(id: string, spec: string | null) { await run(api.addSkill(id, spec)); skillLibraryOpen.value = false; }
async function addTechnique(id: string) { await run(api.addTechnique(id)); skillLibraryOpen.value = false; }

function onKey(e: KeyboardEvent) {
  const t = e.target as HTMLElement;
  const typing = t && (t.tagName === 'INPUT' || t.tagName === 'SELECT' || t.tagName === 'TEXTAREA');
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); libraryOpen.value = true; return; }
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') { e.preventDefault(); if (inTauri) save(view.value?.path ?? undefined); return; }
  if (e.key === 'Escape') { libraryOpen.value = false; skillLibraryOpen.value = false; weaponLibraryOpen.value = false; equipmentTarget.value = null; explainKey.value = null; helpOpen.value = false; return; }
  if (e.altKey && /^[1-7]$/.test(e.key)) { section.value = sections[+e.key - 1].id; e.preventDefault(); return; }
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
        <span aria-hidden="true">{{ i.severity === 'error' ? '⚠' : 'ℹ' }}</span> {{ tx(i.message) }}
      </div>

      <div v-if="['sheet', 'traits', 'skills', 'equipment', 'combat'].includes(section)" class="sheet-grid">
        <AttributesPanel v-if="section === 'sheet'" class="sheet-grid__left" :sheet="view.sheet" :rolls="rolls" :selected="explainKey"
          @set-level="setLevel" @roll="roll" @explain="(k) => (explainKey = k)"
          @focus-row="(k) => (focusKey = k)" />
        <TraitsPanel v-if="section === 'sheet' || section === 'traits'" ref="traitsPanel" class="sheet-grid__right" :traits="view.traits" :points="view.sheet.points"
          @add="libraryOpen = true" @remove="removeTrait" @set-levels="setTraitLevels" @set-explanation="setTraitExplanation" />
        <SkillsPanel v-if="section === 'sheet' || section === 'skills'" class="sheet-grid__full" :skills="view.skills" :techniques="view.techniques" :points="view.sheet.points.skills"
          @add="openSkillLibrary" @set-points="setSkillPoints" @remove="removeSkill" @set-explanation="setSkillExplanation"
          @set-specialization="setSpecialization" />
        <EquipmentPanel v-if="section === 'sheet' || section === 'equipment'" class="sheet-grid__full" :items="view.equipment ?? []" :encumbrance="view.sheet.encumbrance"
          @add="openEquipmentLibrary" @set-quantity="setEquipmentQuantity" @set-carried="setEquipmentCarried" @remove="removeEquipment"
          @set-explanation="setEquipmentExplanation" />
        <DefensesPanel v-if="section === 'sheet' || section === 'combat'" class="sheet-grid__left" :defenses="view.sheet.defenses" />
        <WeaponsPanel v-if="section === 'sheet' || section === 'combat'" class="sheet-grid__full" :weapons="view.weapons ?? []"
          @add="openWeaponLibrary" @remove="removeWeapon" @set-explanation="setWeaponExplanation" />
      </div>
      <section v-else class="panel empty-state">
        <h2 class="panel__title">{{ sections.find((s) => s.id === section)?.label }}</h2>
        <p class="muted">Раздел появится в следующих версиях.</p>
      </section>
    </main>

    <ProvenancePanel v-if="explainRow" :row="explainRow" @close="explainKey = null" />
    <LibraryDialog v-if="libraryOpen" :library="library" @close="libraryOpen = false" @add="addTraits" @reloaded="reloadLibrary" />
    <SkillLibraryDialog v-if="skillLibraryOpen" :library="skillLibrary" @close="skillLibraryOpen = false" @add-skill="addSkill" @add-technique="addTechnique" />
    <EquipmentLibraryDialog v-if="equipmentTarget" :library="equipmentLibrary" :target="equipmentTargetName" @close="equipmentTarget = null" @add="addEquipment" />
    <WeaponLibraryDialog v-if="weaponLibraryOpen" :library="weaponLibrary" :owned="(view.weapons ?? []).map((w) => w.id)" @close="weaponLibraryOpen = false" @add="addWeapon" />

    <div v-if="helpOpen" class="modal-backdrop" @click.self="helpOpen = false">
      <div class="modal help-dialog" role="dialog" aria-modal="true" aria-labelledby="help-title">
        <h2 id="help-title" class="panel__title">Клавиши</h2>
        <dl class="kv">
          <dt><kbd>Ctrl</kbd>+<kbd>K</kbd></dt><dd>Библиотека</dd>
          <dt><kbd>/</kbd></dt><dd>Фильтр черт</dd>
          <dt><kbd>P</kbd></dt><dd>Откуда взялось число (строка в фокусе)</dd>
          <dt><kbd>R</kbd></dt><dd>Бросок 3d6 по строке в фокусе</dd>
          <dt><kbd>Alt</kbd>+<kbd>1..7</kbd></dt><dd>Разделы</dd>
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
