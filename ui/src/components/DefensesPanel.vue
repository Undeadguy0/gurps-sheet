<script setup lang="ts">
import { computed } from 'vue';
import type { Defense, Defenses } from '../types';
import { engineNames, tx } from '../engineRu';
const fromText = (f: string) => engineNames.value.get(f) ?? tx(f);

const props = defineProps<{ defenses?: Defenses | null }>();
const d = computed(() => props.defenses ?? null);
const items = computed(() => {
  const x = d.value;
  if (!x) return [];
  const r: { id: string; label: string; v: Defense | null | undefined; none: string }[] = [
    { id: 'dodge', label: 'Уклонение', v: x.dodge, none: '' },
    { id: 'parry', label: 'Парирование', v: x.parry, none: 'нет оружия' },
    { id: 'block', label: 'Блок', v: x.block, none: 'нет щита' },
  ];
  return r;
});
</script>

<template>
  <section v-if="d" class="panel defenses-panel" aria-labelledby="defenses-title">
    <header class="panel__header">
      <h2 id="defenses-title" class="panel__title">Активные защиты</h2>
      <span v-if="d.bonus" class="panel__meta muted num">+{{ d.bonus }} ко всем (Боевые рефлексы и т. п.)</span>
    </header>
    <dl class="defenses">
      <div v-for="i in items" :key="i.id" class="defenses__item" :title="i.v?.explain.map(tx).join('\n')">
        <dt>{{ i.label }}</dt>
        <dd>
          <strong class="defenses__value num">{{ i.v ? i.v.value : '—' }}</strong>
          <span v-if="i.v && d.db" class="muted num" title="Бонус защиты щита против атак спереди"> +{{ d.db }} DB</span>
          <small class="muted">{{ i.v ? fromText(i.v.from) : i.none }}</small>
        </dd>
      </div>
      <div v-if="d.db" class="defenses__item">
        <dt>DB щита</dt>
        <dd><strong class="defenses__value num">+{{ d.db }}</strong><small class="muted">{{ d.db_from }} · только против атак спереди</small></dd>
      </div>
    </dl>
    <table v-if="d.armor.length" class="data-table armor-table">
      <thead><tr><th scope="col">Броня</th><th scope="col" class="is-num">DR</th></tr></thead>
      <tbody><tr v-for="(a, i) in d.armor" :key="i"><td>{{ a.item }}</td><td class="is-num num">{{ a.dr }}</td></tr></tbody>
    </table>
    <p v-else class="muted">Брони с DR нет.</p>
  </section>
</template>
