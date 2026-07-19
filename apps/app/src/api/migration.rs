use crate::api::Result;
use std::path::PathBuf;
use tauri::Runtime;
use tauri::plugin::TauriPlugin;
use theseus::migration::{self, MigrationCandidate};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("migration")
        .invoke_handler(tauri::generate_handler![
            find_modrinth_install_candidate,
            migrate_from_modrinth,
        ])
        .build()
}

/// Reports whether a Modrinth App data directory exists that Meverinth could
/// migrate from. Returns `null` if Meverinth has already been initialized or
/// there is no Modrinth App install to copy.
#[tauri::command]
pub async fn find_modrinth_install_candidate<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<MigrationCandidate>> {
    let identifier = app.config().identifier.clone();
    Ok(migration::find_modrinth_install_candidate(&identifier).await?)
}

/// Copies the Modrinth App data directory at `sourcePath` into Meverinth's
/// data directory. Must be called before `initialize_state`, since this
/// command writes into the very directory the state init will read from.
#[tauri::command]
pub async fn migrate_from_modrinth<R: Runtime>(
    app: tauri::AppHandle<R>,
    source_path: PathBuf,
) -> Result<()> {
    let identifier = app.config().identifier.clone();
    migration::migrate_from_path(&identifier, source_path).await?;
    Ok(())
}
