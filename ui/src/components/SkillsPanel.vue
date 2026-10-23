<script setup lang="ts">
import { pts } from '../plural';
import { computed, ref } from 'vue';
import { tx } from '../engineRu';
import { createColumnHelper, FlexRender, getCoreRowModel, getSortedRowModel, useVueTable, type SortingState } from '@tanstack/vue-table';
import type { SkillView, TechniqueView } from '../types';
import { DIFF_LABEL, nextPoints, nextTechniquePoints, prevPoints, prevTechniquePoints } from '../skills';
import type { Difficulty } from '../types';
import { excerpt, renderMarkdown } from '../markdown';
import ExplanationEditor from './ExplanationEditor.vue';

const props = defineProps<{ skills: SkillView[]; techniques: TechniqueView[]; points: number }>();
const emit = defineEmits<{
  add: []; setPoints: ['skill' | 'technique', number, number]; remove: ['skill' | 'technique', number];
  setExplanation: ['skill' | 'technique', number, string | null]; setSpecialization: [number, string | null];
}>();

type Row = { kind: 'skill'; key: string; v: SkillView } | { kind: 'technique'; key: string; v: TechniqueView };
const filter = ref('');
const rows = computed<Row[]>(() => {
  const q = filter.value.trim().toLowerCase();
  const all: Row[] = [
    ...props.skills.map((v) => ({ kind: 'skill' as const, key: `s${v.index}`, v })),
    ...props.techniques.map((v) => ({ kind: 'technique' as const, key: `t${v.index}`, v })),
  ];
  return q ? all.filter((r) => [r.v.name, r.v.original_name ?? ''].some((n) => n.toLowerCase().includes(q))) : all;
});

const expanded = ref<Set<string>>(new Set());
function toggle(k: string) { const s = new Set(expanded.value); if (s.has(k)) s.delete(k); else s.add(k); expanded.value = s; }

const col = createColumnHelper<Row>();
const columns = [
  col.accessor((r) => r.v.name, { id: 'name', header: 'Название' }),
  col.accessor((r) => (r.kind === 'skill' ? `${r.v.attribute}/${r.v.difficulty}` : `приём/${r.v.difficulty}`), { id: 'type', header: 'Атр/Сл' }),
  col.accessor((r) => r.v.level ?? -99, { id: 'level', header: 'Уровень', meta: { num: true } }),
  col.accessor((r) => r.v.points, { id: 'points', header: 'Очки', meta: { num: true } }),
  col.accessor((r) => r.v.page ?? '', { id: 'page', header: 'Стр.' }),
  col.display({ id: 'actions', header: '' }),
];
const sorting = ref<SortingState>([]);
const table = useVueTable({
  get data() { return rows.value; },
  columns,
  getRowId: (r) => r.key,
  state: { get sorting() { return sorting.value; } },
  onSortingChange: (u) => { sorting.value = typeof u === 'function' ? u(sorting.value) : u; },
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
});

function step(r: Row, dir: 1 | -1) {
  const d = r.v.difficulty as Difficulty;
  const p = r.kind === 'skill' ? (dir > 0 ? nextPoints(r.v.points) : prevPoints(r.v.points))
    : (dir > 0 ? nextTechniquePoints(d, r.v.points) : prevTechniquePoints(d, r.v.points));
  emit('setPoints', r.kind, r.v.index, p);
}
const atCap = (r: Row) => r.kind === 'technique' && r.v.level != null && r.v.max_level != null && r.v.level >= r.v.max_level;
const tip = (r: Row) => [r.v.explain.map(tx).join('\n'), excerpt(r.v.explanation)].filter(Boolean).join('\n\n') || undefined;
const specDraft = ref<Record<number, string>>({});
function saveSpec(v: SkillView) {
  const t = specDraft.value[v.index];
  if (t !== undefined && t !== (v.specialization ?? '')) emit('setSpecialization', v.index, t.trim() ? t : null);
}
</script>

<template>
  <section class="panel skills-panel" aria-labelledby="skills-title">
    <header class="panel__header">
      <h2 id="skills-title" class="panel__title">Навыки</h2>
      <span class="panel__meta num">{{ pts(points) }}</span>
      <span class="panel__spacer"></span>
      <input v-model="filter" class="field field--filter" placeholder="Фильтр" aria-label="Фильтр навыков" />
      <button class="btn" type="button" title="Библиотека навыков" @click="emit('add')">+ Добавить</button>
    </header>

    <div v-if="!skills.length && !techniques.length" class="empty-state">
      <p>Навыков пока нет.</p>
      <button class="btn btn--primary" type="button" @click="emit('add')">Открыть библиотеку навыков</button>
    </div>
    <table v-else class="data-table skills-table">
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
          <tr class="data-table__row" :data-kind="row.original.kind" tabindex="0" :title="tip(row.original)" @keydown.enter.self="toggle(row.id)">
            <td v-for="cell in row.getVisibleCells()" :key="cell.id" :class="['cell', `cell--${cell.column.id}`, { 'is-num': (cell.column.columnDef.meta as any)?.num }]">
              <template v-if="cell.column.id === 'name'">
                <button class="btn btn--icon btn--expand" type="button" :aria-expanded="expanded.has(row.id)"
                  :aria-label="`Подробности: ${row.original.v.name}`" @click="toggle(row.id)">{{ expanded.has(row.id) ? '▾' : '▸' }}</button>
                <span :class="{ 'technique-name': row.original.kind === 'technique' }">{{ row.original.v.name }}</span>
                <span v-if="row.original.v.original_name" class="muted original-name"> ({{ row.original.v.original_name }})</span>
                <span v-if="row.original.v.explanation" class="explanation-mark" :class="{ 'is-custom': row.original.v.custom_explanation }" aria-hidden="true">ⓘ</span>
                <span v-if="row.original.kind === 'skill' && row.original.v.specialization_required && !row.original.v.specialization"
                  class="badge badge--warn" title="Нужна специализация">спец.?</span>
              </template>
              <template v-else-if="cell.column.id === 'type'">
                <span class="muted" :title="DIFF_LABEL[row.original.v.difficulty]">{{ cell.getValue() }}</span>
              </template>
              <template v-else-if="cell.column.id === 'level'">
                <template v-if="row.original.v.level != null">
                  <strong class="num">{{ row.original.v.level }}</strong>
                  <span class="muted skill-rel"> {{ row.original.v.relative }}</span>
                  <span v-if="row.original.kind === 'skill' && row.original.v.from_default" class="muted skill-tag" title="По умолчанию">умолч.</span>
                  <span v-if="row.original.kind === 'skill' && row.original.v.bonus" class="skill-tag" title="Бонус таланта">+{{ row.original.v.bonus }} тал.</span>
                  <span v-if="atCap(row.original)" class="muted skill-tag" title="Максимум приёма">макс.</span>
                </template>
                <span v-else class="muted" title="Нет очков и нет умолчания">—</span>
              </template>
              <template v-else-if="cell.column.id === 'points'">
                <span class="stepper">
                  <button class="btn btn--step" type="button" aria-label="Меньше очков" :disabled="row.original.v.points <= 0" @click="step(row.original, -1)">−</button>
                  <span class="stepper__value num">{{ row.original.v.points }}</span>
                  <button class="btn btn--step" type="button" aria-label="Больше очков" :disabled="atCap(row.original)" @click="step(row.original, 1)">+</button>
                </span>
              </template>
              <template v-else-if="cell.column.id === 'actions'">
                <button class="btn btn--icon btn--danger" type="button" :aria-label="`Удалить ${row.original.v.name}`" @click="emit('remove', row.original.kind, row.original.v.index)">×</button>
              </template>
              <span v-else><FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" /></span>
            </td>
          </tr>
          <tr v-if="expanded.has(row.id)" class="explanation-row">
            <td :colspan="row.getVisibleCells().length">
              <div class="skill-details">
                <label v-if="row.original.kind === 'skill' && (row.original.v.specialization_required || row.original.v.specialization || row.original.v.specializations.length)" class="skill-details__spec">
                  Специализация<span v-if="row.original.v.specialization_required"> (обязательна)</span>:
                  <input class="field" :list="`spec-${row.id}`" :value="specDraft[row.original.v.index] ?? row.original.v.specialization ?? ''"
                    @input="specDraft[(row.original.v as SkillView).index] = ($event.target as HTMLInputElement).value"
                    @change="saveSpec(row.original.v as SkillView)" @keydown.enter="saveSpec(row.original.v as SkillView)" />
                  <datalist :id="`spec-${row.id}`"><option v-for="s in row.original.v.specializations" :key="s" :value="s" /></datalist>
                </label>
                <p v-if="row.original.kind === 'skill' && row.original.v.defaults.length" class="muted">По умолчанию: {{ row.original.v.defaults.join(', ') }}</p>
                <p v-if="row.original.kind === 'technique'" class="muted">
                  Умолчание: {{ row.original.v.modifier }} от навыка; максимум: навык{{ row.original.v.max ? (row.original.v.max > 0 ? '+' : '') + row.original.v.max : '' }}<template v-if="row.original.v.max_level != null"> ({{ row.original.v.max_level }})</template>
                </p>
                <ul class="skill-details__explain num"><li v-for="(l, i) in row.original.v.explain" :key="i">{{ tx(l) }}</li></ul>
              </div>
              <ExplanationEditor :text="row.original.v.explanation" :default-text="row.original.v.default_explanation"
                :custom="row.original.v.custom_explanation" :label="row.original.v.name"
                @save="(t) => emit('setExplanation', row.original.kind, row.original.v.index, t)" />
            </td>
          </tr>
          <tr v-else-if="row.original.v.explanation" class="explanation-row explanation-row--print-only">
            <td :colspan="row.getVisibleCells().length"><div class="md" v-html="renderMarkdown(row.original.v.explanation)"></div></td>
          </tr>
        </template>
      </tbody>
    </table>
  </section>
</template>
