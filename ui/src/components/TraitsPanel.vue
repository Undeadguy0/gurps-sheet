<script setup lang="ts">
import { computed, ref } from 'vue';
import { createColumnHelper, FlexRender, getCoreRowModel, getFilteredRowModel, getSortedRowModel, useVueTable, type SortingState } from '@tanstack/vue-table';
import type { PointsBreakdown, TraitView } from '../types';
import { CATEGORY_LABEL } from '../labels';
import { tx } from '../engineRu';
import { excerpt } from '../markdown';
import ExplanationRows from './ExplanationRows.vue';
import TraitModifiers from './TraitModifiers.vue';

const props = defineProps<{ traits: TraitView[]; points: PointsBreakdown }>();
const emit = defineEmits<{
  add: []; remove: [number]; setLevels: [number, number]; setExplanation: [number, string | null];
  addModifier: [number]; removeModifier: [number, number]; setModifierExplanation: [number, number, string | null];
}>();

// Expanded explanation rows, keyed by trait index.
const expanded = ref<Set<number>>(new Set());
function toggle(i: number) {
  const s = new Set(expanded.value);
  if (s.has(i)) s.delete(i); else s.add(i);
  expanded.value = s;
}
// Filter matches the translated and the original name.
const shown = computed(() => {
  const q = filter.value.trim().toLowerCase();
  return q ? props.traits.filter((t) => [t.name, t.original_name ?? ''].some((n) => n.toLowerCase().includes(q))) : props.traits;
});
const tip = (t: TraitView) => excerpt(t.explanation) || undefined;

const filter = ref('');
const sorting = ref<SortingState>([]);
const col = createColumnHelper<TraitView>();
const columns = [
  col.accessor('name', { header: 'Название' }),
  col.accessor('category', { header: 'Тип', cell: (c) => CATEGORY_LABEL[c.getValue()] }),
  col.accessor('levels', { header: 'Уровень', meta: { num: true } }),
  col.accessor('cost', { header: 'Стоим.', meta: { num: true } }),
  col.accessor((r) => r.page ?? '', { id: 'page', header: 'Стр.' }),
  col.display({ id: 'actions', header: '' }),
];
const table = useVueTable({
  get data() { return shown.value; },
  columns,
  state: {
    get sorting() { return sorting.value; },
  },
  onSortingChange: (u) => { sorting.value = typeof u === 'function' ? u(sorting.value) : u; },
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getSortedRowModel: getSortedRowModel(),
});
const filterInput = ref<HTMLInputElement | null>(null);
defineExpose({ focusFilter: () => filterInput.value?.focus() });
const summary = computed(() => `Преим ${props.points.advantages + props.points.perks} · Недост ${props.points.disadvantages} · Причуды ${props.points.quirks}`);
</script>

<template>
  <section class="panel traits-panel" aria-labelledby="traits-title">
    <header class="panel__header">
      <h2 id="traits-title" class="panel__title">Черты</h2>
      <span class="panel__meta num">{{ summary }}</span>
      <span class="panel__spacer"></span>
      <input ref="filterInput" v-model="filter" class="field field--filter" placeholder="Фильтр (/)" aria-label="Фильтр черт" />
      <button class="btn" type="button" title="Библиотека (Ctrl+K)" @click="emit('add')">+ Добавить</button>
    </header>

    <div v-if="!traits.length" class="empty-state">
      <p>Черт пока нет.</p>
      <button class="btn btn--primary" type="button" @click="emit('add')">Открыть библиотеку (Ctrl+K)</button>
    </div>
    <table v-else class="data-table traits-table">
      <thead>
        <tr v-for="hg in table.getHeaderGroups()" :key="hg.id">
          <th v-for="hd in hg.headers" :key="hd.id" scope="col" :class="{ 'is-num': (hd.column.columnDef.meta as any)?.num, 'is-sortable': hd.column.getCanSort() }"
            :aria-sort="hd.column.getIsSorted() === 'asc' ? 'ascending' : hd.column.getIsSorted() === 'desc' ? 'descending' : undefined"
            @click="hd.column.getToggleSortingHandler()?.($event)">
            <FlexRender :render="hd.column.columnDef.header" :props="hd.getContext()" />
            <span v-if="hd.column.getIsSorted()" aria-hidden="true">{{ hd.column.getIsSorted() === 'asc' ? ' ▲' : ' ▼' }}</span>
          </th>
        </tr>
      </thead>
      <tbody>
        <template v-for="row in table.getRowModel().rows" :key="row.id">
        <tr class="data-table__row" :data-category="row.original.category" tabindex="0" :title="tip(row.original)"
          @keydown.enter.self="toggle(row.original.index)">
          <td v-for="cell in row.getVisibleCells()" :key="cell.id" :class="['cell', `cell--${cell.column.id}`, { 'is-num': (cell.column.columnDef.meta as any)?.num }]">
            <template v-if="cell.column.id === 'levels'">
              <span v-if="row.original.leveled" class="stepper">
                <button class="btn btn--step" type="button" aria-label="Уменьшить уровень" :disabled="row.original.levels <= row.original.min_level"
                  @click="emit('setLevels', row.original.index, row.original.levels - 1)">−</button>
                <span class="stepper__value num">{{ row.original.levels }}</span>
                <button class="btn btn--step" type="button" aria-label="Увеличить уровень"
                  @click="emit('setLevels', row.original.index, row.original.levels + 1)">+</button>
              </span>
              <span v-else class="muted">—</span>
            </template>
            <template v-else-if="cell.column.id === 'cost'">
              <span class="num" :title="row.original.cost_explain.map(tx).join('\n') || undefined">{{ row.original.cost }}</span>
              <span v-if="row.original.modifiers.length" class="muted num"> ({{ row.original.base_cost }})</span>
            </template>
            <template v-else-if="cell.column.id === 'actions'">
              <button class="btn btn--icon btn--danger" type="button" :aria-label="`Удалить ${row.original.name}`" @click="emit('remove', row.original.index)">×</button>
            </template>
            <template v-else-if="cell.column.id === 'name'">
              <button class="btn btn--icon btn--expand" type="button" :aria-expanded="expanded.has(row.original.index)"
                :aria-label="`Пояснение: ${row.original.name}`" :title="expanded.has(row.original.index) ? 'Скрыть пояснение' : 'Показать пояснение'"
                @click="toggle(row.original.index)">{{ expanded.has(row.original.index) ? '▾' : '▸' }}</button>
              <span>{{ row.original.name }}</span>
              <span v-if="row.original.original_name" class="muted original-name"> ({{ row.original.original_name }})</span>
              <span v-if="row.original.explanation" class="explanation-mark" :class="{ 'is-custom': row.original.custom_explanation }"
                aria-hidden="true" title="">ⓘ</span>
              <span v-for="m in row.original.modifiers" :key="m.index" class="skill-tag" :class="m.cost_percent > 0 ? 'is-enh' : 'is-lim'"
                :title="excerpt(m.explanation) || undefined">{{ m.name }} {{ m.cost_percent > 0 ? '+' : '' }}{{ m.cost_percent }}%</span>
            </template>
            <span v-else :class="{ num: (cell.column.columnDef.meta as any)?.num }"><FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" /></span>
          </td>
        </tr>
        <ExplanationRows :open="expanded.has(row.original.index)" :colspan="row.getVisibleCells().length" :label="row.original.name"
          :text="row.original.explanation" :default-text="row.original.default_explanation" :custom="row.original.custom_explanation"
          @save="(t) => emit('setExplanation', row.original.index, t)">
          <TraitModifiers :t="row.original" @add="emit('addModifier', row.original.index)"
            @remove="(m) => emit('removeModifier', row.original.index, m)"
            @set-explanation="(m, t) => emit('setModifierExplanation', row.original.index, m, t)" />
        </ExplanationRows>
        <tr v-if="!expanded.has(row.original.index) && row.original.modifiers.length" class="explanation-row explanation-row--print-only">
          <td :colspan="row.getVisibleCells().length">
            <TraitModifiers :t="row.original" print-only />
          </td>
        </tr>
        </template>
      </tbody>
    </table>
  </section>
</template>
