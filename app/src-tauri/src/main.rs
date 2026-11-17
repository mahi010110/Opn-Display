// Sans ça, Windows ouvre une console à côté de l'app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use moteur::Rapport;
use moteur::installation::{self, Options};
use tauri::{AppHandle, Emitter, Manager, State};

/// Passe à vrai quand l'utilisateur arrête l'installation.
struct Arret(Arc<AtomicBool>);

fn dossier_wsl(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("wsl"))
}

#[tauri::command]
async fn verifier_pc() -> Result<Rapport, String> {
    tauri::async_runtime::spawn_blocking(moteur::verifier)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn installer(
    app: AppHandle,
    arret: State<'_, Arret>,
    memoire_go: Option<u32>,
) -> Result<Rapport, String> {
    let dossier = dossier_wsl(&app)?;
    let arret = arret.0.clone();
    arret.store(false, Ordering::Relaxed);
    tauri::async_runtime::spawn_blocking(move || {
        let options = Options {
            dossier,
            memoire_go,
        };
        installation::installer(
            &options,
            &mut |etape| {
                let _ = app.emit("installation", etape);
            },
            &arret,
        )?;
        Ok(moteur::verifier())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn arreter_installation(arret: State<'_, Arret>) {
    arret.0.store(true, Ordering::Relaxed);
}

#[tauri::command]
async fn retirer_linux(app: AppHandle) -> Result<Rapport, String> {
    let dossier = dossier_wsl(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        installation::retirer(&dossier)?;
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
        .manage(Arret(Arc::new(AtomicBool::new(false))))
        .invoke_handler(tauri::generate_handler![
            verifier_pc,
            installer,
            arreter_installation,
            retirer_linux,
            installer_wsl
        ])
        .run(tauri::generate_context!())
        .expect("l'app n'a pas pu démarrer");
}
