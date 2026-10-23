<script setup lang="ts">
// «За столом»: compact play view. HP/FP edits go through the undoable
// set_current command; rolls are not character edits and only feed the log.
import { computed, ref } from 'vue';
import { api } from '../api';
import type { Outcome, SheetView } from '../types';
import { OUTCOME_LABEL, STAT_LABEL, fmt, signed } from '../labels';
import { tx } from '../engineRu';

const props = defineProps<{ view: SheetView }>();
const emit = defineEmits<{ setCurrent: ['HP' | 'FP', number | null] }>();

interface LogEntry { id: number; title: string; total: number; detail: string; outcome?: Outcome; margin?: number }
const log = ref<LogEntry[]>([]);
const modifier = ref(0);
const filter = ref('');
const shieldFront = ref(true);
let seq = 0;

const session = computed(() => props.view.session ?? { hp: 10, hp_max: 10, fp: 10, fp_max: 10 });
const trackers = computed(() => {
  const s = session.value;
  return ([['HP', 'Единицы жизни (HP)', s.hp, s.hp_max], ['FP', 'Единицы усталости (FP)', s.fp, s.fp_max]] as const).map(([key, label, cur, max]) => ({
    key, label, cur, max,
    pct: Math.max(0, Math.min(100, (cur / Math.max(1, max)) * 100)),
    // Own summary of the low-HP/FP effects; check the book (# unverified B419, B426).
    note: cur <= 0 ? (key === 'HP' ? '0 и ниже: проверки HT, чтобы не потерять сознание' : '0 и ниже: каждое усилие стоит HP, риск обморока')
      : cur < max / 3 ? (key === 'HP' ? 'меньше 1/3: Движение и Уклонение вдвое меньше' : 'меньше 1/3: Движение, Уклонение и ST вдвое меньше') : '',
  }));
});
function step(key: 'HP' | 'FP', d: number) {
  const s = session.value; emit('setCurrent', key, (key === 'HP' ? s.hp : s.fp) + d);
}
function setTyped(key: 'HP' | 'FP', e: Event) {
  const v = parseInt((e.target as HTMLInputElement).value, 10);
  if (Number.isFinite(v)) emit('setCurrent', key, v);
}

interface Row { id: string; group: string; name: string; level: number; note: string; damage?: { expr: string; type: string } }
const rows = computed<Row[]>(() => {
  const v = props.view; const s = v.sheet; const r: Row[] = [];
  const d = s.defenses;
  if (d) {
    const db = shieldFront.value ? d.db : 0;
    r.push({ id: 'def-dodge', group: 'Защита', name: 'Уклонение', level: d.dodge.value + db, note: db ? `вкл. DB +${db}` : tx(d.dodge.from) });
    if (d.parry) r.push({ id: 'def-parry', group: 'Защита', name: 'Парирование', level: d.parry.value + db, note: db ? `${d.parry.from} · DB +${db}` : d.parry.from });
    if (d.block) r.push({ id: 'def-block', group: 'Защита', name: 'Блок', level: d.block.value + db, note: db ? `${d.block.from} · DB +${db}` : d.block.from });
  }
  for (const w of s.weapons ?? []) for (const a of w.attacks) {
    if (a.level == null) continue;
    const name = v.weapons?.find((x) => x.index === w.index)?.name ?? w.name;
    r.push({ id: `atk-${w.index}-${a.mode}`, group: 'Атака', name: `${name} (${a.mode})`, level: a.level,
      note: `${a.damage} ${a.damage_type}${a.reach ? ` · досяг. ${a.reach}` : ''}`, damage: { expr: a.damage, type: a.damage_type } });
  }
  for (const k of v.skills) if (k.level != null) r.push({ id: `sk-${k.index}`, group: 'Навык', name: k.specialization ? `${k.name} (${k.specialization})` : k.name, level: k.level, note: k.relative ?? '' });
  for (const k of v.techniques) if (k.level != null) r.push({ id: `te-${k.index}`, group: 'Приём', name: k.name, level: k.level, note: k.relative ?? '' });
  for (const k of v.spells ?? []) if (k.level != null) r.push({ id: `sp-${k.index}`, group: 'Заклинание', name: k.name, level: k.level, note: k.relative ?? '' });
  for (const a of [...s.attributes, ...s.secondary]) if (['ST', 'DX', 'IQ', 'HT', 'Will', 'Per'].includes(a.key))
    r.push({ id: `at-${a.key}`, group: 'Атрибут', name: STAT_LABEL[a.key] ?? a.key, level: Math.floor(a.total), note: '' });
  const f = filter.value.trim().toLowerCase();
  return f ? r.filter((x) => `${x.group} ${x.name}`.toLowerCase().includes(f)) : r;
});

function push(e: Omit<LogEntry, 'id'>) { log.value = [{ ...e, id: ++seq }, ...log.value].slice(0, 100); }
async function roll(r: Row) {
  const res = await api.roll(r.level, modifier.value);
  push({ title: `${r.name} vs ${res.effective}`, total: res.roll, outcome: res.outcome, margin: res.margin,
    detail: `${res.dice.join('+')} · база ${r.level}${res.modifier ? ` · мод. ${signed(res.modifier)}` : ''}` });
}
async function rollDamage(r: Row) {
  if (!r.damage) return;
  const d = await api.rollDamage(r.damage.expr, r.damage.type.startsWith('cr'));
  push({ title: `Урон ${r.name}: ${d.expr}`, total: d.total, detail: `${d.dice.join('+')}${d.adds ? ` ${signed(d.adds)}` : ''} · ${r.damage.type}` });
}
const enc = computed(() => props.view.sheet.encumbrance);
</script>

<template>
  <div class="table-mode" data-test="table-mode">
    <div class="table-mode__col">
      <section v-for="t in trackers" :key="t.key" class="panel tracker" :data-test="`tracker-${t.key}`">
        <h2 class="panel__title">{{ t.label }}</h2>
        <div class="tracker__row">
          <button class="btn tracker__btn" type="button" :aria-label="`${t.key} −1`" @click="step(t.key, $event.shiftKey ? -5 : -1)">−</button>
          <label class="tracker__value num">
            <span class="visually-hidden">Текущие {{ t.key }}</span>
            <input class="field tracker__input num" type="number" :value="t.cur" @change="setTyped(t.key, $event)" />
            <span class="muted">/ {{ t.max }}</span>
          </label>
          <button class="btn tracker__btn" type="button" :aria-label="`${t.key} +1`" @click="step(t.key, $event.shiftKey ? 5 : 1)">+</button>
        </div>
        <div class="tracker__bar" role="meter" :aria-valuenow="t.cur" :aria-valuemin="0" :aria-valuemax="t.max" :aria-label="t.label">
          <div class="tracker__fill" :class="{ 'is-low': t.cur < t.max / 3 }" :style="{ width: `${t.pct}%` }"></div>
          <div class="tracker__tick" style="left: 33.33%"></div>
        </div>
        <div class="tracker__foot">
          <small class="muted">{{ t.note || 'Shift+клик по ± — по 5' }}</small>
          <button class="btn btn--link" type="button" :disabled="t.cur === t.max" @click="emit('setCurrent', t.key, null)">Сброс</button>
        </div>
      </section>
      <section v-if="enc" class="panel" aria-labelledby="tm-enc">
        <h2 id="tm-enc" class="panel__title">Нагрузка</h2>
        <p class="tracker__enc"><strong>{{ tx(enc.name) }}</strong> <span class="muted num">({{ enc.level }}) · {{ fmt(enc.carried_weight) }} фнт</span></p>
        <p class="muted num">Движение {{ enc.move_value }} · Уклонение {{ enc.dodge }}{{ enc.dodge_penalty ? ` (${signed(-enc.dodge_penalty)})` : '' }}</p>
      </section>
    </div>

    <section class="panel table-mode__col" aria-labelledby="tm-rolls">
      <header class="panel__header">
        <h2 id="tm-rolls" class="panel__title">Быстрые броски</h2>
        <input v-model="filter" class="field" type="search" placeholder="Фильтр" aria-label="Фильтр бросков" />
      </header>
      <div class="table-mode__opts">
        <label>Модификатор <input v-model.number="modifier" class="field field--num num" type="number" data-test="roll-mod" /></label>
        <label v-if="view.sheet.defenses?.db"><input v-model="shieldFront" type="checkbox" /> Атака спереди (+DB щита)</label>
      </div>
      <ul class="quick-rolls">
        <li v-for="r in rows" :key="r.id" class="quick-rolls__row" :data-test="`qr-${r.id}`">
          <div><strong>{{ r.name }}</strong><br /><small class="muted">{{ r.group }}{{ r.note ? ` · ${r.note}` : '' }}</small></div>
          <span class="quick-rolls__level num">{{ r.level }}</span>
          <button class="btn btn--roll" type="button" :title="`3d6 против ${r.level + modifier}`" @click="roll(r)">3d6</button>
          <button v-if="r.damage" class="btn btn--roll" type="button" :title="`Урон ${r.damage.expr} ${r.damage.type}`" @click="rollDamage(r)">урон</button>
          <span v-else></span>
        </li>
        <li v-if="!rows.length" class="muted">Нечего бросать.</li>
      </ul>
    </section>

    <section class="panel table-mode__col" aria-labelledby="tm-log">
      <header class="panel__header">
        <h2 id="tm-log" class="panel__title">Журнал бросков</h2>
        <button v-if="log.length" class="btn btn--link" type="button" @click="log = []">Очистить</button>
      </header>
      <ol class="roll-log" aria-live="polite" data-test="roll-log">
        <li v-for="e in log" :key="e.id" class="roll-log__item" :class="e.outcome?.startsWith('critical') && 'is-critical'">
          <small class="muted">{{ e.title }}</small>
          <div class="roll-log__line">
            <strong class="roll-log__total num">{{ e.total }}</strong>
            <span v-if="e.outcome" class="roll-result" :class="`roll-result--${e.outcome}`">
              {{ OUTCOME_LABEL[e.outcome] }}, запас {{ signed(e.margin ?? 0) }}
            </span>
          </div>
          <small class="muted num">{{ e.detail }}</small>
        </li>
        <li v-if="!log.length" class="muted">Бросков пока не было. Крит. успех: 3–4, 5 при уровне 15+, 6 при 16+; крит. провал: 18, 17 при уровне до 15, на 10+ хуже уровня (B348 # unverified).</li>
      </ol>
    </section>
  </div>
</template>

<style scoped>
.table-mode { display: grid; grid-template-columns: minmax(240px, 1fr) minmax(300px, 1.3fr) minmax(240px, 1fr); gap: var(--md); align-items: start; }
.table-mode__col { display: flex; flex-direction: column; gap: var(--md); min-width: 0; }
.tracker__row { display: flex; align-items: center; justify-content: space-between; gap: var(--sm); }
.tracker__btn { min-width: 44px; min-height: 44px; font-size: 1.4em; }
.tracker__value { display: flex; align-items: baseline; gap: var(--xs); font-size: 1.8em; font-weight: 700; }
.tracker__input { width: 3.2em; font-size: inherit; font-weight: inherit; text-align: right; }
.tracker__bar { position: relative; height: 12px; margin: var(--sm) 0 var(--xs); border: 1px solid var(--color-border); border-radius: 6px; overflow: hidden; }
.tracker__fill { height: 100%; background: var(--color-success); }
.tracker__fill.is-low { background: var(--color-danger); }
.tracker__tick { position: absolute; top: 0; bottom: 0; width: 1px; background: var(--color-text); }
.tracker__foot { display: flex; justify-content: space-between; gap: var(--sm); }
.table-mode__opts { display: flex; flex-wrap: wrap; gap: var(--md); margin-bottom: var(--sm); }
.quick-rolls, .roll-log { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--xs); }
.quick-rolls__row { display: grid; grid-template-columns: 1fr auto auto auto; align-items: center; gap: var(--sm); padding: var(--xs) var(--sm); border: 1px solid var(--color-border); border-radius: var(--radiusSm); }
.quick-rolls__level { font-size: 1.3em; font-weight: 700; min-width: 2ch; text-align: right; }
.roll-log__item { padding: var(--xs) var(--sm); border: 1px solid var(--color-border); border-radius: var(--radiusSm); }
.roll-log__item.is-critical { border-width: 2px; border-color: var(--color-text); }
.roll-log__line { display: flex; align-items: baseline; justify-content: space-between; gap: var(--sm); }
.roll-log__total { font-size: 1.6em; }
@media (max-width: 900px) { .table-mode { grid-template-columns: 1fr; } }
</style>
