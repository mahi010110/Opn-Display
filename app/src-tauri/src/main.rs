// Sans ça, Windows ouvre une console à côté de l'app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use moteur::installation::{self, Options};
use moteur::Rapport;
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
async fn verifier_pc() -> Result<Rapport, String> {
    tauri::async_runtime::spawn_blocking(moteur::verifier)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn installer(app: AppHandle, memoire_go: Option<u32>) -> Result<Rapport, String> {
    let dossier = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("wsl");
    tauri::async_runtime::spawn_blocking(move || {
        let options = Options { dossier, memoire_go };
        installation::installer(&options, &mut |etape| {
            let _ = app.emit("installation", etape);
        })?;
        Ok(moteur::verifier())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn installer_wsl() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(installation::installer_wsl)
        .await
        .map_err(|e| e.to_string())?
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![verifier_pc, installer, installer_wsl])
        .run(tauri::generate_context!())
        .expect("l'app n'a pas pu démarrer");
}
