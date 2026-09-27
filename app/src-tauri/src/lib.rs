use std::sync::{Arc, RwLock};

use engine::auth::{self, LoginPage};
use engine::model::{Action, CleanupItem, Group, Label, Proposal, Report, Status, Summary};
use engine::settings::Catalog;
use engine::{Error, ErrorCode, Gmail, Result, Settings, Storage, cleanup, organize, rules};
use tauri::{AppHandle, Emitter, Manager};

struct Session {
    storage: Storage,
    gmail: RwLock<Option<Arc<Gmail>>>,
}

impl Session {
    fn status(&self) -> Status {
        Status {
            has_credentials: self.storage.credentials().exists(),
            account: self.gmail.read().unwrap().as_ref().map(|g| g.account.clone()),
        }
    }
}

async fn background<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| Error::with(ErrorCode::Io, e))?
}

async fn with_gmail<T: Send + 'static>(
    app: &AppHandle,
    f: impl FnOnce(&Gmail, &Settings, Report) -> Result<T> + Send + 'static,
) -> Result<T> {
    let session = app.state::<Session>();
    let gmail = session.gmail.read().unwrap().clone().ok_or(ErrorCode::NoSession)?;
    let settings = Settings::load(&session.storage);
    let app = app.clone();
    background(move || {
        f(&gmail, &settings, &|p| {
            let _ = app.emit("progress", p);
        })
    })
    .await
}

#[tauri::command]
async fn status(app: AppHandle) -> Result<Status> {
    let session = app.state::<Session>();
    if session.gmail.read().unwrap().is_none() && session.storage.credentials().exists() {
        let storage = session.storage.clone();
        let gmail = background(move || Gmail::restore(&storage)).await?;
        *session.gmail.write().unwrap() = gmail.map(Arc::new);
    }
    Ok(session.status())
}

#[tauri::command]
fn save_credentials(app: AppHandle, json: String) -> Result<Status> {
    let session = app.state::<Session>();
    auth::save_credentials(&session.storage, &json)?;
    Ok(session.status())
}

#[tauri::command]
async fn connect(app: AppHandle, page: LoginPage) -> Result<Status> {
    let session = app.state::<Session>();
    let storage = session.storage.clone();
    let gmail = background(move || Gmail::login(&storage, &page)).await?;
    *session.gmail.write().unwrap() = Some(Arc::new(gmail));
    Ok(session.status())
}

#[tauri::command]
fn disconnect(app: AppHandle) -> Result<Status> {
    let session = app.state::<Session>();
    session.storage.clear_token()?;
    *session.gmail.write().unwrap() = None;
    Ok(session.status())
}

#[tauri::command]
async fn summary(app: AppHandle) -> Result<Summary> {
    with_gmail(&app, |g, _, _| organize::summary(g)).await
}

#[tauri::command]
async fn analyze(app: AppHandle, reorganize: bool, locale: String) -> Result<Proposal> {
    with_gmail(&app, move |g, s, report| organize::analyze(g, s, reorganize, &locale, report)).await
}

#[tauri::command]
async fn apply(app: AppHandle, groups: Vec<Group>) -> Result<()> {
    with_gmail(&app, move |g, _, report| rules::apply(g, &groups, report)).await
}

#[tauri::command]
async fn labels(app: AppHandle) -> Result<Vec<Label>> {
    with_gmail(&app, |g, s, _| rules::labels(g, s)).await
}

#[tauri::command]
async fn change_action(app: AppHandle, label_id: String, action: Action) -> Result<()> {
    with_gmail(&app, move |g, _, _| rules::change_action(g, &label_id, action)).await
}

#[tauri::command]
async fn rename_label(app: AppHandle, label_id: String, name: String) -> Result<()> {
    let new = name.clone();
    let old = with_gmail(&app, move |g, _, _| rules::rename_label(g, &label_id, &name)).await?;
    let session = app.state::<Session>();
    let mut settings = Settings::load(&session.storage);
    if settings.rename(&old, new.trim()) {
        settings.save(&session.storage)?;
    }
    Ok(())
}

#[tauri::command]
async fn delete_label(app: AppHandle, label_id: String) -> Result<()> {
    with_gmail(&app, move |g, _, _| rules::delete_label(g, &label_id)).await
}

#[tauri::command]
async fn remove_rule(app: AppHandle, filter_id: String) -> Result<()> {
    with_gmail(&app, move |g, _, _| rules::remove_rule(g, &filter_id)).await
}

#[tauri::command]
async fn cleanup_items(app: AppHandle, days: u32) -> Result<Vec<CleanupItem>> {
    with_gmail(&app, move |g, s, _| cleanup::items(g, s, days)).await
}

#[tauri::command]
async fn clean(app: AppHandle, ids: Vec<String>, days: u32) -> Result<usize> {
    with_gmail(&app, move |g, s, report| cleanup::clean(g, s, &ids, days, report)).await
}

#[tauri::command]
fn settings(app: AppHandle) -> Settings {
    Settings::load(&app.state::<Session>().storage)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<()> {
    settings.save(&app.state::<Session>().storage)
}

#[tauri::command]
fn protect(app: AppHandle, name: String, protected: bool) -> Result<()> {
    let session = app.state::<Session>();
    let mut settings = Settings::load(&session.storage);
    settings.protect(&name, protected);
    settings.save(&session.storage)
}

#[tauri::command]
fn catalog() -> Catalog {
    Catalog::get()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let storage = Storage::new(app.path().app_data_dir()?, app.config().identifier.clone());
            app.manage(Session {
                storage,
                gmail: RwLock::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            status,
            save_credentials,
            connect,
            disconnect,
            summary,
            analyze,
            apply,
            labels,
            change_action,
            rename_label,
            delete_label,
            remove_rule,
            cleanup_items,
            clean,
            settings,
            save_settings,
            protect,
            catalog
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Ordo");
}
