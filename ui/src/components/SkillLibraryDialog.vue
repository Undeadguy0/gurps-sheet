<script setup lang="ts">
import LibraryExplanation from './LibraryExplanation.vue';
import { computed, nextTick, onMounted, ref } from 'vue';
import type { SkillLibEntry, SkillLibrary, TechniqueLibEntry } from '../types';
import { matchesLibrary } from '../i18n';
import { DIFF_LABEL, signedInt } from '../skills';
import { excerpt } from '../markdown';

const props = defineProps<{ library: SkillLibrary }>();
const emit = defineEmits<{ close: []; addSkill: [string, string | null]; addTechnique: [string] }>();

const tab = ref<'skill' | 'technique'>('skill');
const query = ref('');
const previewId = ref<string | null>(null);
const spec = ref('');
const search = ref<HTMLInputElement | null>(null);
onMounted(() => nextTick(() => search.value?.focus()));

type Entry = SkillLibEntry | TechniqueLibEntry;
const list = computed<Entry[]>(() => (tab.value === 'skill' ? props.library.skills : props.library.techniques));
const items = computed(() => list.value.filter((e) => matchesLibrary(e, query.value)));
const preview = computed(() => list.value.find((e) => e.id === previewId.value) ?? null);
const isSkill = (e: Entry): e is SkillLibEntry => 'attribute' in e;
const kindText = (e: Entry) => (isSkill(e) ? `${e.attribute}/${e.difficulty}` : `${e.skill_name}${signedInt(e.default_mod)} · ${e.difficulty}`);
const defaultsText = (e: SkillLibEntry) => (e.defaults ?? []).map((d) => (d.attribute ? `${d.attribute}${signedInt(d.mod)}`
  : `${props.library.skills.find((s) => s.id === d.skill)?.translated_name ?? props.library.skills.find((s) => s.id === d.skill)?.name ?? d.skill}${signedInt(d.mod)}`)).join(', ');
function pick(e: Entry) { previewId.value = e.id; spec.value = ''; }
function add(e: Entry) {
  if (isSkill(e)) emit('addSkill', e.id, spec.value.trim() || null); else emit('addTechnique', e.id);
}
function switchTab(t: 'skill' | 'technique') { tab.value = t; previewId.value = null; }
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal library-dialog" role="dialog" aria-modal="true" aria-labelledby="slib-title" @keydown.esc.stop="emit('close')">
      <header class="modal__header">
        <h2 id="slib-title" class="panel__title">Библиотека навыков</h2>
        <input ref="search" v-model="query" class="field field--search" placeholder="Поиск (рус./англ.)…" aria-label="Поиск навыков" />
        <button class="btn btn--icon" type="button" aria-label="Закрыть (Esc)" @click="emit('close')">Esc</button>
      </header>
      <div class="chip-row" role="tablist" aria-label="Вид">
        <button type="button" role="tab" class="chip" :class="{ 'is-active': tab === 'skill' }" :aria-selected="tab === 'skill'" @click="switchTab('skill')">Навыки ({{ library.skills.length }})</button>
        <button type="button" role="tab" class="chip" :class="{ 'is-active': tab === 'technique' }" :aria-selected="tab === 'technique'" @click="switchTab('technique')">Приёмы ({{ library.techniques.length }})</button>
      </div>
      <div class="library-dialog__body">
        <ul v-if="items.length" class="library-list" role="listbox">
          <li v-for="e in items" :key="e.id" class="library-list__item library-list__item--skill" role="option" :aria-selected="preview?.id === e.id"
            :title="excerpt(e.explanation) || undefined" :class="{ 'is-preview': preview?.id === e.id }" @click="pick(e)" @dblclick="add(e)">
            <span class="library-list__name">{{ e.translated_name ?? e.name }}<span v-if="e.translated_name" class="muted original-name"> ({{ e.name }})</span><span v-if="e.plugin === 'custom'" class="badge badge--custom">своё</span></span>
            <span class="muted num">{{ kindText(e) }}</span>
            <span class="muted">{{ e.page }}</span>
          </li>
        </ul>
        <div v-else class="empty-state"><p>Ничего не найдено.</p><button class="btn" type="button" @click="query = ''">Сбросить поиск</button></div>
        <aside v-if="preview" class="library-preview" aria-label="Превью">
          <header class="library-preview__head">
            <h3 class="library-preview__title">{{ preview.translated_name ?? preview.name }}<small v-if="preview.translated_name" class="muted original-name"> ({{ preview.name }})</small></h3>
            <p class="library-preview__meta muted">
              <template v-if="isSkill(preview)">{{ preview.attribute }}/{{ preview.difficulty }} — {{ DIFF_LABEL[preview.difficulty] }}</template>
              <template v-else>Приём навыка «{{ preview.skill_name }}» · {{ DIFF_LABEL[preview.difficulty] }}</template>
              <template v-if="preview.page"> · стр. {{ preview.page }}</template>
            </p>
          </header>
          <LibraryExplanation :text="preview.explanation" />
          <section class="library-preview__section">
            <h4 class="library-preview__label">На листе</h4>
            <ul class="library-preview__effects">
              <template v-if="isSkill(preview)">
                <li>Стоимость: 1, 2, 4, 8 очков, дальше +4 за уровень; 1 очко = {{ preview.attribute }}{{ signedInt({ E: 0, A: -1, H: -2, VH: -3 }[preview.difficulty]) }}</li>
                <li v-if="preview.defaults?.length">По умолчанию: {{ defaultsText(preview) }}</li>
                <li v-else>Без умолчания: без очков навык недоступен</li>
              </template>
              <template v-else>
                <li>Умолчание: {{ preview.skill_name }}{{ signedInt(preview.default_mod) }}</li>
                <li>Максимум: {{ preview.skill_name }}{{ preview.max_mod ? signedInt(preview.max_mod) : '' }}</li>
                <li>{{ preview.difficulty === 'A' ? '1 очко за уровень' : '2 очка за первый уровень, затем по 1 очку' }}</li>
              </template>
            </ul>
          </section>
          <label v-if="isSkill(preview) && (preview.spec_required || preview.specializations?.length)" class="skill-details__spec">
            Специализация<span v-if="preview.spec_required"> (обязательна)</span>:
            <input v-model="spec" class="field" list="slib-spec" />
            <datalist id="slib-spec"><option v-for="s in preview.specializations" :key="s" :value="s" /></datalist>
          </label>
          <p class="library-preview__note muted">Плагин: {{ preview.plugin }}</p>
          <button class="btn btn--primary" type="button" @click="add(preview)">Добавить</button>
        </aside>
      </div>
    </div>
  </div>
</template>
