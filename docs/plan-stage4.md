# Этап 4: gurps-rules -> gurps-calc (план и состояние)

Ветка `feat/db-storage`. Расчёт — только `gurps-calc` (mcp-сервер, `crates/gurps-calc`), все константы правил — из `ref_*` БД.

## Сделано (mcp-сервер, feat/gurps-db)

- 70cf15f: `gurps-calc::encumbrance` — уровни нагрузки из `ref_encumbrance`, Move/Dodge под нагрузкой
  (`ref_rule` move_attribute, encumbrance_move_min, dodge_attribute, dodge_base, dodge_min),
  `Defenses` (Dodge + бонус, лучшее Parry: оружие предметов, иначе природные атаки).
  `Computed.encumbrance/defenses`; загрузчик gurps-db: `refs.encumbrance`, `carried_weight`
  (сумма qty × item.weight по owns_item). Новый сид `ref_rule:move_attribute = 'Move'`.

- c2082d8: опции правил в БД: `ref_rule` trait_cost_negative_rounding (сид toward_positive, -7.5 -> -7),
  disadvantage_limit_percent (50), quirk_limit (5); персонаж перекрывает правила через `Input.rule_overrides`.
- 4ea2fd4: Block: `ref_rule` block_base/block_skill_divisor/block_rounding (только структура, сиды — от Мастера),
  `item.shield.block_skill {skill, spec}`; Block без нагрузки, DB щита к Dodge/Parry/Block.

- 163a59d: sheet-bridge — школа заклинания по коду/имени en из БД.
- cb3e57f: `calc_input::load_doc/doc_of` — вход расчёта из документа персонажа `(pack, sid)`.
- 06e1109: встроенные определения (`inline`) в документе рядом со ссылками `(pack, sid)` — своё
  содержимое листа (решение: хранится в документе персонажа, не в каталоге; на этапе 5 — в записи персонажа).
- 042a111: gurps-calc — `attributes.rows` (база, шаги, шаг, цена шага, очки, эффекты), `points.perks/quirks/quirk_count`,
  `equipment` (вес/стоимость всего и при себе, DR надетой брони по зонам), `max_level` техник,
  `SkillLevel::is_default()`; «при себе» в БД-источнике учитывает контейнеры.

## Сделано (gurps-sheet)

- 40d1664: броски — крейт приложения `gurps-dice`.
- 9494fce…16af44f: `gurps-library/tests/golden_db.rs` — golden на БД (ожидания прежние), документ = БД.
- fca5a13, 39b6cd6, 8da7fbf: `gurps_library::calc` — лист -> документ (каталог или inline), `rule_overrides`,
  лимиты недостатков/причуд из `ref_rule` (`validate`), `Engine` (синхронный расчёт для приложения).
- b405f92: SheetView.calc / calc_error / calc_issues рядом со старым `sheet`; TS-тип `Computed`.

## Осталось

1. gurps-calc: explain-строки (навыки, урон, атаки, защиты, стоимость черт) — сейчас только структурные данные;
   inline-предметы без оружия/щита, inline-заклинания — школа как slug, у inline нет пререквизитов.
2. UI с `sheet` на `calc` (подписи атрибутов — переводы БД), удалить старый путь.
3. Модель персонажа `(pack, sid)` + inline, gurps-io v2 (старый JSON не читается).
4. Удалить `gurps-rules`, `db_adapter`, `legacy-plugins`, `gurps-plugins`, `plugins/` (ftl — только переводы каталога);
   golden.rs/spells_modifiers.rs gurps-cli — покрыты golden_db, удалить вместе с gurps-rules.
5. `warrior_unarmed_pending`: сиды punch/kick (mod 0) и связь техника -> атака (Kicking).
