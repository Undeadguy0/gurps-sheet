<script setup lang="ts">
import { remaining } from '../plural';
import { ref, watch } from 'vue';
import type { LocaleInfo, SheetView, NegativeRounding } from '../types';
import { THEMES, theme, scheme } from '../theme';
import { inTauri } from '../api';

const props = defineProps<{ view: SheetView; dirty: boolean; status: string; locales?: LocaleInfo | null }>();
const emit = defineEmits<{
  rename: [string]; setLocale: [string, boolean]; save: [string | undefined]; load: [string]; 'new': []; help: []; undo: []; redo: []; setRounding: [NegativeRounding];
}>();

const name = ref(props.view.sheet.name);
watch(() => props.view.sheet.name, (n) => (name.value = n));
const print = () => window.print();
const LANG_LABEL: Record<string, string> = { ru: 'Русский', en: 'English' };
const ORIGINAL = '__original__';
const langValue = () => (props.locales?.original_only ? ORIGINAL : props.locales?.locale ?? 'ru');
function onLang(v: string) {
  if (v === ORIGINAL) emit('setLocale', props.locales?.locale ?? 'ru', true);
  else emit('setLocale', v, false);
}
const path = ref(props.view.path ?? 'character.gurps.json');
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
        Бюджет {{ view.budget }} · потрачено {{ view.sheet.points.total }} ·
        <span :class="{ 'is-danger': view.sheet.points.unspent < 0 }">{{ remaining(view.sheet.points.unspent) }}</span>
      </strong>
      <dl class="points-summary__parts">
        <div><dt>Атр</dt><dd class="num">{{ view.sheet.points.attributes + view.sheet.points.secondary }}</dd></div>
        <div><dt>Преим</dt><dd class="num">{{ view.sheet.points.advantages + view.sheet.points.perks }}</dd></div>
        <div><dt>Недост</dt><dd class="num">{{ view.sheet.points.disadvantages }}</dd></div>
        <div><dt>Причуды</dt><dd class="num">{{ view.sheet.points.quirks }}</dd></div>
        <div><dt>Навыки</dt><dd class="num">{{ view.sheet.points.skills }}</dd></div>
        <div v-if="view.sheet.points.spells"><dt>Заклинания</dt><dd class="num">{{ view.sheet.points.spells }}</dd></div>
      </dl>
    </div>

    <div class="app-header__file">
      <button class="btn" type="button" data-test="undo" :disabled="!view.history?.can_undo"
        :title="view.history?.undo_label ? `Отменить: ${view.history.undo_label} (Ctrl+Z)` : 'Нечего отменять'"
        :aria-label="view.history?.undo_label ? `Отменить: ${view.history.undo_label}` : 'Отменить'" @click="emit('undo')">↶ Отменить</button>
      <button class="btn" type="button" data-test="redo" :disabled="!view.history?.can_redo"
        :title="view.history?.redo_label ? `Повторить: ${view.history.redo_label} (Ctrl+Shift+Z, Ctrl+Y)` : 'Нечего повторять'"
        :aria-label="view.history?.redo_label ? `Повторить: ${view.history.redo_label}` : 'Повторить'" @click="emit('redo')">↷ Вернуть</button>
      <button class="btn" type="button" @click="emit('new')">Новый</button>
      <input v-if="inTauri" v-model="path" class="field field--path" aria-label="Путь к файлу" />
      <button v-if="inTauri" class="btn" type="button" @click="emit('load', path)">Открыть</button>
      <span class="save-status" :class="{ 'is-dirty': dirty }">
        <span class="save-status__dot" aria-hidden="true"></span>{{ status }}
      </span>
      <button class="btn" :class="{ 'btn--primary': dirty }" type="button" :disabled="!inTauri" @click="emit('save', path)">Сохранить</button>
    </div>

    <div class="app-header__prefs">
      <label class="visually-hidden" for="lang-select">Язык данных</label>
      <select v-if="locales" id="lang-select" class="field field--select" :value="langValue()" title="Язык названий и пояснений"
        @change="onLang(($event.target as HTMLSelectElement).value)">
        <option v-for="l in locales.available" :key="l" :value="l">{{ LANG_LABEL[l] ?? l }}</option>
        <option :value="ORIGINAL">Только оригинал</option>
      </select>
      <label class="visually-hidden" for="rounding-select">Округление отрицательной стоимости</label>
      <select id="rounding-select" class="field field--select" :value="view.rules?.negative_rounding ?? 'toward_positive'"
        title="Правило персонажа: как округлять дробную отрицательную стоимость черты после модификаторов"
        @change="emit('setRounding', ($event.target as HTMLSelectElement).value as NegativeRounding)">
        <option value="toward_positive">Отриц. стоимость: вверх (−7,5 → −7)</option>
        <option value="by_magnitude">Отриц. стоимость: по модулю (−7,5 → −8)</option>
      </select>
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
