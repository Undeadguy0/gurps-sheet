<script setup lang="ts">
// Character notes: rendered Markdown, or a raw textarea while editing.
// One editing session = one backend call (= one undo step): the draft is only
// sent when editing finishes (Готово, Esc, Ctrl+Enter or leaving the field).
import { computed, nextTick, ref, watch } from 'vue';
import { renderMarkdown } from '../markdown';

const props = defineProps<{ notes: string; compact?: boolean }>();
const emit = defineEmits<{ save: [string] }>();

const editing = ref(false);
const draft = ref(props.notes);
const open = ref(!props.compact || !!props.notes);
const area = ref<HTMLTextAreaElement | null>(null);
watch(() => props.notes, (n) => { if (!editing.value) draft.value = n; });
const html = computed(() => renderMarkdown(editing.value ? draft.value : props.notes));

async function start() {
  draft.value = props.notes; editing.value = true; open.value = true;
  await nextTick(); area.value?.focus();
}
function finish() {
  if (!editing.value) return;
  editing.value = false;
  if (draft.value !== props.notes) emit('save', draft.value);
}
function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' || (e.key === 'Enter' && (e.ctrlKey || e.metaKey))) { e.preventDefault(); e.stopPropagation(); finish(); }
}
defineExpose({ start, finish });
</script>

<template>
  <section class="panel notes-panel" :class="{ 'notes-panel--compact': compact, 'is-empty': !notes.trim() }" aria-labelledby="notes-title" data-test="notes">
    <header class="panel__header">
      <h2 id="notes-title" class="panel__title">
        <button v-if="compact" class="btn btn--link notes-panel__toggle" type="button" :aria-expanded="open" @click="open = !open">
          {{ open ? '▾' : '▸' }} Заметки
        </button>
        <template v-else>Заметки</template>
      </h2>
      <span v-if="editing" class="muted notes-panel__hint">Markdown · Ctrl+Enter или Esc — готово</span>
      <button v-if="editing" class="btn btn--primary" type="button" @mousedown.prevent @click="finish">Готово</button>
      <button v-else-if="open" class="btn" type="button" data-test="notes-edit" @click="start">Править</button>
    </header>
    <div v-show="open || editing">
      <textarea v-if="editing" ref="area" v-model="draft" class="field notes-panel__editor" :rows="compact ? 8 : 16"
        aria-label="Заметки (Markdown)" spellcheck="true" @keydown="onKey" @blur="finish"></textarea>
      <div v-else-if="notes.trim()" class="md notes-panel__view" title="Двойной клик — править" @dblclick="start" v-html="html"></div>
      <p v-else class="muted notes-panel__empty">
        Заметок нет. <button class="btn btn--link" type="button" @click="start">Написать</button>
        — поддерживается Markdown: заголовки, списки (вложенные), таблицы, **жирный**, ссылки.
      </p>
    </div>
  </section>
</template>

<style scoped>
.notes-panel__editor { width: 100%; box-sizing: border-box; font-family: var(--font-num); resize: vertical; min-height: 8em; }
.notes-panel__hint { margin-left: auto; font-size: 0.85em; }
.notes-panel__toggle { font: inherit; color: inherit; }
.notes-panel--compact .notes-panel__view { max-height: 40vh; overflow: auto; }
@media print {
  .notes-panel.is-empty { display: none; }
  .notes-panel__editor, .notes-panel__hint, .notes-panel__empty { display: none !important; }
  .notes-panel > div { display: block !important; }
  .notes-panel--compact .notes-panel__view { max-height: none; overflow: visible; }
}
</style>
