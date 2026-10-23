<script setup lang="ts">
// «Очки»: point pool summary, ledger (TanStack) and the add/edit entry dialog.
// The character's own cost is live (from the sheet); only the ledger is stored.
import { computed, h, ref } from 'vue';
import { createColumnHelper, FlexRender, getCoreRowModel, getSortedRowModel, useVueTable, type SortingState } from '@tanstack/vue-table';
import type { LedgerEntry, LedgerInput, LedgerKind, PoolView, SkillView } from '../types';
import { INCOME_KINDS, KIND_LABEL, SPEND_KINDS, isSpend } from '../pool';
import { pts } from '../plural';

const props = defineProps<{ pool: PoolView; skills: SkillView[] }>();
const emit = defineEmits<{ add: [LedgerInput]; update: [number, LedgerInput]; remove: [number]; setLimit: [number | null] }>();

const signedPts = (n: number) => (n > 0 ? `+${pts(n)}` : pts(n));
const skillName = (id?: string | null) => (id ? props.skills.find((s) => s.id === id)?.name ?? id : '');

const col = createColumnHelper<LedgerEntry>();
const sorting = ref<SortingState>([]);
const columns = [
  col.accessor('date', { header: 'Дата', cell: (c) => c.getValue() || '—' }),
  col.accessor('kind', { header: 'Вид', cell: (c) => KIND_LABEL[c.getValue()] }),
  col.accessor('amount', { header: 'Очки', meta: { num: true },
    cell: (c) => h('span', { class: c.getValue() < 0 ? 'amount--spend' : 'amount--income' }, signedPts(c.getValue())) }),
  col.accessor('note', { header: 'Пояснение', enableSorting: false,
    cell: (c) => [c.getValue() || '', c.row.original.skill ? h('span', { class: 'muted' }, ` · навык: ${skillName(c.row.original.skill)}`) : null] }),
  col.display({ id: 'actions', header: '', enableSorting: false }),
];
const table = useVueTable({
  get data() { return props.pool.entries; },
  columns,
  state: { get sorting() { return sorting.value; } },
  onSortingChange: (u) => { sorting.value = typeof u === 'function' ? u(sorting.value) : u; },
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
});

// dialog
const editing = ref<number | null>(null); // id, or 0 = new
const form = ref<{ kind: LedgerKind; amount: number; note: string; date: string; skill: string }>({ kind: 'session-award', amount: 1, note: '', date: '', skill: '' });
const formError = computed(() => (form.value.kind === 'other' && !form.value.note.trim() ? 'Для «Другое» пояснение обязательно'
  : !form.value.amount ? 'Количество очков не может быть 0' : ''));
const today = () => new Date().toLocaleDateString('sv-SE');
function openNew() { form.value = { kind: 'session-award', amount: 1, note: '', date: today(), skill: '' }; editing.value = 0; }
function openEdit(e: LedgerEntry) {
  form.value = { kind: e.kind, amount: e.kind === 'correction' ? e.amount : Math.abs(e.amount), note: e.note ?? '', date: e.date ?? '', skill: e.skill ?? '' };
  editing.value = e.id;
}
function submit() {
  if (formError.value || editing.value === null) return;
  const f = form.value;
  const entry: LedgerInput = { kind: f.kind, amount: f.amount, note: f.note, date: f.date, skill: f.kind === 'training' && f.skill ? f.skill : null };
  if (editing.value === 0) emit('add', entry); else emit('update', editing.value, entry);
  editing.value = null;
}
const limitText = ref('');
function applyLimit() {
  const v = limitText.value.trim(); limitText.value = '';
  if (!v) return emit('setLimit', null);
  const n = Math.abs(parseInt(v, 10)); if (Number.isFinite(n)) emit('setLimit', n);
}
const overLimit = computed(() => props.pool.disadvantages > props.pool.disadvantage_limit);
const tooManyQuirks = computed(() => props.pool.quirks > 5 || props.pool.quirk_points < -5);
</script>

<template>
  <div class="points-view">
    <section class="panel pool-summary" aria-labelledby="pool-title" data-test="pool-summary">
      <h2 id="pool-title" class="panel__title">Очки персонажа</h2>
      <dl class="pool-grid">
        <div><dt>Стартовые</dt><dd class="num">{{ pts(pool.starting) }}</dd></div>
        <div><dt>Заработано в игре</dt><dd class="num">{{ signedPts(pool.income) }}</dd></div>
        <div><dt>Всего заработано</dt><dd class="num"><strong>{{ pts(pool.earned) }}</strong></dd></div>
        <div><dt>Стоимость персонажа</dt><dd class="num" title="Считается по листу, не хранится">{{ pts(pool.character_cost) }}</dd></div>
        <div><dt>Потрачено вне листа</dt><dd class="num">{{ pts(pool.spent_outside) }}</dd></div>
        <div :class="{ 'is-negative': pool.unspent < 0 }" data-test="pool-unspent">
          <dt>Не потрачено</dt><dd class="num"><strong>{{ pts(pool.unspent) }}</strong><span v-if="pool.unspent < 0" class="pool-warn"> ⚠ перерасход</span></dd>
        </div>
      </dl>
      <p class="muted pool-formula">Не потрачено = заработано − стоимость персонажа − траты вне листа.</p>
      <div class="pool-limits">
        <p :class="{ 'is-negative': overLimit }">
          Недостатки: <strong class="num">{{ pts(-pool.disadvantages) }}</strong> из лимита <span class="num">{{ pts(-pool.disadvantage_limit) }}</span>
          <span class="muted">({{ pool.disadvantage_limit_custom ? 'задан вручную' : '50% стартовых' }})</span>
          <span v-if="overLimit"> ⚠ выше лимита</span>
        </p>
        <p :class="{ 'is-negative': tooManyQuirks }">
          Причуды (отдельно от лимита): <strong class="num">{{ pool.quirks }}</strong> на <span class="num">{{ pts(pool.quirk_points) }}</span>
          <span class="muted">· обычно не больше 5 по −1</span><span v-if="tooManyQuirks"> ⚠</span>
        </p>
        <label class="pool-limit-edit">Лимит недостатков:
          <input v-model="limitText" class="field field--num num" type="number" min="0" :placeholder="String(pool.disadvantage_limit)" aria-label="Лимит недостатков в очках" @keydown.enter="applyLimit" />
          <button class="btn" type="button" @click="applyLimit">Задать</button>
          <button v-if="pool.disadvantage_limit_custom" class="btn btn--link" type="button" @click="emit('setLimit', null)">по умолчанию (50%)</button>
        </label>
      </div>
    </section>

    <section class="panel" aria-labelledby="ledger-title">
      <header class="panel__header">
        <h2 id="ledger-title" class="panel__title">Журнал очков</h2>
        <span class="panel__meta muted">{{ pool.entries.length }} зап.</span>
        <button class="btn btn--primary" type="button" data-test="ledger-add" @click="openNew">+ Запись</button>
      </header>
      <table class="data-table ledger-table">
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
          <tr v-for="row in table.getRowModel().rows" :key="row.id" class="data-table__row" :data-test="`ledger-${row.original.id}`">
            <td v-for="cell in row.getVisibleCells()" :key="cell.id" :class="['cell', `cell--${cell.column.id}`, { 'is-num': (cell.column.columnDef.meta as any)?.num }]">
              <template v-if="cell.column.id === 'actions'">
                <button class="btn btn--icon" type="button" :aria-label="`Править запись ${row.original.id}`" title="Править" @click="openEdit(row.original)">✎</button>
                <button class="btn btn--icon" type="button" :aria-label="`Удалить запись ${row.original.id}`" title="Удалить" @click="emit('remove', row.original.id)">✕</button>
              </template>
              <FlexRender v-else :render="cell.column.columnDef.cell" :props="cell.getContext()" />
            </td>
          </tr>
          <tr v-if="!pool.entries.length"><td colspan="5" class="muted">Записей нет — стартовых очков 0.</td></tr>
        </tbody>
      </table>
    </section>

    <div v-if="editing !== null" class="modal-backdrop" @click.self="editing = null" @keydown.esc.stop="editing = null">
      <form class="modal ledger-dialog" role="dialog" aria-modal="true" aria-labelledby="ledger-dlg-title" data-test="ledger-dialog" @submit.prevent="submit">
        <h2 id="ledger-dlg-title" class="panel__title">{{ editing === 0 ? 'Новая запись' : 'Правка записи' }}</h2>
        <label>Вид
          <select v-model="form.kind" class="field field--select" data-test="ledger-kind">
            <optgroup label="Доход">
              <option v-for="k in INCOME_KINDS" :key="k" :value="k">{{ KIND_LABEL[k] }}</option>
            </optgroup>
            <optgroup label="Траты вне листа">
              <option v-for="k in SPEND_KINDS" :key="k" :value="k">{{ KIND_LABEL[k] }}</option>
            </optgroup>
          </select>
        </label>
        <label>{{ form.kind === 'correction' ? 'Очки (со знаком)' : isSpend(form.kind) ? 'Потрачено очков' : 'Получено очков' }}
          <input v-model.number="form.amount" class="field field--num num" type="number" data-test="ledger-amount" />
        </label>
        <label>Дата <input v-model="form.date" class="field" type="date" /></label>
        <label v-if="form.kind === 'training'">Навык (необязательно)
          <select v-model="form.skill" class="field field--select">
            <option value="">—</option>
            <option v-for="s in skills" :key="s.index" :value="s.id">{{ s.name }}</option>
          </select>
        </label>
        <label>Пояснение{{ form.kind === 'other' ? ' (обязательно)' : '' }}
          <input v-model="form.note" class="field" type="text" placeholder="например, подарок от Мастера" data-test="ledger-note" />
        </label>
        <p v-if="formError" class="alert alert--warning" role="status">{{ formError }}</p>
        <div class="ledger-dialog__actions">
          <button class="btn" type="button" @click="editing = null">Отмена</button>
          <button class="btn btn--primary" type="submit" :disabled="!!formError" data-test="ledger-save">Сохранить</button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.points-view { display: flex; flex-direction: column; gap: var(--md); }
.pool-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: var(--sm); margin: 0; }
.pool-grid > div { border: 1px solid var(--color-border); border-radius: var(--radiusSm); padding: var(--sm); }
.pool-grid dt { color: var(--color-muted); font-size: 0.85em; }
.pool-grid dd { margin: 0; font-size: 1.3em; }
.is-negative, .is-negative dd { color: var(--color-danger); }
.pool-formula { font-size: 0.85em; }
.pool-limits p { margin: var(--xs) 0; }
.pool-limit-edit { display: inline-flex; align-items: center; gap: var(--sm); flex-wrap: wrap; }
.ledger-table { width: 100%; }
.ledger-table :deep(.amount--spend) { color: var(--color-danger); }
.ledger-table :deep(.amount--income) { color: var(--color-success); }
.ledger-dialog { display: flex; flex-direction: column; gap: var(--sm); min-width: min(420px, 90vw); }
.ledger-dialog label { display: flex; flex-direction: column; gap: var(--xs); }
.ledger-dialog__actions { display: flex; justify-content: flex-end; gap: var(--sm); }
@media print {
  .pool-limit-edit, .cell--actions, .ledger-table th:last-child { display: none !important; }
}
</style>
