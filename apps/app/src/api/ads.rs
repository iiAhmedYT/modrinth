// Ads have been removed from this privacy-focused build. The plugin and
// commands are kept as no-ops so any stale calls from the frontend or
// previously persisted settings do not error.

use tauri::Runtime;
use tauri::plugin::TauriPlugin;
use theseus::settings;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("ads")
        .invoke_handler(tauri::generate_handler![
            init_ads_window,
            hide_ads_window,
            show_ads_window,
            show_ads_consent_overlay,
            hide_ads_consent_overlay,
            record_ads_click,
            open_link,
            get_ads_personalization,
        ])
        .build()
}

#[tauri::command]
pub async fn init_ads_window(dpr: f32, override_shown: bool) -> crate::api::Result<()> {
    let _ = (dpr, override_shown);
    Ok(())
}

#[tauri::command]
pub async fn show_ads_window(dpr: f32) -> crate::api::Result<()> {
    let _ = dpr;
    Ok(())
}

#[tauri::command]
pub async fn hide_ads_window(reset: Option<bool>) -> crate::api::Result<()> {
    let _ = reset;
    Ok(())
}

#[tauri::command]
pub async fn show_ads_consent_overlay() -> crate::api::Result<()> {
    Ok(())
}

#[tauri::command]
pub async fn hide_ads_consent_overlay(dpr: Option<f32>) -> crate::api::Result<()> {
    let _ = dpr;
    Ok(())
}

#[tauri::command]
pub async fn record_ads_click() -> crate::api::Result<()> {
    Ok(())
}

#[tauri::command]
pub async fn open_link(path: String, origin: String) -> crate::api::Result<()> {
    let _ = (path, origin);
    Ok(())
}

#[tauri::command]
pub async fn get_ads_personalization() -> crate::api::Result<bool> {
    // Personalized ads are unconditionally off in this build, but we still
    // surface the user's stored preference so the settings UI stays consistent.
    let res = settings::get().await?;
    Ok(res.personalized_ads)
}
