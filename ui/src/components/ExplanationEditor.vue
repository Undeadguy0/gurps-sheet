<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import { renderMarkdown } from '../markdown';

// Explanation (пояснение) viewer/editor: rendered Markdown, edit with live preview,
// reset to the plugin default. Emits `save(null)` to reset.
const props = defineProps<{ text?: string | null; defaultText?: string | null; custom: boolean; label: string }>();
const emit = defineEmits<{ save: [string | null] }>();

const editing = ref(false);
const draft = ref('');
const area = ref<HTMLTextAreaElement | null>(null);
const html = computed(() => renderMarkdown(props.text));
const draftHtml = computed(() => renderMarkdown(draft.value));

function edit() {
  draft.value = props.text ?? '';
  editing.value = true;
  nextTick(() => area.value?.focus());
}
function save() {
  const t = draft.value;
  // Saving the untouched plugin text keeps it as the default (no override).
  emit('save', !t.trim() || (!props.custom && t === (props.defaultText ?? '')) ? null : t);
  editing.value = false;
}
function reset() { emit('save', null); editing.value = false; }
</script>

<template>
  <div class="explanation" :class="{ 'is-editing': editing }">
    <template v-if="!editing">
      <div v-if="html" class="explanation__body md" v-html="html"></div>
      <p v-else class="explanation__empty muted">Пояснения нет.</p>
      <div class="explanation__actions">
        <span class="explanation__badge muted">{{ custom ? 'своё пояснение' : text ? 'из плагина' : '' }}</span>
        <button class="btn" type="button" @click="edit">{{ text ? 'Изменить' : 'Добавить пояснение' }}</button>
        <button v-if="custom" class="btn" type="button" :title="defaultText ? 'Вернуть текст плагина' : 'Удалить своё пояснение'" @click="reset">
          Сбросить
        </button>
      </div>
    </template>
    <template v-else>
      <div class="explanation__editor" @keydown.esc.stop.prevent="editing = false" @keydown.ctrl.enter.prevent="save" @keydown.meta.enter.prevent="save">
        <label class="explanation__pane">
          <span class="muted">Markdown</span>
          <textarea ref="area" v-model="draft" class="field explanation__textarea" rows="8" :aria-label="`Пояснение: ${label}`"
            placeholder="**жирный**, *курсив*, - список, # заголовок"></textarea>
        </label>
        <div class="explanation__pane">
          <span class="muted">Предпросмотр</span>
          <div class="explanation__preview md" v-html="draftHtml" aria-live="polite"></div>
        </div>
      </div>
      <div class="explanation__actions">
        <span class="muted explanation__hint">Ctrl+Enter — сохранить, Esc — отмена</span>
        <button class="btn btn--primary" type="button" @click="save">Сохранить</button>
        <button class="btn" type="button" @click="editing = false">Отмена</button>
        <button v-if="defaultText && (custom || draft !== defaultText)" class="btn" type="button" @click="draft = defaultText ?? ''">Текст плагина</button>
        <button v-if="custom" class="btn" type="button" @click="reset">Сбросить к плагину</button>
      </div>
    </template>
  </div>
</template>
