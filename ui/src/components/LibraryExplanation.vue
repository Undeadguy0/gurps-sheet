<script setup lang="ts">
import { renderMarkdown } from '../markdown';
// "Пояснение" section of every library preview.
defineProps<{ text?: string | null }>();
</script>

<template>
  <section class="library-preview__section">
    <h4 class="library-preview__label">Пояснение</h4>
    <div v-if="text" class="library-preview__explanation md" v-html="renderMarkdown(text)"></div>
    <p v-else class="muted">Пояснения в плагине нет.</p>
  </section>
</template>
