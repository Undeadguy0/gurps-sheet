# Этап 3: слой хранения gurps-sheet -> SurrealDB

Ветка `feat/db-storage` (от `main` a5fbb42). Схема БД (`mcp-сервер/Data/database`) первична.

## Решение: где живёт `gurps-db`

`gurps-db` создаётся **в репозитории mcp-сервер** (`crates/gurps-db`, ветка `feat/gurps-db`):
из сервера туда вынесены `Db`/`DbConfig`/загрузчик входа расчёта; добавлено чтение каталога
(паки по priority, записи `(pack, sid)`, переводы, `ref_*`, `effective`) и `testkit`
(временная БД: `surreal start memory` + `surrealkit sync/seed`). Сервер и `sheet-bridge`
используют его — одна реализация запросов для MCP, моста и листа.

gurps-sheet подключает его path-зависимостью
`../../ПРОЕКТЫ/mcp-сервер/crates/gurps-db` (относительно корня workspace).
Плюсы: нет дублирования, изменения схемы и запросов видны обоим сразу.
Минусы: сборка листа требует соседнего checkout'а mcp-сервер по фиксированному
относительному пути (кириллица в пути), версии не закреплены.
Альтернативы на будущее: (а) git-зависимость `gurps-db = { git = "...", rev = "..." }`
после публикации репозитория; (б) общий workspace/монорепо; (в) `[patch]` в
`.cargo/config.toml` для локальной разработки поверх git-зависимости. Рекомендация —
(а)+(в), когда у mcp-сервер появится remote.

## Что делается на этапе 3

1. mcp-сервер: крейт `gurps-db` (готово отдельным коммитом).
2. gurps-sheet: крейт `gurps-library` — источник каталога для приложения:
   `LibrarySource::Db(DbConfig)` (рабочий режим; env `SURREALDB_*`, общий сервер с gurps-mcp,
   только чтение каталога). Ключ библиотеки — код пака из БД (`basic`, `homebrew`), а не id плагина.
3. src-tauri: `AppState` берёт каталог из `gurps-library`; зависимость от `gurps-plugins` у
   приложения убрана; экспорт «Своего» в папку плагина удалён (каталог правится только
   сидами/MCP). JSON-файл персонажа (`gurps-io`) остаётся как формат обмена (экспорт/импорт).

## Временные адаптеры (снимаются на этапе 4)

- `gurps-library::db_adapter`: записи БД -> старые типы `gurps-core` (`TraitDef`, `SkillDef`, …)
  через проекцию `sheet-bridge` (`catalog::export::to_sheet`) + serde-перекладку. Нужен, пока
  расчёт (`gurps-rules`) и UI работают на типах `gurps-core` с enum'ами
  `Stat/Difficulty/TraitCategory` и kebab-id.
- `gurps-library` feature `legacy-plugins` (`LibrarySource::Plugins`): загрузка TOML-плагинов
  через `gurps-plugins` — **только для тестов** (state-тесты src-tauri, golden). В приложении
  не используется.
- `gurps-plugins` остаётся в workspace только как dev-зависимость тестов.

## Вне этапа

- Этап 4: `gurps-rules` -> `gurps-calc`, замена enum'ов на ref-коды, id `(pack, sid)` в модели
  персонажа, переработка golden-тестов, удаление адаптеров и `gurps-plugins`.
- Этап 5: персонаж в БД (рёбра has_*/owns_item, point_ledger).

## Проверка

`cargo build/clippy/test` в обоих репозиториях (`CARGO_BUILD_JOBS=2`); DB-тесты — на временной
SurrealDB 3.3 в памяти (свободный порт, sync+seed через surrealkit, процесс гасится в Drop).