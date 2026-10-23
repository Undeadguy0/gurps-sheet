<script setup lang="ts">
import { computed, h, ref } from 'vue';
import { createColumnHelper, FlexRender, getCoreRowModel, useVueTable } from '@tanstack/vue-table';
import type { RollResult, Sheet, StatRow } from '../types';
import { OUTCOME_LABEL, ROLLABLE, STAT_LABEL, fmt, signed } from '../labels';

const props = defineProps<{ sheet: Sheet; rolls: Record<string, RollResult>; selected: string | null }>();
const emit = defineEmits<{ setLevel: [string, number]; roll: [string, number]; explain: [string]; focusRow: [string] }>();

interface Row extends StatRow { group: 'primary' | 'secondary' | 'derived'; buyable: boolean }

const rows = computed<Row[]>(() => [
  ...props.sheet.attributes.map((r) => ({ ...r, group: 'primary' as const, buyable: true })),
  ...props.sheet.secondary.map((r) => ({ ...r, group: 'secondary' as const, buyable: true })),
  { ...props.sheet.basic_lift, group: 'derived' as const, buyable: false },
]);

const expanded = ref<Set<string>>(new Set());
function toggle(key: string) {
  const s = new Set(expanded.value);
  if (s.has(key)) s.delete(key); else s.add(key);
  expanded.value = s;
}

const col = createColumnHelper<Row>();
const columns = [
  col.accessor('key', {
    header: 'Характеристика',
    cell: (c) => h('span', { class: 'stat-name' }, [
      h('abbr', { class: 'stat-name__key', title: STAT_LABEL[c.getValue()] }, c.row.original.label),
    ]),
  }),
  col.accessor('level', { header: 'Уровень', meta: { num: true } }),
  col.accessor('points', { header: 'Очки', meta: { num: true } }),
  col.accessor('total', { header: 'Итого', meta: { num: true } }),
  col.display({ id: 'sources', header: 'Источники' }),
  col.display({ id: 'roll', header: 'Бросок 3d6' }),
];

const table = useVueTable({
  get data() { return rows.value; },
  columns,
  getCoreRowModel: getCoreRowModel(),
  getRowId: (r) => r.key,
});

function step(r: Row, d: number) { emit('setLevel', r.key, r.level + d); }
function stepLabel(r: Row) { return r.key === 'BasicSpeed' ? '0.25' : '1'; }
const heading = (g: Row['group']) => ({ primary: 'Атрибуты', secondary: 'Вторичные характеристики', derived: 'Производные' })[g];
</script>

<template>
  <section class="panel attributes-panel" aria-labelledby="attr-title">
    <header class="panel__header">
      <h2 id="attr-title" class="panel__title">Атрибуты</h2>
      <span class="panel__meta num">{{ sheet.points.attributes + sheet.points.secondary }} оч.</span>
    </header>

    <table class="data-table attributes-table">
      <thead>
        <tr v-for="hg in table.getHeaderGroups()" :key="hg.id">
          <th v-for="hd in hg.headers" :key="hd.id" scope="col"
            :class="{ 'is-num': (hd.column.columnDef.meta as any)?.num }">
            <FlexRender :render="hd.column.columnDef.header" :props="hd.getContext()" />
          </th>
        </tr>
      </thead>
      <tbody>
        <template v-for="(row, i) in table.getRowModel().rows" :key="row.id">
          <tr v-if="i === 0 || table.getRowModel().rows[i - 1].original.group !== row.original.group" class="data-table__group">
            <th :colspan="columns.length" scope="rowgroup">{{ heading(row.original.group) }}</th>
          </tr>
          <tr class="data-table__row" :class="{ 'is-selected': selected === row.id }" :data-stat="row.id"
            tabindex="0" @focus="emit('focusRow', row.id)">
            <td v-for="cell in row.getVisibleCells()" :key="cell.id"
              :class="['cell', `cell--${cell.column.id}`, { 'is-num': (cell.column.columnDef.meta as any)?.num }]">
              <template v-if="cell.column.id === 'level'">
                <span v-if="row.original.buyable" class="stepper">
                  <button class="btn btn--step" type="button" :aria-label="`−${stepLabel(row.original)} ${row.original.label}`" @click="step(row.original, -1)">−</button>
                  <span class="stepper__value num">{{ signed(row.original.level) }}</span>
                  <button class="btn btn--step" type="button" :aria-label="`+${stepLabel(row.original)} ${row.original.label}`" @click="step(row.original, 1)">+</button>
                </span>
                <span v-else class="muted">—</span>
              </template>
              <template v-else-if="cell.column.id === 'points'">
                <span class="num">[{{ row.original.points }}]</span>
              </template>
              <template v-else-if="cell.column.id === 'total'">
                <span class="tooltip-host">
                  <button class="value-chip num" type="button" :aria-describedby="`tip-${row.id}`"
                    title="Откуда взялось число (P)" @click="emit('explain', row.id)">{{ fmt(row.original.total) }}</button>
                  <span :id="`tip-${row.id}`" role="tooltip" class="tooltip">
                    <strong>{{ row.original.label }} = {{ fmt(row.original.total) }}</strong>
                    <span v-for="(l, j) in row.original.explain" :key="j" class="tooltip__line">{{ l }}</span>
                    <small class="muted">P или клик — подробно</small>
                  </span>
                </span>
              </template>
              <template v-else-if="cell.column.id === 'sources'">
                <button class="btn btn--link" type="button" :aria-expanded="expanded.has(row.id)" @click="toggle(row.id)">
                  {{ row.original.value.contributions.length ? `${row.original.value.contributions.length} ист.` : 'база' }}
                  {{ expanded.has(row.id) ? '▴' : '▾' }}
                </button>
              </template>
              <template v-else-if="cell.column.id === 'roll'">
                <span v-if="ROLLABLE.has(row.id)" class="roll-cell">
                  <button class="btn btn--roll" type="button" :title="`Бросить 3d6 против ${row.original.label} ${fmt(row.original.total)} (R)`"
                    @click="emit('roll', row.id, row.original.total)">3d6</button>
                  <output v-if="rolls[row.id]" class="roll-result" :class="`roll-result--${rolls[row.id].outcome}`">
                    <span class="num">{{ rolls[row.id].roll }}</span>
                    <span>{{ OUTCOME_LABEL[rolls[row.id].outcome] }}</span>
                    <span class="muted num">({{ signed(rolls[row.id].margin) }})</span>
                  </output>
                </span>
              </template>
              <FlexRender v-else :render="cell.column.columnDef.cell" :props="cell.getContext()" />
            </td>
          </tr>
          <tr v-if="expanded.has(row.id)" class="data-table__detail">
            <td :colspan="columns.length">
              <ol class="explain-list">
                <li class="explain-list__item"><span>{{ row.original.value.base_reason === 'default' ? 'База' : `База ${row.original.value.base_reason}` }}</span><span class="num">{{ fmt(row.original.value.base) }}</span></li>
                <li v-for="(c, j) in row.original.value.contributions" :key="j" class="explain-list__item" :data-kind="c.source.kind">
                  <span>{{ c.reason }}</span><span class="num">{{ signed(c.amount) }}</span>
                </li>
                <li class="explain-list__item explain-list__item--total"><span>Итого</span><span class="num">{{ fmt(row.original.total) }}</span></li>
              </ol>
            </td>
          </tr>
        </template>
      </tbody>
    </table>
    <p class="panel__footnote muted" :title="sheet.damage.explain?.join('\n')">
      Урон: толчок <span class="num">{{ sheet.damage.thrust }}</span> · замах <span class="num">{{ sheet.damage.swing }}</span>
      (ST {{ sheet.damage.based_on_st }})
    </p>
  </section>
</template>
