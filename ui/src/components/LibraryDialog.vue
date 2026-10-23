<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue';
import type { LibraryEntry, TraitCategory } from '../types';
import { CATEGORY_LABEL } from '../labels';

const props = defineProps<{ library: LibraryEntry[] }>();
const emit = defineEmits<{ close: []; add: [string[]] }>();

const query = ref('');
const cat = ref<TraitCategory | ''>('');
const selected = ref<Set<string>>(new Set());
const preview = ref<LibraryEntry | null>(null);
const search = ref<HTMLInputElement | null>(null);
onMounted(() => nextTick(() => search.value?.focus()));

const items = computed(() => props.library.filter((e) =>
  (!cat.value || e.category === cat.value) && e.name.toLowerCase().includes(query.value.toLowerCase())));

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
          <li v-for="e in items" :key="e.id" class="library-list__item" role="option" :aria-selected="selected.has(e.id)"
            :class="{ 'is-preview': preview?.id === e.id }" @click="preview = e">
            <input type="checkbox" :checked="selected.has(e.id)" :aria-label="`Выбрать ${e.name}`" @click.stop="toggle(e.id)" />
            <span class="library-list__name">{{ e.name }}</span>
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
          <h3 class="library-preview__title">{{ preview.name }}</h3>
          <dl class="kv">
            <dt>Тип</dt><dd>{{ CATEGORY_LABEL[preview.category] }}</dd>
            <dt>Стоимость</dt><dd class="num">{{ costText(preview) }}</dd>
            <dt>Стр.</dt><dd>{{ preview.page ?? '—' }}</dd>
            <dt>Плагин</dt><dd>{{ preview.plugin }}</dd>
            <dt>Эффекты</dt>
            <dd>
              <span v-if="!preview.features.length" class="muted">нет (только стоимость)</span>
              <span v-for="(f, i) in preview.features" :key="i" class="num">
                {{ f.type === 'attribute_bonus' ? `${f.target} +${f.amount}` : f.type === 'lifting_st_bonus' ? `ST для груза +${f.amount}` : `ST для урона +${f.amount}` }}{{ f.per_level ? '/ур.' : '' }}
              </span>
            </dd>
          </dl>
          <button class="btn btn--primary" type="button" @click="emit('add', [preview.id])">Добавить</button>
        </aside>
      </div>
      <footer class="modal__footer">
        <button class="btn btn--primary" type="button" :disabled="!selected.size" @click="emit('add', [...selected])">
          Добавить выбранные ({{ selected.size }})
        </button>
      </footer>
    </div>
  </div>
</template>
