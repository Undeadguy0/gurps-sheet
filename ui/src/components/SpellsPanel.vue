<script setup lang="ts">
import { computed, ref } from 'vue';
import type { SpellView } from '../types';
import { tx } from '../engineRu';
import { excerpt } from '../markdown';
import { DIFF_LABEL, nextPoints, prevPoints } from '../skills';
import ExplanationRows from './ExplanationRows.vue';

// Spells: IQ/H or IQ/VH skills without default; Magery adds its level. Prerequisites are text only.
const props = defineProps<{ spells: SpellView[]; points: number }>();
const emit = defineEmits<{ add: []; setPoints: [number, number]; remove: [number]; setExplanation: [number, string | null] }>();

const expanded = ref<Set<number>>(new Set());
function toggle(i: number) { const s = new Set(expanded.value); if (s.has(i)) s.delete(i); else s.add(i); expanded.value = s; }
const filter = ref('');
const shown = computed(() => {
  const q = filter.value.trim().toLowerCase();
  return q ? props.spells.filter((s) => [s.name, s.original_name ?? '', ...s.college].some((n) => n.toLowerCase().includes(q))) : props.spells;
});
const COLS = 6;
</script>

<template>
  <section class="panel spells-panel" aria-labelledby="spells-title">
    <header class="panel__header">
      <h2 id="spells-title" class="panel__title">Заклинания</h2>
      <span class="panel__meta num">{{ points }} оч.</span>
      <span class="panel__spacer"></span>
      <input v-model="filter" class="field field--filter" placeholder="Фильтр" aria-label="Фильтр заклинаний" />
      <button class="btn" type="button" title="Библиотека заклинаний" @click="emit('add')">+ Добавить</button>
    </header>
    <div v-if="!spells.length" class="empty-state">
      <p>Заклинаний пока нет.</p>
      <button class="btn btn--primary" type="button" @click="emit('add')">Открыть библиотеку заклинаний</button>
    </div>
    <table v-else class="data-table spells-table">
      <thead>
        <tr><th scope="col">Название</th><th scope="col">Школа</th><th scope="col">Тип</th><th scope="col" class="is-num">Уровень</th><th scope="col" class="is-num">Очки</th><th scope="col"></th></tr>
      </thead>
      <tbody>
        <template v-for="s in shown" :key="s.index">
          <tr class="data-table__row" tabindex="0" :title="excerpt(s.explanation) || undefined" @keydown.enter.self="toggle(s.index)">
            <td class="cell cell--name">
              <button class="btn btn--icon btn--expand" type="button" :aria-expanded="expanded.has(s.index)" :aria-label="`Подробности: ${s.name}`"
                @click="toggle(s.index)">{{ expanded.has(s.index) ? '▾' : '▸' }}</button>
              <span>{{ s.name }}</span>
              <span v-if="s.original_name" class="muted original-name"> ({{ s.original_name }})</span>
              <span v-if="s.explanation" class="explanation-mark" :class="{ 'is-custom': s.custom_explanation }" aria-hidden="true">ⓘ</span>
            </td>
            <td class="cell muted">{{ s.college.join(', ') }}</td>
            <td class="cell muted" :title="DIFF_LABEL[s.difficulty]">{{ s.attribute }}/{{ s.difficulty }}</td>
            <td class="cell is-num">
              <template v-if="s.level != null">
                <strong class="num">{{ s.level }}</strong><span class="muted skill-rel"> {{ s.relative }}</span>
                <span v-if="s.bonus" class="skill-tag" title="Бонус магических способностей">+{{ s.bonus }} маг.</span>
              </template>
              <span v-else class="muted" title="У заклинаний нет умолчания">—</span>
            </td>
            <td class="cell is-num">
              <span class="stepper">
                <button class="btn btn--step" type="button" aria-label="Меньше очков" :disabled="s.points <= 0" @click="emit('setPoints', s.index, prevPoints(s.points))">−</button>
                <span class="stepper__value num">{{ s.points }}</span>
                <button class="btn btn--step" type="button" aria-label="Больше очков" @click="emit('setPoints', s.index, nextPoints(s.points))">+</button>
              </span>
            </td>
            <td class="cell cell--actions">
              <button class="btn btn--icon btn--danger" type="button" :aria-label="`Удалить ${s.name}`" @click="emit('remove', s.index)">×</button>
            </td>
          </tr>
          <ExplanationRows :open="expanded.has(s.index)" :colspan="COLS" :label="s.name" :text="s.explanation"
            :default-text="s.default_explanation" :custom="s.custom_explanation" @save="(t) => emit('setExplanation', s.index, t)">
            <div class="skill-details">
              <p class="muted">Требования (не проверяются): {{ s.prerequisites || '—' }}<template v-if="s.page"> · стр. {{ s.page }}</template></p>
              <ul class="skill-details__explain num"><li v-for="(l, i) in s.explain" :key="i">{{ tx(l) }}</li></ul>
            </div>
          </ExplanationRows>
        </template>
      </tbody>
    </table>
  </section>
</template>
