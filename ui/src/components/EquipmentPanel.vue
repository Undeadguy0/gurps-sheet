<script setup lang="ts">
import { computed, ref } from 'vue';
import { tx } from '../engineRu';
import { createColumnHelper, FlexRender, getCoreRowModel, useVueTable } from '@tanstack/vue-table';
import type { Encumbrance, EquipmentView } from '../types';
import { LEVEL_LABEL_RU, lb, money } from '../encumbrance';
import { ITEMS, plural } from '../plural';
import { excerpt } from '../markdown';
import ExplanationRows from './ExplanationRows.vue';

const props = defineProps<{ items: EquipmentView[]; encumbrance?: Encumbrance | null }>();
const emit = defineEmits<{
  add: [number[] | null]; setQuantity: [number[], number]; setCarried: [number[], boolean];
  remove: [number[]]; setExplanation: [number[], string | null];
}>();

const key = (v: EquipmentView) => v.path.join('.');
const filter = ref('');
const rows = computed(() => {
  const q = filter.value.trim().toLowerCase();
  return q ? props.items.filter((r) => [r.name, r.original_name ?? ''].some((n) => n.toLowerCase().includes(q))) : props.items;
});
const expanded = ref<Set<string>>(new Set());
function toggle(k: string) { const s = new Set(expanded.value); if (s.has(k)) s.delete(k); else s.add(k); expanded.value = s; }

const col = createColumnHelper<EquipmentView>();
const columns = [
  col.accessor('name', { header: 'Предмет' }),
  col.accessor('quantity', { header: 'Кол-во', meta: { num: true } }),
  col.accessor('weight', { header: 'Вес, lb', meta: { num: true } }),
  col.accessor('cost', { header: 'Цена', meta: { num: true } }),
  col.accessor('total_weight', { header: 'Σ вес', meta: { num: true } }),
  col.accessor('total_cost', { header: 'Σ цена', meta: { num: true } }),
  col.accessor('carried', { header: 'При себе' }),
  col.accessor((r) => r.page ?? '', { id: 'page', header: 'Стр.' }),
  col.display({ id: 'actions', header: '' }),
];
const table = useVueTable({
  get data() { return rows.value; },
  columns,
  getRowId: key,
  getCoreRowModel: getCoreRowModel(),
});
const tip = (v: EquipmentView) => excerpt(v.explanation) || undefined;
const fmt = (v: number) => (Number.isInteger(v) ? String(v) : String(Math.round(v * 100) / 100));
const enc = computed(() => props.encumbrance ?? null);
const count = computed(() => props.items.reduce((a, v) => a + v.quantity, 0));
</script>

<template>
  <section class="panel equipment-panel" aria-labelledby="equipment-title">
    <header class="panel__header">
      <h2 id="equipment-title" class="panel__title">Снаряжение</h2>
      <span class="panel__meta num">{{ plural(count, ITEMS) }}<template v-if="enc"> · {{ lb(enc.carried_weight) }} при себе · {{ money(enc.total_cost) }}</template></span>
      <span class="panel__spacer"></span>
      <input v-model="filter" class="field field--filter" placeholder="Фильтр" aria-label="Фильтр снаряжения" />
      <button class="btn" type="button" title="Библиотека снаряжения" @click="emit('add', null)">+ Добавить</button>
    </header>

    <div v-if="enc" class="encumbrance" role="group" aria-label="Нагрузка">
      <p class="encumbrance__summary">
        Нагрузка: <strong :class="{ 'is-danger': enc.overloaded }">{{ LEVEL_LABEL_RU[enc.level] }} ({{ enc.level }})</strong>
        · Движение <strong class="num">{{ enc.move_value }}</strong>
        · Уклонение <strong class="num">{{ enc.dodge }}</strong><span v-if="enc.dodge_penalty" class="muted num"> ({{ enc.dodge_penalty }})</span>
        <span class="muted"> · базовый груз {{ lb(enc.basic_lift) }}</span>
        <span v-if="enc.overloaded" class="badge badge--warn">больше 10×BL — двигаться нельзя</span>
      </p>
      <table class="data-table encumbrance__table">
        <thead><tr><th scope="col">Уровень</th><th scope="col" class="is-num">До, lb</th><th scope="col" class="is-num">Движение</th><th scope="col" class="is-num">Уклонение</th></tr></thead>
        <tbody>
          <tr v-for="l in enc.table" :key="l.level" :class="{ 'is-current': l.level === enc.level && !enc.overloaded }">
            <td>{{ l.level }} — {{ LEVEL_LABEL_RU[l.level] }}</td><td class="is-num num">{{ fmt(l.max_weight) }}</td>
            <td class="is-num num">{{ l.move_value }}</td><td class="is-num num">{{ l.dodge }}</td>
          </tr>
        </tbody>
      </table>
      <ul class="skill-details__explain num muted"><li v-for="(l, i) in enc.explain" :key="i">{{ tx(l) }}</li></ul>
      <p class="muted encumbrance__note">Здесь Уклонение только от нагрузки. Итог с Боевыми рефлексами и щитом — в блоке «Защиты» на вкладке «Оружие».</p>
    </div>

    <div v-if="!items.length" class="empty-state">
      <p>Снаряжения пока нет.</p>
      <button class="btn btn--primary" type="button" @click="emit('add', null)">Открыть библиотеку снаряжения</button>
    </div>
    <table v-else class="data-table equipment-table">
      <thead>
        <tr v-for="hg in table.getHeaderGroups()" :key="hg.id">
          <th v-for="hd in hg.headers" :key="hd.id" scope="col" :class="{ 'is-num': (hd.column.columnDef.meta as any)?.num }">
            <FlexRender :render="hd.column.columnDef.header" :props="hd.getContext()" />
          </th>
        </tr>
      </thead>
      <tbody>
        <template v-for="row in table.getRowModel().rows" :key="row.id">
          <tr class="data-table__row" :class="{ 'is-stored': !row.original.effectively_carried }" tabindex="0" :title="tip(row.original)" @keydown.enter.self="toggle(row.id)">
            <td v-for="cell in row.getVisibleCells()" :key="cell.id" :class="['cell', `cell--${cell.column.id}`, { 'is-num': (cell.column.columnDef.meta as any)?.num }]">
              <template v-if="cell.column.id === 'name'">
                <span class="equipment-indent" :style="{ paddingInlineStart: `${row.original.depth * 1.25}rem` }">
                  <button class="btn btn--icon btn--expand" type="button" :aria-expanded="expanded.has(row.id)"
                    :aria-label="`Подробности: ${row.original.name}`" @click="toggle(row.id)">{{ expanded.has(row.id) ? '▾' : '▸' }}</button>
                  <span v-if="row.original.depth" class="muted" aria-hidden="true">↳ </span>{{ row.original.name }}
                </span>
                <span v-if="row.original.original_name" class="muted original-name"> ({{ row.original.original_name }})</span>
                <span v-if="row.original.stats" class="skill-tag">{{ row.original.stats }}</span>
                <span v-if="row.original.container" class="skill-tag muted" title="Контейнер">конт.</span>
                <span v-if="row.original.explanation" class="explanation-mark" :class="{ 'is-custom': row.original.custom_explanation }" aria-hidden="true">ⓘ</span>
              </template>
              <template v-else-if="cell.column.id === 'quantity'">
                <span class="stepper">
                  <button class="btn btn--step" type="button" aria-label="Меньше" :disabled="row.original.quantity <= 1" @click="emit('setQuantity', row.original.path, row.original.quantity - 1)">−</button>
                  <span class="stepper__value num">{{ row.original.quantity }}</span>
                  <button class="btn btn--step" type="button" aria-label="Больше" @click="emit('setQuantity', row.original.path, row.original.quantity + 1)">+</button>
                </span>
              </template>
              <template v-else-if="cell.column.id === 'weight' || cell.column.id === 'total_weight'">{{ fmt(cell.getValue() as number) }}</template>
              <template v-else-if="cell.column.id === 'cost' || cell.column.id === 'total_cost'">{{ money(cell.getValue() as number) }}</template>
              <template v-else-if="cell.column.id === 'carried'">
                <label class="check" :title="row.original.effectively_carried || !row.original.carried ? undefined : 'Контейнер не при себе'">
                  <input type="checkbox" :checked="row.original.carried" :aria-label="`При себе: ${row.original.name}`"
                    @change="emit('setCarried', row.original.path, ($event.target as HTMLInputElement).checked)" />
                </label>
              </template>
              <template v-else-if="cell.column.id === 'actions'">
                <button v-if="row.original.container" class="btn btn--icon" type="button" :aria-label="`Положить в ${row.original.name}`" title="Положить внутрь" @click="emit('add', row.original.path)">+</button>
                <button class="btn btn--icon btn--danger" type="button" :aria-label="`Удалить ${row.original.name}`" @click="emit('remove', row.original.path)">×</button>
              </template>
              <span v-else><FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" /></span>
            </td>
          </tr>
          <ExplanationRows :open="expanded.has(row.id)" :colspan="row.getVisibleCells().length" :label="row.original.name"
            :text="row.original.explanation" :default-text="row.original.default_explanation" :custom="row.original.custom_explanation"
            @save="(t) => emit('setExplanation', row.original.path, t)">
              <p class="muted">{{ row.original.quantity }} × {{ lb(row.original.weight) }} = {{ lb(row.original.quantity * row.original.weight) }}<template v-if="row.original.container">, с содержимым {{ lb(row.original.total_weight) }}</template>; {{ row.original.quantity }} × {{ money(row.original.cost) }}<template v-if="row.original.source"> · плагин {{ row.original.source }}</template></p>
          </ExplanationRows>
        </template>
      </tbody>
    </table>
  </section>
</template>
