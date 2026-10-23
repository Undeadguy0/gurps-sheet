<script setup lang="ts">
import { computed, ref } from 'vue';
import { createColumnHelper, FlexRender, getCoreRowModel, getFilteredRowModel, getSortedRowModel, useVueTable, type SortingState } from '@tanstack/vue-table';
import type { PointsBreakdown, TraitView } from '../types';
import { CATEGORY_LABEL } from '../labels';

const props = defineProps<{ traits: TraitView[]; points: PointsBreakdown }>();
const emit = defineEmits<{ add: []; remove: [number]; setLevels: [number, number] }>();

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
  get data() { return props.traits; },
  columns,
  state: {
    get globalFilter() { return filter.value; },
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
        <tr v-for="row in table.getRowModel().rows" :key="row.id" class="data-table__row" :data-category="row.original.category" tabindex="0">
          <td v-for="cell in row.getVisibleCells()" :key="cell.id" :class="['cell', `cell--${cell.column.id}`, { 'is-num': (cell.column.columnDef.meta as any)?.num }]">
            <template v-if="cell.column.id === 'levels'">
              <span v-if="row.original.leveled" class="stepper">
                <button class="btn btn--step" type="button" aria-label="Уменьшить уровень" :disabled="row.original.levels <= 1"
                  @click="emit('setLevels', row.original.index, row.original.levels - 1)">−</button>
                <span class="stepper__value num">{{ row.original.levels }}</span>
                <button class="btn btn--step" type="button" aria-label="Увеличить уровень"
                  @click="emit('setLevels', row.original.index, row.original.levels + 1)">+</button>
              </span>
              <span v-else class="muted">—</span>
            </template>
            <template v-else-if="cell.column.id === 'actions'">
              <button class="btn btn--icon btn--danger" type="button" :aria-label="`Удалить ${row.original.name}`" @click="emit('remove', row.original.index)">×</button>
            </template>
            <span v-else :class="{ num: (cell.column.columnDef.meta as any)?.num }"><FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" /></span>
          </td>
        </tr>
      </tbody>
    </table>
  </section>
</template>
