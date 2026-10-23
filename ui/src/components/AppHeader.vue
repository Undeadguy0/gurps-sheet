<script setup lang="ts">
import { ref, watch } from 'vue';
import type { SheetView } from '../types';
import { THEMES, theme, scheme } from '../theme';
import { inTauri } from '../api';

const props = defineProps<{ view: SheetView; dirty: boolean; status: string }>();
const emit = defineEmits<{
  rename: [string]; save: [string | undefined]; load: [string]; 'new': []; help: [];
}>();

const name = ref(props.view.sheet.name);
watch(() => props.view.sheet.name, (n) => (name.value = n));
const print = () => window.print();
const path = ref(props.view.path ?? 'examples/hero.json');
watch(() => props.view.path, (p) => { if (p) path.value = p; });
</script>

<template>
  <header class="app-header" role="banner">
    <div class="app-header__identity">
      <span class="app-logo" aria-hidden="true">G</span>
      <input
        class="field field--title" :value="name" aria-label="Имя персонажа"
        @change="emit('rename', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <div class="points-summary" role="status" aria-label="Очки персонажа">
      <strong class="points-summary__main num">
        Очки: {{ view.budget }} / {{ view.sheet.points.total }} /
        <span :class="{ 'is-danger': view.sheet.points.unspent < 0 }">{{ view.sheet.points.unspent }} своб.</span>
      </strong>
      <dl class="points-summary__parts">
        <div><dt>Атр</dt><dd class="num">{{ view.sheet.points.attributes + view.sheet.points.secondary }}</dd></div>
        <div><dt>Преим</dt><dd class="num">{{ view.sheet.points.advantages + view.sheet.points.perks }}</dd></div>
        <div><dt>Недост</dt><dd class="num">{{ view.sheet.points.disadvantages }}</dd></div>
        <div><dt>Причуды</dt><dd class="num">{{ view.sheet.points.quirks }}</dd></div>
        <div><dt>Навыки</dt><dd class="num">{{ view.sheet.points.skills }}</dd></div>
      </dl>
    </div>

    <div class="app-header__file">
      <button class="btn" type="button" @click="emit('new')">Новый</button>
      <input v-if="inTauri" v-model="path" class="field field--path" aria-label="Путь к файлу" />
      <button v-if="inTauri" class="btn" type="button" @click="emit('load', path)">Открыть</button>
      <span class="save-status" :class="{ 'is-dirty': dirty }">
        <span class="save-status__dot" aria-hidden="true"></span>{{ status }}
      </span>
      <button class="btn" :class="{ 'btn--primary': dirty }" type="button" :disabled="!inTauri" @click="emit('save', path)">Сохранить</button>
    </div>

    <div class="app-header__prefs">
      <label class="visually-hidden" for="theme-select">Тема</label>
      <select id="theme-select" v-model="theme" class="field field--select">
        <option v-for="t in THEMES" :key="t.id" :value="t.id">{{ t.label }}</option>
      </select>
      <button class="btn btn--icon" type="button" :aria-pressed="scheme === 'dark'"
        :title="scheme === 'dark' ? 'Светлая схема' : 'Тёмная схема'" @click="scheme = scheme === 'dark' ? 'light' : 'dark'">
        {{ scheme === 'dark' ? '☀' : '☾' }}
      </button>
      <button class="btn btn--icon" type="button" title="Печать / PDF" @click="print">⎙</button>
      <button class="btn btn--icon" type="button" title="Шпаргалка клавиш (?)" @click="emit('help')">?</button>
    </div>
  </header>
</template>
