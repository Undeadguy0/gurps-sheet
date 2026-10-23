<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue';
import type { LibraryEntry, PluginStatus, TraitCategory } from '../types';
import { api, inTauri } from '../api';
import { CATEGORY_LABEL, featureText } from '../labels';
import { excerpt, renderMarkdown } from '../markdown';
import { matchesLibrary } from '../i18n';

const props = defineProps<{ library: LibraryEntry[] }>();
const emit = defineEmits<{ close: []; add: [string[]]; reloaded: [] }>();

const status = ref<PluginStatus | null>(null);
onMounted(async () => { status.value = await api.pluginStatus(); });
async function reload() { status.value = await api.reloadPlugins(); emit('reloaded'); }
const statusText = computed(() => {
  const s = status.value; if (!s) return '';
  if (s.errors.length) return `Ошибка плагинов: ${s.errors.join('; ')}`;
  return `${inTauri ? s.dir ?? '—' : 'демо-данные браузера'} · ${s.plugins.join(', ')} · черт: ${s.traits}`;
});

const query = ref('');
const cat = ref<TraitCategory | ''>('');
const selected = ref<Set<string>>(new Set());
// Keep only the id so the preview follows library refreshes (e.g. language switch).
const previewId = ref<string | null>(null);
const preview = computed(() => props.library.find((e) => e.id === previewId.value) ?? null);
const search = ref<HTMLInputElement | null>(null);
onMounted(() => nextTick(() => search.value?.focus()));

const items = computed(() => props.library.filter((e) => (!cat.value || e.category === cat.value) && matchesLibrary(e, query.value)));

function toggle(id: string) {
  const s = new Set(selected.value);
  if (s.has(id)) s.delete(id); else s.add(id);
  selected.value = s;
}
const costText = (e: LibraryEntry) => e.cost_per_level ? `${e.cost ? e.cost + ' + ' : ''}${e.cost_per_level}/ур.` : String(e.cost);
function reset() { query.value = ''; cat.value = ''; }
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal library-dialog" role="dialog" aria-modal="true" aria-labelledby="lib-title" @keydown.esc.stop="emit('close')">
      <header class="modal__header">
        <h2 id="lib-title" class="panel__title">Библиотека</h2>
        <input ref="search" v-model="query" class="field field--search" placeholder="Поиск…" aria-label="Поиск по библиотеке" />
        <button class="btn btn--icon" type="button" aria-label="Закрыть (Esc)" @click="emit('close')">Esc</button>
      </header>
      <div class="chip-row" role="group" aria-label="Тип">
        <button type="button" class="chip" :class="{ 'is-active': cat === '' }" @click="cat = ''">Все</button>
        <button v-for="(l, k) in CATEGORY_LABEL" :key="k" type="button" class="chip" :class="{ 'is-active': cat === k }" @click="cat = k as TraitCategory">{{ l }}</button>
      </div>
      <div class="library-dialog__body">
        <ul v-if="items.length" class="library-list" role="listbox" aria-multiselectable="true">
          <li v-for="e in items" :key="e.id" class="library-list__item" role="option" :aria-selected="selected.has(e.id)" :title="excerpt(e.explanation) || undefined"
            :class="{ 'is-preview': preview?.id === e.id }" @click="previewId = e.id">
            <input type="checkbox" :checked="selected.has(e.id)" :aria-label="`Выбрать ${e.name}`" @click.stop="toggle(e.id)" />
            <span class="library-list__name">{{ e.translated_name ?? e.name }}<span v-if="e.translated_name" class="muted original-name"> ({{ e.name }})</span></span>
            <span class="muted">{{ CATEGORY_LABEL[e.category] }}</span>
            <span class="num">{{ costText(e) }}</span>
            <span class="muted">{{ e.page }}</span>
          </li>
        </ul>
        <div v-else class="empty-state">
          <p>Ничего не найдено.</p>
          <button class="btn" type="button" @click="reset">Сбросить фильтры</button>
        </div>
        <aside v-if="preview" class="library-preview" aria-label="Превью">
          <header class="library-preview__head">
            <h3 class="library-preview__title">{{ preview.translated_name ?? preview.name }}<small v-if="preview.translated_name" class="muted original-name"> ({{ preview.name }})</small></h3>
            <p class="library-preview__meta muted">
              {{ CATEGORY_LABEL[preview.category] }} · <span class="num">{{ costText(preview) }}</span> оч.<template v-if="preview.page"> · стр. {{ preview.page }}</template>
            </p>
          </header>
          <section class="library-preview__section" aria-labelledby="lib-expl-title">
            <h4 id="lib-expl-title" class="library-preview__label">Пояснение</h4>
            <div v-if="preview.explanation" class="library-preview__explanation md" v-html="renderMarkdown(preview.explanation)"></div>
            <p v-else class="muted">Пояснения в плагине нет.</p>
          </section>
          <section v-if="preview.features.length" class="library-preview__section" aria-labelledby="lib-fx-title">
            <h4 id="lib-fx-title" class="library-preview__label">Эффекты на листе</h4>
            <ul class="library-preview__effects">
              <li v-for="(f, i) in preview.features" :key="i">{{ featureText(f) }}</li>
            </ul>
          </section>
          <p v-else class="library-preview__note muted">Автоматических эффектов на листе нет — учитывается только стоимость.</p>
          <p class="library-preview__note muted">Плагин: {{ preview.plugin }}</p>
          <button class="btn btn--primary" type="button" @click="emit('add', [preview.id])">Добавить</button>
        </aside>
      </div>
      <footer class="modal__footer">
        <span class="plugin-status muted" :class="{ 'is-error': status?.errors.length }" :title="statusText">{{ statusText }}</span>
        <button v-if="inTauri" class="btn" type="button" title="Перечитать плагины с диска" @click="reload">Перечитать</button>
        <button class="btn btn--primary" type="button" :disabled="!selected.size" @click="emit('add', [...selected])">
          Добавить выбранные ({{ selected.size }})
        </button>
      </footer>
    </div>
  </div>
</template>
