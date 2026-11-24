# GURPS Sheet

Редактор листа персонажа GURPS 4e: ядро на Rust, десктоп-оболочка Tauri v2, интерфейс Vue 3.
Open-source GURPS 4e character editor: Rust core + Tauri v2 + Vue 3 (English summary below).

> GURPS — торговая марка Steve Jackson Games. Проект неофициальный; каталог содержит
> названия, стоимость, ссылки на страницы и краткие пояснения своими словами, без текста правил.
> GURPS is a trademark of Steve Jackson Games. This project is unofficial and ships
> no rules text — the catalog contains names, point costs, page references and short
> explanations written in our own words.

<!-- Скриншоты: положить в docs/screenshots/ (лист в теме «Инструмент», режим «За столом»,
     печать в теме «Бумажный лист») и вставить сюда. -->

## Возможности

- **Лист персонажа:** атрибуты и вторичные характеристики, черты (преимущества, недостатки,
  перки, причуды) с модификаторами, навыки и приёмы, заклинания, снаряжение с нагрузкой,
  оружие (режимы атак предметов и природные атаки), активные защиты.
- **Откуда взялось число:** каждое производное значение — `база + вклады`, интерфейс
  показывает разбор (например, ST 13 = 10 база + 2 купленных + 1 от черты) и пояснения расчёта.
- **Учёт очков:** пул очков с журналом начислений и трат, лимиты недостатков и причуд,
  настраиваемое округление отрицательной стоимости черт.
- **«Своё»:** собственные черты, навыки, заклинания и предметы прямо в листе (хранятся
  встроенными определениями в файле персонажа).
- **Пояснения в Markdown:** у записей каталога есть пояснения; их можно заменить своими прямо
  в листе. Markdown выводится безопасно (HTML экранируется, ссылки только http(s)/mailto).
  Заметки персонажа — тоже Markdown.
- **История:** undo/redo на 200 шагов с названиями действий.
- **Режим «За столом»:** трекеры HP/FP, быстрые броски 3d6 и урона, журнал бросков.
- **Темы:** «Инструмент», «Стол мастера», «Бумажный лист» (она же для печати/PDF);
  генерируются из дизайн-токенов `ui/src/styles/tokens.json` (`npm run gen:themes`).
- **Языки:** переводы названий и пояснений берутся из каталога БД; в шапке — выбор языка и
  режим «Только оригинал», названия показываются как «Перевод (Оригинал)», поиск ищет по обоим.
- **Файл персонажа:** JSON `version: 2`, атомарное сохранение (временный файл + rename).
  Записи ссылаются на каталог как `{pack, sid}` либо несут встроенное определение
  `{inline: {...}}`; файлы версии 1 не читаются.
- **CLI** `gurps` с JSON-выводом: расчёт, проверка и броски.

## Стек

- **Rust** (edition 2021): workspace из крейтов ядра, CLI и приложения Tauri v2.
- **Каталог и расчёт:** общая БД **SurrealDB** и крейты `gurps-db`, `gurps-calc`,
  `sheet-bridge` из соседнего репозитория `mcp-сервер` (см. «Зависимость от mcp-сервер»).
- **UI:** Vue 3 + TypeScript, Vite, TanStack Table, тесты на Vitest.
- **Шрифты** лежат в `ui/public/fonts` (SIL Open Font License 1.1).

## Структура

```text
crates/
  gurps-core/      модель персонажа: ссылки (pack, sid) и встроенные определения, журнал очков
  gurps-io/        JSON-формат файла персонажа (версия 2), атомарная запись
  gurps-dice/      броски 3d6 и урона для приложения
  gurps-library/   каталог из SurrealDB, проекция в типы листа, расчёт через gurps-calc
  gurps-cli/       CLI `gurps` (calc / validate / roll), JSON-вывод
src-tauri/         приложение Tauri v2: состояние, история, команды для UI
ui/                интерфейс Vue 3; src/mock*.ts — встроенный демо-бэкенд для браузера
  scripts/         генерация тем из токенов, обёртка над Tauri CLI
  tests/           тесты Vitest
examples/          пример персонажа hero.json и тестовые персонажи (test-chars/)
docs/              планы этапов 3–4 и тестовые данные
scripts/           gen-engine-lines.py — эталонные строки пояснений расчёта для тестов UI
```

## Зависимость от mcp-сервер

Каталог (паки, переводы, константы правил `ref_*`) и расчёт живут в отдельном репозитории
`mcp-сервер` (схема БД — `Data/database`). Workspace подключает его крейты path-зависимостями:

```toml
gurps-db = { path = "../../ПРОЕКТЫ/mcp-сервер/crates/gurps-db" }
sheet-bridge = { path = "../../ПРОЕКТЫ/mcp-сервер/tools/sheet-bridge" }
```

Поэтому Rust-часть собирается только при наличии этого checkout'а по указанному
относительному пути. Решение и альтернативы (git-зависимость после публикации `mcp-сервер`,
`[patch]` для локальной разработки) описаны в [`docs/plan-stage3.md`](docs/plan-stage3.md).
UI в браузере (`npm run dev`) от этого не зависит — он работает на демо-бэкенде.

## Сборка и запуск

### Требования

- Rust stable (rustup), Node.js 20+ с npm.
- Для десктоп-приложения — системные зависимости Tauri v2.

**Arch Linux**
```sh
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg xdotool nodejs npm rustup
rustup default stable
```

**Debian/Ubuntu:** `libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.

**Windows 10/11:** WebView2 Runtime (уже есть в Windows 10/11), Microsoft C++ Build Tools
(workload «Desktop development with C++»), Rust через rustup (`rustup default stable-msvc`).

### Только интерфейс в браузере (без БД и без Rust)

```sh
cd ui && npm install && npm run dev      # демо-бэкенд из ui/src/mock*.ts
cd ui && npm run build && npm test       # проверка типов + сборка + тесты UI
```

### Приложение и CLI

Нужны checkout `mcp-сервер` (см. выше) и доступная SurrealDB с каталогом. Параметры
подключения читаются из окружения: `SURREALDB_HOST`, `SURREALDB_NAMESPACE`, `SURREALDB_NAME`,
`SURREALDB_USER`, `SURREALDB_PASSWORD`.

```sh
cd ui && npm install && npm run tauri dev   # десктоп-приложение через @tauri-apps/cli из devDependencies
cargo tauri dev                              # то же через cargo-CLI (cargo install tauri-cli --version "^2" --locked)

cargo run -p gurps-cli -- calc examples/hero.json
cargo run -p gurps-cli -- validate examples/hero.json
cargo run -p gurps-cli -- roll 12 --mod -2
```

`beforeDevCommand` в `tauri.conf.json` имеет явный `cwd: "../ui"`, поэтому `cargo tauri dev`
работает и из корня, и из `src-tauri/`. Относительные имена файлов сохраняются в
`Documents/GURPS Sheet`.

**Hyprland / Wayland:** если окно пустое или падает — `WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo tauri dev`.

### Тесты

```sh
cargo test                                   # все Rust-тесты
cargo clippy --all-targets -- -D warnings
cd ui && npm test                            # тесты UI (Vitest)
```

Тесты, которым нужна БД (`gurps-library/tests`, состояние приложения, CLI), поднимают временную
SurrealDB в памяти через `testkit` из `gurps-db` (`surreal start memory` + `surrealkit sync/seed`
по сидам `mcp-сервер`), поэтому нужны бинарники `surreal` и `surrealkit` в `PATH`. Без них такие
тесты пропускаются.

## Статус и планы

Версия 0.1.0, в активной разработке. Основная ветка разработки — `feat/db-storage`
(перенос хранения каталога на SurrealDB и расчёта на `gurps-calc`, этапы 3–4).

Осталось по [`docs/plan-stage4.md`](docs/plan-stage4.md):

- строки пояснений расчёта (навыки, урон, атаки, защиты, стоимость черт) в `gurps-calc` —
  сейчас там только структурные данные;
- у своих (inline) определений из UI пока есть только стоимость, без пререквизитов и эффектов;
- безоружные атаки: сиды punch/kick и связь «приём → атака» (тест `warrior_unarmed_pending`
  помечен `#[ignore]`);
- после публикации `mcp-сервер` — заменить path-зависимости на git-зависимости.

## Как это сделано

- **Поэтапные планы** в [`docs/`](docs/): `plan-stage3.md` (решение о слое хранения с
  плюсами, минусами и альтернативами), `plan-stage4.md` (план переноса расчёта со статусом
  «сделано/осталось» и ссылками на коммиты); коммиты подписаны по этапам.
- **Golden-тесты на эталонных персонажах:** `examples/test-chars/` — воин, маг, разбойник;
  ожидания взяты из данных Мастера ([`docs/test-data-stage2-3.md`](docs/test-data-stage2-3.md)),
  `gurps-library/tests/golden_db.rs` проверяет расчёт по БД против этих ожиданий.
- **Тесты на всех уровнях:** unit-тесты крейтов, тесты состояния приложения на временной БД,
  тесты CLI и 10 наборов тестов UI (Vitest), включая TS-зеркало расчёта и разбор Markdown.
- **Генерация вместо ручной синхронизации:** темы — из дизайн-токенов (`ui/scripts/gen-themes.mjs`),
  эталонные строки пояснений для тестов UI — `scripts/gen-engine-lines.py`.

CI пока не настроен.

## English

**What it is.** A desktop GURPS 4e character sheet editor. Every derived value is
`base + contributions`, so the UI shows *where a number came from*. Features: attributes,
traits with modifiers, skills and techniques, spells, equipment and encumbrance, weapons and
active defences, point pool ledger with limits, custom content stored inline, Markdown
explanations and notes (sanitised), 200-step undo/redo, a play mode (HP/FP trackers, 3d6 and
damage rolls), three themes (one for print/PDF) generated from design tokens, catalog
translations, and a JSON CLI.

**Build.** `cd ui && npm i && npm run dev` runs the UI in a browser on a built-in demo backend.
The Rust workspace depends on crates from the sibling `mcp-сервер` repository via path
dependencies and reads the catalog from SurrealDB (`SURREALDB_*` env vars); with those in place,
use `cd ui && npm run tauri dev` or `cargo tauri dev`, and `cargo test`.

**Character file.** JSON `version: 2`: entries reference the catalog as `{pack, sid}` or carry an
inline definition (`{inline: {...}}`); version 1 files are rejected.

## Лицензия / License

MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`). Шрифты в `ui/public/fonts` — SIL Open Font
License 1.1.
