use std::path::PathBuf;

use engine::{Gmail, Settings, Storage, cleanup, organize, rules};

const SERVICE: &str = "com.lojan.ordo";

fn storage() -> Storage {
    let dir = std::env::var("ORDO_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("APPDATA").expect("APPDATA")).join(SERVICE));
    Storage::new(dir, SERVICE)
}

#[test]
#[ignore]
fn reads_the_real_mailbox_without_changing_it() {
    let storage = storage();
    let gmail = Gmail::restore(&storage).expect("restore").expect("a saved session");
    let settings = Settings::load(&storage);

    assert!(gmail.account.contains('@'));
    let summary = organize::summary(&gmail).expect("summary");
    let labels = rules::labels(&gmail, &settings).expect("labels");
    assert_eq!(summary.labels, labels.len());
    assert!(labels.iter().all(|l| !l.id.is_empty() && !l.name.is_empty()));
    let items = cleanup::items(&gmail, &settings, settings.cleanup_days).expect("cleanup items");
    assert!(items.iter().filter(|i| i.protected).all(|i| i.total == 0));
    assert!(items.iter().any(|i| i.special));
}
