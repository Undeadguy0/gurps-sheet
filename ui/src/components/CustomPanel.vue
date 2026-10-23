<script setup lang="ts">
// «Своё»: user-defined spells, traits, skills and equipment stored in the
// character file. One shared form; fields depend on the kind.
import { computed, ref } from 'vue';
import type { CustomContent, CustomKind, Difficulty, TraitCategory } from '../types';
import { renderMarkdown } from '../markdown';

const props = defineProps<{ custom: CustomContent; canExport: boolean }>();
const emit = defineEmits<{ save: [CustomKind, Record<string, unknown>]; remove: [CustomKind, string]; export: [] }>();

const KIND_LABEL: Record<CustomKind, string> = { spell: 'Заклинание', trait: 'Черта', skill: 'Навык', equipment: 'Снаряжение' };
const CATEGORY_LABEL: Record<TraitCategory, string> = { advantage: 'Преимущество', perk: 'Перк', disadvantage: 'Недостаток', quirk: 'Причуда' };
const ATTRS = ['ST', 'DX', 'IQ', 'HT', 'Will', 'Per'];

type Row = { kind: CustomKind; id: string; name: string; def: Record<string, unknown> };
const rows = computed<Row[]>(() => [
  ...(props.custom.spells ?? []).map((d) => ({ kind: 'spell' as const, id: d.id, name: d.name, def: d as unknown as Record<string, unknown> })),
  ...(props.custom.traits ?? []).map((d) => ({ kind: 'trait' as const, id: d.id, name: d.name, def: d as unknown as Record<string, unknown> })),
  ...(props.custom.skills ?? []).map((d) => ({ kind: 'skill' as const, id: d.id, name: d.name, def: d as unknown as Record<string, unknown> })),
  ...(props.custom.equipment ?? []).map((d) => ({ kind: 'equipment' as const, id: d.id, name: d.name, def: d as unknown as Record<string, unknown> })),
]);

const blank = () => ({ id: '', name: '', college: '', difficulty: 'H' as Difficulty, attribute: 'IQ', prerequisites: '', description: '',
  category: 'advantage' as TraitCategory, cost: 0, cost_per_level: 0, max_levels: null as number | null, weight: 0, money: 0 });
const kind = ref<CustomKind | null>(null);
const f = ref(blank());
const preview = ref(false);
function openNew(k: CustomKind) { kind.value = k; f.value = blank(); if (k === 'skill') f.value.difficulty = 'A'; preview.value = false; }
const text = (v: unknown) => (typeof v === 'string' ? v : v && typeof v === 'object' ? (v as Record<string, string>).ru ?? Object.values(v)[0] ?? '' : '');
function openEdit(r: Row) {
  const d = r.def; const b = blank();
  f.value = { ...b, id: r.id, name: r.name, description: text(d.description), prerequisites: String(d.prerequisites ?? ''),
    college: ((d.college as string[]) ?? []).join(', '), difficulty: (d.difficulty as Difficulty) ?? b.difficulty, attribute: String(d.attribute ?? 'IQ'),
    category: (d.category as TraitCategory) ?? b.category, cost: r.kind === 'equipment' ? b.cost : Number(d.cost ?? 0),
    cost_per_level: Number(d.cost_per_level ?? 0), max_levels: (d.max_levels as number) ?? null, weight: Number(d.weight ?? 0),
    money: r.kind === 'equipment' ? Number(d.cost ?? 0) : 0 };
  kind.value = r.kind; preview.value = false;
}
const nameError = computed(() => (!f.value.name.trim() ? 'Нужно название' : ''));
function submit() {
  if (!kind.value || nameError.value) return;
  const v = f.value; const base: Record<string, unknown> = { id: v.id, name: v.name.trim(), description: v.description.trim() || null };
  const def: Record<string, unknown> =
    kind.value === 'spell' ? { ...base, attribute: 'IQ', difficulty: v.difficulty, prerequisites: v.prerequisites.trim() || null,
      college: v.college.split(',').map((s) => s.trim()).filter(Boolean) }
    : kind.value === 'skill' ? { ...base, attribute: v.attribute, difficulty: v.difficulty }
    : kind.value === 'trait' ? { ...base, category: v.category, cost: v.cost, cost_per_level: v.cost_per_level, max_levels: v.cost_per_level ? v.max_levels : null, features: [] }
    : { ...base, weight: v.weight, cost: v.money };
  emit('save', kind.value, def);
  kind.value = null;
}
const summary = (r: Row) => {
  const d = r.def;
  if (r.kind === 'spell') return `${((d.college as string[]) ?? []).join(', ') || 'без школы'} · IQ/${d.difficulty}`;
  if (r.kind === 'skill') return `${d.attribute}/${d.difficulty}`;
  if (r.kind === 'trait') return `${CATEGORY_LABEL[d.category as TraitCategory]} · ${d.cost}${d.cost_per_level ? ` + ${d.cost_per_level}/ур.` : ''}`;
  return `${d.weight} фнт · $${d.cost}`;
};
</script>

<template>
  <section class="panel custom-panel" aria-labelledby="custom-title">
    <header class="panel__header">
      <h2 id="custom-title" class="panel__title">Своё</h2>
      <span class="panel__meta muted">хранится в файле персонажа; появляется в библиотеках с пометкой «своё»</span>
    </header>
    <div class="custom-actions">
      <button v-for="k in (['spell', 'trait', 'skill', 'equipment'] as CustomKind[])" :key="k" class="btn" type="button" :data-test="`custom-new-${k}`" @click="openNew(k)">
        + {{ k === 'spell' ? 'Своё заклинание' : k === 'trait' ? 'Своя черта' : k === 'skill' ? 'Свой навык' : 'Своё снаряжение' }}
      </button>
      <button v-if="canExport && rows.length" class="btn" type="button" title="Записать в папку пользовательских плагинов" @click="emit('export')">Экспорт в плагин</button>
    </div>
    <table v-if="rows.length" class="data-table custom-table">
      <thead><tr><th scope="col">Вид</th><th scope="col">Название</th><th scope="col">Параметры</th><th scope="col"></th></tr></thead>
      <tbody>
        <tr v-for="r in rows" :key="r.id" class="data-table__row" :data-test="`custom-${r.id}`">
          <td>{{ KIND_LABEL[r.kind] }}</td>
          <td>{{ r.name }} <span class="badge badge--custom">своё</span></td>
          <td class="muted">{{ summary(r) }}</td>
          <td class="cell--actions">
            <button class="btn btn--icon" type="button" :aria-label="`Править: ${r.name}`" @click="openEdit(r)">✎</button>
            <button class="btn btn--icon" type="button" :aria-label="`Удалить: ${r.name}`" @click="emit('remove', r.kind, r.id)">✕</button>
          </td>
        </tr>
      </tbody>
    </table>
    <p v-else class="muted">Своих определений пока нет.</p>

    <div v-if="kind" class="modal-backdrop" @click.self="kind = null" @keydown.esc.stop="kind = null">
      <form class="modal custom-dialog" role="dialog" aria-modal="true" aria-labelledby="custom-dlg-title" data-test="custom-dialog" @submit.prevent="submit">
        <h2 id="custom-dlg-title" class="panel__title">{{ f.id ? 'Правка' : 'Новое' }}: {{ KIND_LABEL[kind].toLowerCase() }}</h2>
        <label>Название <input v-model="f.name" class="field" type="text" data-test="custom-name" /></label>
        <template v-if="kind === 'spell'">
          <label>Школа (через запятую) <input v-model="f.college" class="field" type="text" placeholder="Огонь" /></label>
          <label>Сложность
            <select v-model="f.difficulty" class="field field--select"><option value="H">Трудная (IQ/H)</option><option value="VH">Очень трудная (IQ/VH)</option></select>
          </label>
          <label>Предварительные условия <input v-model="f.prerequisites" class="field" type="text" /></label>
        </template>
        <template v-else-if="kind === 'skill'">
          <div class="custom-row">
            <label>Атрибут <select v-model="f.attribute" class="field field--select"><option v-for="a in ATTRS" :key="a">{{ a }}</option></select></label>
            <label>Сложность <select v-model="f.difficulty" class="field field--select"><option v-for="d in ['E', 'A', 'H', 'VH']" :key="d">{{ d }}</option></select></label>
          </div>
        </template>
        <template v-else-if="kind === 'trait'">
          <label>Тип <select v-model="f.category" class="field field--select"><option v-for="(l, c) in CATEGORY_LABEL" :key="c" :value="c">{{ l }}</option></select></label>
          <div class="custom-row">
            <label>Стоимость <input v-model.number="f.cost" class="field field--num num" type="number" /></label>
            <label>За уровень <input v-model.number="f.cost_per_level" class="field field--num num" type="number" /></label>
            <label v-if="f.cost_per_level">Макс. уровень <input v-model.number="f.max_levels" class="field field--num num" type="number" min="1" /></label>
          </div>
        </template>
        <template v-else>
          <div class="custom-row">
            <label>Вес, фнт <input v-model.number="f.weight" class="field field--num num" type="number" min="0" step="0.1" /></label>
            <label>Цена, $ <input v-model.number="f.money" class="field field--num num" type="number" min="0" /></label>
          </div>
        </template>
        <label>Пояснение (Markdown)
          <textarea v-if="!preview" v-model="f.description" class="field custom-text" rows="6" data-test="custom-text" />
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div v-else class="md custom-text" v-html="renderMarkdown(f.description)" />
        </label>
        <button class="btn btn--link" type="button" @click="preview = !preview">{{ preview ? 'Редактировать' : 'Предпросмотр' }}</button>
        <p v-if="nameError" class="alert alert--warning" role="status">{{ nameError }}</p>
        <div class="custom-dialog__actions">
          <button class="btn" type="button" @click="kind = null">Отмена</button>
          <button class="btn btn--primary" type="submit" :disabled="!!nameError" data-test="custom-save">Сохранить</button>
        </div>
      </form>
    </div>
  </section>
</template>

<style scoped>
.custom-actions { display: flex; flex-wrap: wrap; gap: var(--sm); margin-bottom: var(--sm); }
.custom-table { width: 100%; }
.custom-dialog { display: flex; flex-direction: column; gap: var(--sm); min-width: min(520px, 92vw); }
.custom-dialog label { display: flex; flex-direction: column; gap: var(--xs); }
.custom-row { display: flex; gap: var(--sm); flex-wrap: wrap; }
.custom-text { min-height: 8em; }
.custom-dialog__actions { display: flex; justify-content: flex-end; gap: var(--sm); }
</style>
