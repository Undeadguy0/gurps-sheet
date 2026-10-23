<script setup lang="ts">
import LibraryExplanation from './LibraryExplanation.vue';
import { computed, nextTick, onMounted, ref } from 'vue';
import type { EquipmentLibEntry } from '../types';
import { matchesLibrary } from '../i18n';
import { lb, money } from '../encumbrance';
import { excerpt } from '../markdown';

const props = defineProps<{ library: EquipmentLibEntry[]; target?: string | null }>();
const emit = defineEmits<{ close: []; add: [string, number] }>();

const query = ref('');
const previewId = ref<string | null>(null);
const qty = ref(1);
const search = ref<HTMLInputElement | null>(null);
onMounted(() => nextTick(() => search.value?.focus()));
const items = computed(() => props.library.filter((e) => matchesLibrary(e, query.value)));
const preview = computed(() => props.library.find((e) => e.id === previewId.value) ?? null);
function pick(e: EquipmentLibEntry) { previewId.value = e.id; qty.value = 1; }
const add = (e: EquipmentLibEntry) => emit('add', e.id, Math.max(1, Math.floor(qty.value || 1)));
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal library-dialog" role="dialog" aria-modal="true" aria-labelledby="elib-title" @keydown.esc.stop="emit('close')">
      <header class="modal__header">
        <h2 id="elib-title" class="panel__title">Библиотека снаряжения<small v-if="target" class="muted"> → в «{{ target }}»</small></h2>
        <input ref="search" v-model="query" class="field field--search" placeholder="Поиск (рус./англ.)…" aria-label="Поиск снаряжения" />
        <button class="btn btn--icon" type="button" aria-label="Закрыть (Esc)" @click="emit('close')">Esc</button>
      </header>
      <div class="library-dialog__body">
        <ul v-if="items.length" class="library-list" role="listbox">
          <li v-for="e in items" :key="e.id" class="library-list__item library-list__item--skill" role="option" :aria-selected="preview?.id === e.id"
            :title="excerpt(e.explanation) || undefined" :class="{ 'is-preview': preview?.id === e.id }" @click="pick(e)" @dblclick="add(e)">
            <span class="library-list__name">{{ e.translated_name ?? e.name }}<span v-if="e.translated_name" class="muted original-name"> ({{ e.name }})</span><span v-if="e.plugin === 'custom'" class="badge badge--custom">своё</span></span>
            <span class="muted num">{{ lb(e.weight) }} · {{ money(e.cost) }}</span>
            <span class="muted">{{ e.page }}</span>
          </li>
        </ul>
        <div v-else class="empty-state"><p>Ничего не найдено.</p><button class="btn" type="button" @click="query = ''">Сбросить поиск</button></div>
        <aside v-if="preview" class="library-preview" aria-label="Превью">
          <header class="library-preview__head">
            <h3 class="library-preview__title">{{ preview.translated_name ?? preview.name }}<small v-if="preview.translated_name" class="muted original-name"> ({{ preview.name }})</small></h3>
            <p class="library-preview__meta muted">
              {{ lb(preview.weight) }} · {{ money(preview.cost) }}<template v-if="preview.stats"> · {{ preview.stats }}</template>
              <template v-if="preview.container"> · контейнер</template><template v-if="preview.page"> · стр. {{ preview.page }}</template>
            </p>
          </header>
          <LibraryExplanation :text="preview.explanation" />
          <label class="skill-details__spec">Количество: <input v-model.number="qty" class="field field--qty" type="number" min="1" step="1" /></label>
          <p class="library-preview__note muted">Плагин: {{ preview.plugin }}</p>
          <button class="btn btn--primary" type="button" @click="add(preview)">Добавить</button>
        </aside>
      </div>
    </div>
  </div>
</template>
