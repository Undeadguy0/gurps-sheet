//! ВРЕМЕННО (этап 4 удалит): старые TOML-плагины как источник библиотеки — только тесты.
use crate::{Library, LocEntry, Translations};
use std::path::Path;

pub(crate) fn load(dir: &Path) -> Result<Library, String> {
    let plugins = gurps_plugins::load_all(dir).map_err(|e| e.to_string())?;
    let cat = gurps_plugins::translations(&plugins);
    let translations = Translations {
        langs: cat
            .langs
            .into_iter()
            .map(|(l, m)| {
                let m = m
                    .into_iter()
                    .map(|(k, e)| {
                        (
                            k,
                            LocEntry {
                                name: e.name,
                                description: e.description,
                            },
                        )
                    })
                    .collect();
                (l, m)
            })
            .collect(),
    };
    Ok(Library {
        packs: plugins
            .iter()
            .map(|p| format!("{} {}", p.manifest.id, p.manifest.version))
            .collect(),
        traits: gurps_plugins::trait_library(&plugins),
        skills: gurps_plugins::skill_library(&plugins),
        techniques: gurps_plugins::technique_library(&plugins),
        equipment: gurps_plugins::equipment_library(&plugins),
        weapons: gurps_plugins::weapon_library(&plugins),
        spells: gurps_plugins::spell_library(&plugins),
        modifiers: gurps_plugins::modifier_library(&plugins),
        translations,
        warnings: vec![],
    })
}