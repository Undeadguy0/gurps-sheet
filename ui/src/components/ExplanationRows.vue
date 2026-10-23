<script setup lang="ts">
import { renderMarkdown } from '../markdown';
import ExplanationEditor from './ExplanationEditor.vue';

// Shared explanation UI for every table (traits, skills, techniques, spells, equipment,
// weapons, modifiers): an expandable row with details (slot) + the editor
// (user override, reset to plugin text), and a print-only row with the text when collapsed.
defineProps<{
  open: boolean; colspan: number; label: string;
  text?: string | null; defaultText?: string | null; custom: boolean;
}>();
const emit = defineEmits<{ save: [string | null] }>();
</script>

<template>
  <tr v-if="open" class="explanation-row">
    <td :colspan="colspan">
      <slot />
      <ExplanationEditor :text="text" :default-text="defaultText" :custom="custom" :label="label" @save="(t) => emit('save', t)" />
    </td>
  </tr>
  <tr v-else-if="text" class="explanation-row explanation-row--print-only">
    <td :colspan="colspan"><div class="md" v-html="renderMarkdown(text)"></div></td>
  </tr>
</template>
