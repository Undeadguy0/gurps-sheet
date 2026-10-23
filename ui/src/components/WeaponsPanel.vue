<script setup lang="ts">
import { computed, ref } from 'vue';
import { createColumnHelper, FlexRender, getCoreRowModel, useVueTable } from '@tanstack/vue-table';
import type { AttackView, WeaponView } from '../types';
import { tx } from '../engineRu';
import { excerpt, renderMarkdown } from '../markdown';
import ExplanationEditor from './ExplanationEditor.vue';

const props = defineProps<{ weapons: WeaponView[] }>();
const emit = defineEmits<{ add: []; remove: [number]; setExplanation: [number, string | null] }>();

/** One table row per attack mode; the weapon name and actions sit on its first row. */
interface Row { key: string; w: WeaponView; a: AttackView; first: boolean; span: number }
const filter = ref('');
const rows = computed<Row[]>(() => {
  const q = filter.value.trim().toLowerCase();
  const ws = q ? props.weapons.filter((w) => [w.name, w.original_name ?? '', ...w.attacks.map((a) => a.skill)].some((n) => n.toLowerCase().includes(q))) : props.weapons;
  return ws.flatMap((w) => w.attacks.map((a, i) => ({ key: `${w.index}.${i}`, w, a, first: i === 0, span: w.attacks.length })));
});
const expanded = ref<Set<number>>(new Set());
function toggle(i: number) { const s = new Set(expanded.value); if (s.has(i)) s.delete(i); else s.add(i); expanded.value = s; }

const DMG_TYPE: Record<string, string> = { cut: 'реж', imp: 'прон', cr: 'дроб', pi: 'прон.пул' };
const MODE: Record<string, string> = { swing: 'замах', thrust: 'укол', punch: 'удар', kick: 'пинок' };
const mode = (m: string) => MODE[m] ?? m;
const col = createColumnHelper<Row>();
const columns = [
  col.accessor((r) => r.w.name, { id: 'weapon', header: 'Оружие' }),
  col.accessor((r) => r.a.mode, { id: 'mode', header: 'Атака' }),
  col.accessor((r) => r.a.skill, { id: 'skill', header: 'Навык' }),
  col.accessor((r) => r.a.level ?? null, { id: 'level', header: 'Ур.', meta: { num: true } }),
  col.accessor((r) => r.a.damage, { id: 'damage', header: 'Урон' }),
  col.accessor((r) => r.a.reach, { id: 'reach', header: 'Досяг.', meta: { num: true } }),
  col.accessor((r) => r.a.parry ?? null, { id: 'parry', header: 'Парир.', meta: { num: true } }),
  col.display({ id: 'actions', header: '' }),
];
const table = useVueTable({ get data() { return rows.value; }, columns, getRowId: (r) => r.key, getCoreRowModel: getCoreRowModel() });
const tip = (r: Row) => [r.a.explain.map(tx).join('\n'), excerpt(r.w.explanation)].filter(Boolean).join('\n\n') || undefined;
const parryText = (a: AttackView) => (!a.can_parry ? 'нет' : a.parry == null ? '—' : `${a.parry}${a.parry_modifier ? ` (${a.parry_modifier > 0 ? '+' : ''}${a.parry_modifier})` : ''}`);
</script>

<template>
  <section class="panel weapons-panel" aria-labelledby="weapons-title">
    <header class="panel__header">
      <h2 id="weapons-title" class="panel__title">Оружие ближнего боя</h2>
      <span class="panel__spacer"></span>
      <input v-model="filter" class="field field--filter" placeholder="Фильтр" aria-label="Фильтр оружия" />
      <button class="btn" type="button" title="Библиотека оружия" @click="emit('add')">+ Добавить</button>
    </header>
    <div v-if="!weapons.length" class="empty-state">
      <p>Оружия нет. Оружие из снаряжения появляется само; удар и пинок есть у нового персонажа.</p>
      <button class="btn btn--primary" type="button" @click="emit('add')">Открыть библиотеку оружия</button>
    </div>
    <table v-else class="data-table weapons-table">
      <thead>
        <tr v-for="hg in table.getHeaderGroups()" :key="hg.id">
          <th v-for="hd in hg.headers" :key="hd.id" scope="col" :class="{ 'is-num': (hd.column.columnDef.meta as any)?.num }">
            <FlexRender :render="hd.column.columnDef.header" :props="hd.getContext()" />
          </th>
        </tr>
      </thead>
      <tbody>
        <template v-for="row in table.getRowModel().rows" :key="row.id">
          <tr class="data-table__row" :class="{ 'is-stored': !row.original.w.available, 'is-continuation': !row.original.first }" tabindex="0"
            :title="tip(row.original)" @keydown.enter.self="toggle(row.original.w.index)">
            <td v-for="cell in row.getVisibleCells()" :key="cell.id" :class="['cell', `cell--${cell.column.id}`, { 'is-num': (cell.column.columnDef.meta as any)?.num }]">
              <template v-if="cell.column.id === 'weapon'">
                <template v-if="row.original.first">
                  <button class="btn btn--icon btn--expand" type="button" :aria-expanded="expanded.has(row.original.w.index)"
                    :aria-label="`Подробности: ${row.original.w.name}`" @click="toggle(row.original.w.index)">{{ expanded.has(row.original.w.index) ? '▾' : '▸' }}</button>
                  {{ row.original.w.name }}<span v-if="row.original.w.original_name" class="muted original-name"> ({{ row.original.w.original_name }})</span>
                  <span v-if="row.original.w.equipment && !row.original.w.available" class="skill-tag muted" title="Предмета нет в снаряжении">нет предмета</span>
                  <span v-else-if="row.original.w.equipment && !row.original.w.ready" class="skill-tag muted" title="Предмет не при себе">не при себе</span>
                  <span v-if="row.original.w.explanation" class="explanation-mark" :class="{ 'is-custom': row.original.w.custom_explanation }" aria-hidden="true">ⓘ</span>
                </template>
              </template>
              <template v-else-if="cell.column.id === 'mode'">{{ mode(row.original.a.mode) }}</template>
              <template v-else-if="cell.column.id === 'skill'">
                {{ row.original.a.skill }}<span v-if="row.original.a.via_technique" class="skill-tag" title="Через приём">приём</span>
              </template>
              <template v-else-if="cell.column.id === 'level'"><strong>{{ row.original.a.level ?? '—' }}</strong></template>
              <template v-else-if="cell.column.id === 'damage'">
                <span class="num">{{ row.original.a.damage }}</span> {{ DMG_TYPE[row.original.a.damage_type] ?? row.original.a.damage_type }}
                <span class="muted num"> ({{ row.original.a.damage_expr }})</span>
              </template>
              <template v-else-if="cell.column.id === 'parry'">{{ parryText(row.original.a) }}</template>
              <template v-else-if="cell.column.id === 'actions'">
                <button v-if="row.original.first" class="btn btn--icon btn--danger" type="button" :aria-label="`Удалить ${row.original.w.name}`" @click="emit('remove', row.original.w.index)">×</button>
              </template>
              <span v-else><FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" /></span>
            </td>
          </tr>
          <tr v-if="expanded.has(row.original.w.index) && row.original.first" class="explanation-row">
            <td :colspan="row.getVisibleCells().length">
              <div v-for="(a, i) in row.original.w.attacks" :key="i">
                <strong>{{ mode(a.mode) }}:</strong>
                <ul class="skill-details__explain num"><li v-for="(l, j) in a.explain" :key="j">{{ tx(l) }}</li></ul>
              </div>
              <p class="muted"><template v-if="row.original.w.page">стр. {{ row.original.w.page }}</template><template v-if="row.original.w.source"> · плагин {{ row.original.w.source }}</template></p>
              <ExplanationEditor :text="row.original.w.explanation" :default-text="row.original.w.default_explanation"
                :custom="row.original.w.custom_explanation" :label="row.original.w.name"
                @save="(t) => emit('setExplanation', row.original.w.index, t)" />
            </td>
          </tr>
          <tr v-else-if="row.original.first && row.original.w.explanation" class="explanation-row explanation-row--print-only">
            <td :colspan="row.getVisibleCells().length"><div class="md" v-html="renderMarkdown(row.original.w.explanation)"></div></td>
          </tr>
        </template>
      </tbody>
    </table>
  </section>
</template>
