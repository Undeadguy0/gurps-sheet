<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue';
import type { WeaponLibEntry } from '../types';
import { matchesLibrary } from '../i18n';
import { excerpt, renderMarkdown } from '../markdown';

const props = defineProps<{ library: WeaponLibEntry[]; owned: string[] }>();
const emit = defineEmits<{ close: []; add: [string] }>();
const query = ref('');
const previewId = ref<string | null>(null);
const search = ref<HTMLInputElement | null>(null);
onMounted(() => nextTick(() => search.value?.focus()));
const items = computed(() => props.library.filter((e) => matchesLibrary(e, query.value)
  || [e.skill_name, e.equipment_name ?? ''].some((n) => n.toLowerCase().includes(query.value.trim().toLowerCase()))));
const preview = computed(() => props.library.find((e) => e.id === previewId.value) ?? null);
const has = (e: WeaponLibEntry) => props.owned.includes(e.id);
const add = (e: WeaponLibEntry) => { if (!has(e)) emit('add', e.id); };
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal library-dialog" role="dialog" aria-modal="true" aria-labelledby="wlib-title" @keydown.esc.stop="emit('close')">
      <header class="modal__header">
        <h2 id="wlib-title" class="panel__title">Библиотека оружия</h2>
        <input ref="search" v-model="query" class="field field--search" placeholder="Поиск (рус./англ.)…" aria-label="Поиск оружия" />
        <button class="btn btn--icon" type="button" aria-label="Закрыть (Esc)" @click="emit('close')">Esc</button>
      </header>
      <div class="library-dialog__body">
        <ul v-if="items.length" class="library-list" role="listbox">
          <li v-for="e in items" :key="e.id" class="library-list__item library-list__item--skill" role="option" :aria-selected="preview?.id === e.id"
            :title="excerpt(e.explanation) || undefined" :class="{ 'is-preview': preview?.id === e.id, 'is-owned': has(e) }" @click="previewId = e.id" @dblclick="add(e)">
            <span class="library-list__name">{{ e.translated_name ?? e.name }}<span v-if="e.translated_name" class="muted original-name"> ({{ e.name }})</span></span>
            <span class="muted">{{ e.skill_name }}<template v-if="has(e)"> · уже есть</template></span>
            <span class="muted">{{ e.page }}</span>
          </li>
        </ul>
        <div v-else class="empty-state"><p>Ничего не найдено.</p><button class="btn" type="button" @click="query = ''">Сбросить поиск</button></div>
        <aside v-if="preview" class="library-preview" aria-label="Превью">
          <header class="library-preview__head">
            <h3 class="library-preview__title">{{ preview.translated_name ?? preview.name }}<small v-if="preview.translated_name" class="muted original-name"> ({{ preview.name }})</small></h3>
            <p class="library-preview__meta muted">Навык: {{ preview.skill_name }}<template v-if="preview.equipment_name"> · предмет: {{ preview.equipment_name }}</template><template v-if="preview.page"> · стр. {{ preview.page }}</template></p>
          </header>
          <ul class="num">
            <li v-for="(a, i) in preview.attacks" :key="i">{{ a.mode }}: {{ a.damage }} {{ a.damage_type }}<template v-if="a.reach">, досяг. {{ a.reach }}</template>,
              парир. {{ a.parry == null ? 'нет' : a.parry === 0 ? '0' : a.parry }}</li>
          </ul>
          <section class="library-preview__section">
            <h4 class="library-preview__label">Пояснение</h4>
            <div v-if="preview.explanation" class="library-preview__explanation md" v-html="renderMarkdown(preview.explanation)"></div>
            <p v-else class="muted">Пояснения в плагине нет.</p>
          </section>
          <p v-if="preview.equipment" class="library-preview__note muted">Оружие появляется само, когда предмет есть в снаряжении.</p>
          <p class="library-preview__note muted">Плагин: {{ preview.plugin }}</p>
          <button class="btn btn--primary" type="button" :disabled="has(preview)" @click="add(preview)">{{ has(preview) ? 'Уже есть' : 'Добавить' }}</button>
        </aside>
      </div>
    </div>
  </div>
</template>
