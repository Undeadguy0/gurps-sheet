# Plugin data format / Формат данных плагинов

```
plugins/<id>/
  plugin.toml               # id, name, version, depends = [...]
  data/*.toml               # [[traits]], [[skills]], [[spells]], [[equipment]], [[weapons]], [[modifiers]]
  locales/<lang>/*.ftl      # optional Fluent translations (any plugin, or a language pack)
```

## Entries / Записи

```toml
[[traits]]
id = "combat-reflexes"      # stable id — also the translation key
name = "Combat Reflexes"    # original name
category = "advantage"
cost = 15
page = "B43"                # page refs are unverified
description = '''
Fast reactions in a fight:
- rarely caught off guard.
'''
```

`description` (пояснение) is optional Markdown. Accepted forms: a plain string (recommended,
the original-language text) or, for backward compatibility, a locale table
`description = { ru = "...", en = "..." }` / `[traits.description] ru = '''...''' en = '''...'''`
(fallback: exact locale → language part → `en` → `ru` → any). Skills, spells, equipment,
weapons and modifiers currently carry `id`, `name`, `page`, `description` only.

`description` — необязательное пояснение в Markdown: строка (рекомендуется) или таблица
`{ ru, en }` (для совместимости). **Только собственные формулировки — не копируйте текст книг SJG.**

### Equipment / Снаряжение

```toml
[[equipment]]
id = "backpack-frame"
name = "Backpack, Frame"
weight = 10.0        # lb per unit
cost = 100.0         # $ per unit
container = true     # optional: can hold other items
stats = "DB 2"       # optional free text (shield DB, armour DR...), not used in maths yet
page = "B288"        # unverified
description = '''Own-words Markdown explanation.'''
```

Fluent keys: `equipment-<id>`. Encumbrance: carried weight vs Basic Lift ×1/2/3/6/10 →
levels 0–4; Move ×1/0.8/0.6/0.4/0.2 rounded down; Dodge −level.

## Translations (Fluent) / Переводы

Format: [Fluent](https://projectfluent.org) `.ftl`, parsed with the `fluent-syntax` crate.
Message id = `<kind>-<id>`, kind ∈ `trait skill spell equipment weapon modifier`;
value = translated name, `.description` = translated Markdown explanation:

```ftl
trait-combat-reflexes = Боевые рефлексы
    .description =
        Быстрая реакция в бою:
        - реже теряется от неожиданности.
```

- Multi-line text: indent continuation lines. A continuation line must not start with
  `*`, `[` or `.` — write lists with `-`, or escape: `{ "*" }`. Braces: `{ "{" }`.
- Later plugins (dependency order) override earlier ones per field, so a language pack
  may translate only names. A language pack is a plugin with only `plugin.toml` + `locales/`
  (see `lang-ru`).
- Lookup: exact locale, then language part (`ru-RU` → `ru`); otherwise the original is shown.

Plurals: the loader reads only plain text patterns (variables and `{ $n -> [one] … }`
selectors are not evaluated), so data strings must not embed counts. Numbers shown in the UI are
formatted by `ui/src/plural.ts` (`Intl.PluralRules('ru')`: 1 очко, 2 очка, 5 очков, 21 очко).

Priority of the explanation shown: user text in the character file → Fluent translation
for the UI language → plugin `description`.
Приоритет: своё пояснение в файле персонажа → перевод Fluent → `description` плагина.
