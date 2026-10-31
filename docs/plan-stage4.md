# Этап 4: gurps-rules -> gurps-calc (план и состояние)

Ветка `feat/db-storage`. Расчёт — только `gurps-calc` (mcp-сервер, `crates/gurps-calc`), все константы правил — из `ref_*` БД.

## Сделано (mcp-сервер, feat/gurps-db)

- 70cf15f: `gurps-calc::encumbrance` — уровни нагрузки из `ref_encumbrance`, Move/Dodge под нагрузкой
  (`ref_rule` move_attribute, encumbrance_move_min, dodge_attribute, dodge_base, dodge_min),
  `Defenses` (Dodge + бонус, лучшее Parry: оружие предметов, иначе природные атаки).
  `Computed.encumbrance/defenses`; загрузчик gurps-db: `refs.encumbrance`, `carried_weight`
  (сумма qty × item.weight по owns_item). Новый сид `ref_rule:move_attribute = 'Move'`.

## Осталось

1. gurps-calc: Block (нужны `ref_rule` block_base/block_skill_divisor и способ указать навыки блока —
   сид/схема), DB щита, DR брони (`parse_db/parse_dr` из gurps-core), explain-строки
   (Value.explain: атрибуты, BL, урон, нагрузка, атаки, стоимость черт с модификаторами).
2. Опции правил листа (`RulesOptions`, `NegativeRounding`) — в `ref_rule` или настройки персонажа.
3. Броски (`gurps_rules::dice`: success roll, damage_roll) — не правила-таблицы; решить: перенести
   в gurps-calc (критические пороги в `ref_rule`) или оставить в приложении.
4. `validate` (лимиты уровней черт и т.п.) -> `gurps-calc::prereq`/новый модуль.
5. Модель персонажа: `gurps-core::Character` -> входная модель с `(pack, sid)` и ref-кодами;
   сборка `gurps_calc::model::Input` из персонажа + каталога (через gurps-library/gurps-db),
   без enum `Stat/Difficulty/TraitCategory`.
6. src-tauri `state.rs` (≈2600 строк, 117 функций) и TS-типы (`ui/src/types.ts`, combat/encumbrance/
   skills/magic) — на `gurps_calc::Computed`; `npm run build`.
7. gurps-io: JSON с `(pack, sid)`; старый JSON не читается.
8. Golden (`gurps-cli/tests/golden.rs`, `spells_modifiers.rs`) — тестовый пак-сид, ожидания прежние.
   Известное изменение правил: Dodge под нагрузкой не ниже 1 (`dodge_min`, B17); раньше минимума не было.
9. Удалить `gurps-rules`, `db_adapter`, feature `legacy-plugins`, `gurps-plugins`, `plugins/`.