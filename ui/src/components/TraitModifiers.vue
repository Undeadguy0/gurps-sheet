<script setup lang="ts">
import { ref } from 'vue';
import type { TraitView } from '../types';
import { tx } from '../engineRu';
import { excerpt, renderMarkdown } from '../markdown';
import ExplanationEditor from './ExplanationEditor.vue';

// Enhancements/limitations of one trait: cost explanation, list with per-modifier
// explanation (editor + override + reset), add/remove. `printOnly` = compact print view.
defineProps<{ t: TraitView; printOnly?: boolean }>();
const emit = defineEmits<{ add: []; remove: [number]; setExplanation: [number, string | null] }>();
const open = ref<Set<number>>(new Set());
function toggle(i: number) { const s = new Set(open.value); if (s.has(i)) s.delete(i); else s.add(i); open.value = s; }
const pct = (v: number) => `${v > 0 ? '+' : ''}${v}%`;
</script>

<template>
  <div class="trait-mods" :class="{ 'trait-mods--print': printOnly }">
    <div class="trait-mods__head">
      <strong>Модификаторы</strong>
      <span v-if="t.modifiers.length" class="muted num"> итого {{ pct(t.modifier_percent) }} · {{ t.base_cost }} → {{ t.cost }}</span>
      <span v-else class="muted"> нет (усиления и ограничения в % к стоимости)</span>
      <button v-if="!printOnly" class="btn" type="button" @click="emit('add')">+ Модификатор</button>
    </div>
    <ul v-if="t.cost_explain.length" class="skill-details__explain num"><li v-for="(l, i) in t.cost_explain" :key="i">{{ tx(l) }}</li></ul>
    <ul v-if="t.modifiers.length" class="trait-mods__list">
      <li v-for="m in t.modifiers" :key="m.index" class="trait-mods__item" :title="excerpt(m.explanation) || undefined">
        <div class="trait-mods__row">
          <button v-if="!printOnly" class="btn btn--icon btn--expand" type="button" :aria-expanded="open.has(m.index)"
            :aria-label="`Пояснение: ${m.name}`" @click="toggle(m.index)">{{ open.has(m.index) ? '▾' : '▸' }}</button>
          <span class="skill-tag" :class="m.cost_percent > 0 ? 'is-enh' : 'is-lim'">{{ m.cost_percent > 0 ? 'усиление' : 'ограничение' }}</span>
          <span>{{ m.name }}</span>
          <span v-if="m.original_name" class="muted original-name"> ({{ m.original_name }})</span>
          <span class="num">{{ pct(m.cost_percent) }}</span>
          <span v-if="m.explanation" class="explanation-mark" :class="{ 'is-custom': m.custom_explanation }" aria-hidden="true">ⓘ</span>
          <span class="muted">{{ m.page }}</span>
          <button v-if="!printOnly" class="btn btn--icon btn--danger" type="button" :aria-label="`Удалить ${m.name}`" @click="emit('remove', m.index)">×</button>
        </div>
        <ExplanationEditor v-if="!printOnly && open.has(m.index)" :text="m.explanation" :default-text="m.default_explanation"
          :custom="m.custom_explanation" :label="m.name" @save="(x) => emit('setExplanation', m.index, x)" />
        <div v-else-if="m.explanation" class="md" :class="{ 'print-only': !printOnly }" v-html="renderMarkdown(m.explanation)"></div>
      </li>
    </ul>
  </div>
</template>
