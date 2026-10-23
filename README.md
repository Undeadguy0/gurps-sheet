# GURPS Sheet

Open-source GURPS 4e character editor: Rust core + Tauri v2 + Vue 3.
Редактор листа персонажа GURPS 4e с открытым исходным кодом: ядро на Rust, оболочка Tauri v2, интерфейс Vue 3.

> GURPS is a trademark of Steve Jackson Games. This project is unofficial and ships
> no rules text — plugins contain only names, point costs and page references.
> GURPS — торговая марка Steve Jackson Games. Проект неофициальный; плагины содержат
> только названия, стоимость и ссылки на страницы, без текста правил.

## Русский

**Состав:** `crates/gurps-core` (модель), `gurps-rules` (граф значений с объяснениями, очки,
броски 3d6), `gurps-plugins` (плагины-данные, порядок зависимостей), `gurps-io` (JSON-формат
с версией), `gurps-cli` (CLI с JSON-выводом), `src-tauri` (приложение), `ui` (Vue 3 + TanStack Table).

Темы: «Инструмент» (по умолчанию), «Стол мастера», «Бумажный лист» (и для печати/PDF).
Генерируются из `ui/src/styles/tokens.json` в `ui/src/styles/themes.css` (`npm run gen:themes`).

## English

Each derived value is `base + contributions`, so the UI can show *where a number came from*
(e.g. ST 13 = base 10 + 2 purchased + 1 from a trait). Themes are CSS custom properties
generated from design tokens; switch via the header selector (stored in `localStorage`).

## Prerequisites / Требования

**Windows 10/11**
- WebView2 Runtime — уже установлен в Windows 10/11 / preinstalled on Windows 10/11 (otherwise: Evergreen Bootstrapper from Microsoft).
- Microsoft C++ Build Tools (Visual Studio Build Tools, workload «Desktop development with C++» / «Разработка классических приложений на C++»).
- Rust via rustup (`rustup default stable-msvc`), Node.js 20+ with npm.

**Arch Linux**
```sh
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg xdotool nodejs npm rustup
rustup default stable
```

Tauri CLI: либо `cd ui && npm install && npm run tauri dev` (CLI из devDependencies), либо `cargo install tauri-cli --version "^2" --locked` и `cargo tauri dev`. Обе команды работают из корня, из `src-tauri/` и (npm) из `ui/` — `beforeDevCommand` имеет явный `cwd: "../ui"`.
Either use `npm run tauri dev` from `ui/` (CLI from devDependencies) or install the cargo CLI and run `cargo tauri dev` from the repo root or `src-tauri/`; the frontend command always runs in `ui/` thanks to an explicit `cwd`.

## Run / Запуск

Requirements / Требуется: Rust stable, Node.js LTS; for the desktop app the Tauri Linux deps
(`libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`) and
`cargo install tauri-cli --version "^2"`.

```sh
cargo test                                    # all Rust tests
cargo clippy --all-targets -- -D warnings
cargo run -p gurps-cli -- calc examples/hero.json
cargo run -p gurps-cli -- roll 12 --mod -2
cargo run -p gurps-cli -- validate examples/hero.json
# Nushell: cargo run -q -p gurps-cli -- calc examples/hero.json | from json | get points

cd ui && npm i && npm run dev                 # browser preview (built-in demo backend)
cd ui && npm run build && npm test            # type check + build + UI tests

cargo tauri dev                               # desktop app: repo root or src-tauri/ (needs `cargo install tauri-cli --version "^2" --locked`)
cd ui && npm run tauri dev                    # same, via bundled @tauri-apps/cli (no cargo install needed)
```

**Hyprland / Wayland:** if the window is blank or crashes, run
`WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo tauri dev`.
Если окно пустое или падает под Hyprland — запускайте с `WEBKIT_DISABLE_DMABUF_RENDERER=1`.

Plugins are loaded from `$GURPS_PLUGINS`, `./plugins` or `../plugins`.

## License

Dual-licensed under MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`).
Bundled fonts (`ui/public/fonts`) are under the SIL Open Font License 1.1.
