<script setup lang="ts">
export interface Section { id: string; label: string; icon: string; ready: boolean }
defineProps<{ sections: Section[]; active: string }>();
const emit = defineEmits<{ select: [string] }>();
</script>

<template>
  <nav class="nav-rail" aria-label="Разделы">
    <button
      v-for="(s, i) in sections" :key="s.id" type="button" class="nav-rail__item"
      :class="{ 'is-active': s.id === active }" :aria-current="s.id === active ? 'page' : undefined"
      :title="`${s.label} (Alt+${i + 1})${s.ready ? '' : ' — в разработке'}`" @click="emit('select', s.id)"
    >
      <span class="nav-rail__icon" aria-hidden="true">{{ s.icon }}</span>
      <span class="nav-rail__label">{{ s.label }}</span>
    </button>
    <span class="nav-rail__hint">Alt+1..0</span>
  </nav>
</template>
