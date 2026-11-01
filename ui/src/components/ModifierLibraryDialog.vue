<script setup lang="ts">
import { modPercent } from '../labels';
import LibraryExplanation from './LibraryExplanation.vue';
import { computed, nextTick, onMounted, ref } from 'vue';
import type { ModifierLibEntry, TraitView } from '../types';
import { matchesLibrary } from '../i18n';
import { excerpt } from '../markdown';
import { modifiedCost } from '../magic';

// Enhancement/limitation picker for one trait; the preview shows the resulting cost.
const props = defineProps<{ library: ModifierLibEntry[]; target: TraitView }>();
const emit = defineEmits<{ close: []; add: [string] }>();
const query = ref('');
const kind = ref<'' | 'enh' | 'lim'>('');
const previewId = ref<string | null>(null);
const preview = computed(() => props.library.find((e) => e.id === previewId.value) ?? null);
const items = computed(() => props.library.filter((e) => (!kind.value || (kind.value === 'enh') === modPercent(e) > 0) && matchesLibrary(e, query.value)));
const search = ref<HTMLInputElement | null>(null);
onMounted(() => nextTick(() => search.value?.focus()));
const pct = (v: number) => `${v > 0 ? '+' : ''}${v}%`;
const after = (e: ModifierLibEntry) => modifiedCost(props.target.base_cost, props.target.modifier_percent + modPercent(e));
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal library-dialog" role="dialog" aria-modal="true" aria-labelledby="mlib-title" @keydown.esc.stop="emit('close')">
      <header class="modal__header">
        <h2 id="mlib-title" class="panel__title">Модификаторы: {{ target.name }}</h2>
        <input ref="search" v-model="query" class="field field--search" placeholder="Поиск (рус./англ.)…" aria-label="Поиск модификаторов" />
        <button class="btn btn--icon" type="button" aria-label="Закрыть (Esc)" @click="emit('close')">Esc</button>
      </header>
      <div class="chip-row" role="group" aria-label="Вид">
        <button type="button" class="chip" :class="{ 'is-active': kind === '' }" @click="kind = ''">Все</button>
        <button type="button" class="chip" :class="{ 'is-active': kind === 'enh' }" @click="kind = 'enh'">Усиления</button>
        <button type="button" class="chip" :class="{ 'is-active': kind === 'lim' }" @click="kind = 'lim'">Ограничения</button>
      </div>
      <div class="library-dialog__body">
        <ul v-if="items.length" class="library-list" role="listbox">
          <li v-for="e in items" :key="e.id" class="library-list__item library-list__item--skill" role="option" :aria-selected="preview?.id === e.id"
            :title="excerpt(e.explanation) || undefined" :class="{ 'is-preview': preview?.id === e.id }" @click="previewId = e.id" @dblclick="emit('add', e.id)">
            <span class="library-list__name">{{ e.translated_name ?? e.name }}<span v-if="e.translated_name" class="muted original-name"> ({{ e.name }})</span></span>
            <span class="num">{{ pct(modPercent(e)) }}</span>
            <span class="muted">{{ e.page }}</span>
          </li>
        </ul>
        <div v-else class="empty-state"><p>Ничего не найдено.</p><button class="btn" type="button" @click="query = ''; kind = ''">Сбросить</button></div>
        <aside v-if="preview" class="library-preview" aria-label="Превью">
          <header class="library-preview__head">
            <h3 class="library-preview__title">{{ preview.translated_name ?? preview.name }}<small v-if="preview.translated_name" class="muted original-name"> ({{ preview.name }})</small></h3>
            <p class="library-preview__meta muted">
              {{ modPercent(preview) > 0 ? 'Усиление' : 'Ограничение' }} · <span class="num">{{ pct(modPercent(preview)) }}</span><template v-if="preview.page"> · стр. {{ preview.page }}</template>
            </p>
          </header>
          <LibraryExplanation :text="preview.explanation" />
          <p class="library-preview__note muted num">
            {{ target.name }}: {{ target.cost }} → {{ after(preview) }} оч. (итог не ниже −80%, округление вверх)
          </p>
          <p class="library-preview__note muted">Плагин: {{ preview.plugin }}</p>
          <button class="btn btn--primary" type="button" @click="emit('add', preview.id)">Добавить</button>
        </aside>
      </div>
    </div>
  </div>
</template>
