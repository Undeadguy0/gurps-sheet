<script setup lang="ts">
import LibraryExplanation from './LibraryExplanation.vue';
import { computed, nextTick, onMounted, ref } from 'vue';
import type { SpellLibEntry } from '../types';
import { matchesLibrary } from '../i18n';
import { DIFF_LABEL } from '../skills';
import { excerpt } from '../markdown';

const props = defineProps<{ library: SpellLibEntry[] }>();
const emit = defineEmits<{ close: []; add: [string] }>();
const query = ref('');
const college = ref('');
const previewId = ref<string | null>(null);
const preview = computed(() => props.library.find((e) => e.id === previewId.value) ?? null);
const colleges = computed(() => [...new Set(props.library.flatMap((e) => e.college ?? []))].sort());
const items = computed(() => props.library.filter((e) => (!college.value || (e.college ?? []).includes(college.value)) && matchesLibrary(e, query.value)));
const search = ref<HTMLInputElement | null>(null);
onMounted(() => nextTick(() => search.value?.focus()));
const kind = (e: SpellLibEntry) => `${e.attribute ?? 'IQ'}/${e.difficulty ?? 'H'}`;
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal library-dialog" role="dialog" aria-modal="true" aria-labelledby="splib-title" @keydown.esc.stop="emit('close')">
      <header class="modal__header">
        <h2 id="splib-title" class="panel__title">Библиотека заклинаний</h2>
        <input ref="search" v-model="query" class="field field--search" placeholder="Поиск (рус./англ.)…" aria-label="Поиск заклинаний" />
        <button class="btn btn--icon" type="button" aria-label="Закрыть (Esc)" @click="emit('close')">Esc</button>
      </header>
      <div class="chip-row" role="group" aria-label="Школа">
        <button type="button" class="chip" :class="{ 'is-active': college === '' }" @click="college = ''">Все</button>
        <button v-for="c in colleges" :key="c" type="button" class="chip" :class="{ 'is-active': college === c }" @click="college = c">{{ c }}</button>
      </div>
      <div class="library-dialog__body">
        <ul v-if="items.length" class="library-list" role="listbox">
          <li v-for="e in items" :key="e.id" class="library-list__item library-list__item--skill" role="option" :aria-selected="preview?.id === e.id"
            :title="excerpt(e.explanation) || undefined" :class="{ 'is-preview': preview?.id === e.id }" @click="previewId = e.id" @dblclick="emit('add', e.id)">
            <span class="library-list__name">{{ e.translated_name ?? e.name }}<span v-if="e.translated_name" class="muted original-name"> ({{ e.name }})</span></span>
            <span class="muted num">{{ kind(e) }}</span>
            <span class="muted">{{ e.page }}</span>
          </li>
        </ul>
        <div v-else class="empty-state"><p>Ничего не найдено.</p><button class="btn" type="button" @click="query = ''; college = ''">Сбросить</button></div>
        <aside v-if="preview" class="library-preview" aria-label="Превью">
          <header class="library-preview__head">
            <h3 class="library-preview__title">{{ preview.translated_name ?? preview.name }}<small v-if="preview.translated_name" class="muted original-name"> ({{ preview.name }})</small></h3>
            <p class="library-preview__meta muted">
              {{ kind(preview) }} — {{ DIFF_LABEL[preview.difficulty ?? 'H'] }} · школа: {{ (preview.college ?? []).join(', ') || '—' }}<template v-if="preview.page"> · стр. {{ preview.page }}</template>
            </p>
          </header>
          <LibraryExplanation :text="preview.explanation" />
          <section class="library-preview__section">
            <h4 class="library-preview__label">Требования</h4>
            <p>{{ preview.prerequisites || '—' }}</p>
            <p class="library-preview__note muted">Требования не проверяются. Уровень = IQ + сложность по очкам + уровень магических способностей; без очков заклинания нет.</p>
          </section>
          <p class="library-preview__note muted">Плагин: {{ preview.plugin }}</p>
          <button class="btn btn--primary" type="button" @click="emit('add', preview.id)">Добавить (1 оч.)</button>
        </aside>
      </div>
    </div>
  </div>
</template>
