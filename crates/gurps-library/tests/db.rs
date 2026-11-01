//! Каталог из временной SurrealDB (surreal memory + surrealkit sync/seed из mcp-сервер/Data/database).
use gurps_db::testkit;
use gurps_library::{Library, LibrarySource};

#[tokio::test(flavor = "multi_thread")]
async fn library_from_seeded_db() {
    let Some((srv, db)) = testkit::start().await else { return };
    let lib = Library::fetch(&db).await.unwrap();
    assert!(lib.packs.iter().any(|p| p.starts_with("basic")), "{:?}", lib.packs);
    assert!(!lib.skills.is_empty(), "навыки из сидов");
    assert!(!lib.traits.is_empty(), "трейты из сидов");
    for (pack, s) in &lib.skills {
        assert!(!pack.is_empty() && !s.sid.is_empty());
    }
    assert!(lib.warnings.is_empty(), "{:?}", lib.warnings);
    assert!(!lib.natural.is_empty(), "естественные атаки");
    assert!(lib.items.iter().any(|(_, i)| !i.attack_modes.is_empty()));
    // переводы из translation (ru) доступны по ключу kind-id
    assert!(lib.translations.languages().iter().any(|l| l == "ru"));

    // синхронный путь (как в приложении): свой поток + runtime
    let cfg = srv.cfg.clone();
    let lib2 = tokio::task::spawn_blocking(move || Library::load(&LibrarySource::Db(cfg)))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(lib2.skills.len(), lib.skills.len());
}

#[test]
fn unreachable_db_is_an_error() {
    let cfg = gurps_library::DbConfig {
        host: "ws://127.0.0.1:1".into(),
        namespace: "x".into(),
        database: "x".into(),
        user: "root".into(),
        password: "root".into(),
    };
    assert!(Library::load(&LibrarySource::Db(cfg)).is_err());
}