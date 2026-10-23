<script setup lang="ts">
import type { StatRow } from '../types';
import { fmt, signed } from '../labels';
import { tx } from '../engineRu';

const props = defineProps<{ row: StatRow }>();
const emit = defineEmits<{ close: [] }>();

function sourceText(kind: string, ref?: string) {
  if (kind === 'purchased') return 'Лист › Атрибуты › куплено за очки';
  if (kind === 'trait') return `Черты › ${ref}`;
  return ref ?? kind;
}
async function copy() {
  const text = props.row.explain.map(tx).join('\n');
  try { await navigator.clipboard.writeText(`${props.row.label}\n${text}`); } catch { /* ignore */ }
}
</script>

<template>
  <aside class="side-panel provenance" role="complementary" aria-labelledby="prov-title" @keydown.esc="emit('close')">
    <header class="side-panel__header">
      <h2 id="prov-title" class="panel__title">Откуда взялось число</h2>
      <button class="btn btn--icon" type="button" aria-label="Закрыть (Esc)" @click="emit('close')">Esc</button>
    </header>
    <p class="muted">{{ row.label }}</p>
    <p class="provenance__total"><span class="num">{{ fmt(row.total) }}</span></p>
    <ol class="provenance__chain">
      <li class="provenance__step">
        <div class="provenance__step-head"><span>База</span><strong class="num">{{ fmt(row.value.base) }}</strong></div>
        <small class="muted">{{ row.value.base_reason === 'default' ? 'значение по умолчанию' : tx(row.value.base_reason) }}</small>
      </li>
      <li v-for="(c, i) in row.value.contributions" :key="i" class="provenance__step" :data-kind="c.source.kind">
        <div class="provenance__step-head"><span>{{ tx(c.reason) }}</span><strong class="num">{{ signed(c.amount) }}</strong></div>
        <small class="muted">Источник: {{ sourceText(c.source.kind, 'ref' in c.source ? c.source.ref : undefined) }}</small>
      </li>
    </ol>
    <div class="provenance__sum"><span>Итого</span><strong class="num">{{ fmt(row.total) }}</strong></div>
    <footer class="side-panel__footer">
      <button class="btn" type="button" @click="copy">Копировать расчёт</button>
    </footer>
  </aside>
</template>
