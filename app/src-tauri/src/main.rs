// Sans ça, Windows ouvre une console à côté de l'app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ia;
mod pannes;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use moteur::Rapport;
use moteur::installation::{self, Options};
use moteur::lab::{self, EtatLab, Lab};
use moteur::terminal::Terminal;
use moteur::veille::{self, Veille};
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

/// Passe à vrai quand l'utilisateur arrête l'installation.
struct Arret(Arc<AtomicBool>);

#[derive(Default)]
struct Terminaux {
    suivant: AtomicU32,
    ouverts: Mutex<HashMap<u32, Terminal>>,
}

#[derive(Default)]
struct Veilleur(Mutex<Option<Veille>>);

/// Pour ne pas arrêter le lab deux fois quand l'app se ferme.
#[derive(Default)]
struct Fermeture(AtomicBool);

fn dossier_wsl(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("wsl"))
}

/// Les commandes du moteur attendent Incus ou WSL : elles tournent à côté,
/// jamais dans le fil de la fenêtre.
pub(crate) async fn a_cote<T: Send + 'static>(
    travail: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(travail)
        .await
        .map_err(|e| e.to_string())?
}

/// Le seul lab de l'étape 3.
pub(crate) fn le_lab() -> Lab {
    lab::premier_lab()
}

#[tauri::command]
async fn verifier_pc() -> Result<Rapport, String> {
    a_cote(|| Ok(moteur::verifier())).await
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
    a_cote(move || {
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
}

#[tauri::command]
fn arreter_installation(arret: State<'_, Arret>) {
    arret.0.store(true, Ordering::Relaxed);
}

#[tauri::command]
async fn retirer_linux(app: AppHandle) -> Result<Rapport, String> {
    let dossier = dossier_wsl(&app)?;
    a_cote(move || {
        installation::retirer(&dossier)?;
        Ok(moteur::verifier())
    })
    .await
}

#[tauri::command]
async fn installer_wsl() -> Result<(), String> {
    a_cote(installation::installer_wsl).await
}

#[tauri::command]
async fn etat_lab() -> Result<EtatLab, String> {
    a_cote(|| lab::etat(&le_lab())).await
}

#[tauri::command]
async fn lancer_lab(app: AppHandle) -> Result<EtatLab, String> {
    a_cote(move || {
        lab::lancer(&le_lab(), &mut |avance| {
            let _ = app.emit("lab-avance", avance);
        })
    })
    .await
}

#[tauri::command]
async fn arreter_lab() -> Result<EtatLab, String> {
    a_cote(|| lab::arreter(&le_lab())).await
}

#[tauri::command]
async fn demarrer_machine(app: AppHandle, nom: String) -> Result<EtatLab, String> {
    a_cote(move || {
        lab::demarrer_machine(&le_lab(), &nom, &mut |avance| {
            let _ = app.emit("lab-avance", avance);
        })
    })
    .await
}

#[tauri::command]
async fn arreter_machine(nom: String) -> Result<EtatLab, String> {
    a_cote(move || lab::arreter_machine(&le_lab(), &nom)).await
}

#[tauri::command]
async fn remettre_a_zero(app: AppHandle) -> Result<EtatLab, String> {
    a_cote(move || {
        lab::remettre_a_zero(&le_lab(), &mut |avance| {
            let _ = app.emit("lab-avance", avance);
        })
    })
    .await
}

/// Ce que le terminal affiche part en octets par `flux`. Sa fin part par le
/// même canal, après les derniers octets : le mot « fin ».
#[tauri::command]
fn ouvrir_terminal(
    app: AppHandle,
    terminaux: State<'_, Terminaux>,
    machine: String,
    colonnes: u16,
    lignes: u16,
    flux: Channel<InvokeResponseBody>,
) -> Result<u32, String> {
    let id = terminaux.suivant.fetch_add(1, Ordering::Relaxed);
    let fini = Arc::new(AtomicBool::new(false));
    let (fini_ici, flux_fin) = (fini.clone(), flux.clone());
    let terminal = moteur::terminal::ouvrir(
        &le_lab(),
        &machine,
        colonnes,
        lignes,
        move |octets| {
            let _ = flux.send(InvokeResponseBody::Raw(octets));
        },
        move || {
            fini_ici.store(true, Ordering::Relaxed);
            let _ = flux_fin.send(InvokeResponseBody::Json("\"fin\"".into()));
            if let Ok(mut ouverts) = app.state::<Terminaux>().ouverts.lock() {
                ouverts.remove(&id);
            }
        },
    )?;
    let mut ouverts = terminaux.ouverts.lock().map_err(|e| e.to_string())?;
    // Un shell parti avant d'être rangé ne doit pas rester dans la liste.
    if !fini.load(Ordering::Relaxed) {
        ouverts.insert(id, terminal);
    }
    Ok(id)
}

#[tauri::command]
fn ecrire_terminal(
    terminaux: State<'_, Terminaux>,
    id: u32,
    donnees: String,
) -> Result<(), String> {
    let ouverts = terminaux.ouverts.lock().map_err(|e| e.to_string())?;
    match ouverts.get(&id) {
        Some(terminal) => terminal.ecrire(donnees.as_bytes()),
        None => Err("Ce terminal est fermé.".into()),
    }
}

#[tauri::command]
fn redimensionner_terminal(
    terminaux: State<'_, Terminaux>,
    id: u32,
    colonnes: u16,
    lignes: u16,
) -> Result<(), String> {
    let ouverts = terminaux.ouverts.lock().map_err(|e| e.to_string())?;
    match ouverts.get(&id) {
        Some(terminal) => terminal.redimensionner(colonnes, lignes),
        None => Ok(()),
    }
}

#[tauri::command]
fn fermer_terminal(terminaux: State<'_, Terminaux>, id: u32) {
    let terminal = terminaux
        .ouverts
        .lock()
        .ok()
        .and_then(|mut o| o.remove(&id));
    drop(terminal);
}

/// Écoute Incus pour suivre les machines, une fois Incus prêt.
#[tauri::command]
fn veiller(app: AppHandle, veilleur: State<'_, Veilleur>) -> Result<(), String> {
    let mut place = veilleur.0.lock().map_err(|e| e.to_string())?;
    if place.is_none() {
        let projet = lab::projet(&le_lab());
        *place = Some(veille::veiller(move |changement| {
            if changement.projet == projet {
                let _ = app.emit("lab", changement);
            }
        }));
    }
    Ok(())
}

/// L'app se ferme : les machines du lab s'arrêtent avec elle, sans attendre
/// plus de dix secondes un Incus qui ne répondrait plus.
fn fermer(app: &AppHandle) {
    let (fait, attente) = mpsc::channel();
    std::thread::spawn(move || {
        lab::arreter_vite(&le_lab());
        let _ = fait.send(());
    });
    let _ = attente.recv_timeout(Duration::from_secs(10));
    if let Ok(mut place) = app.state::<Veilleur>().0.lock() {
        place.take();
    }
    if let Ok(mut ouverts) = app.state::<Terminaux>().ouverts.lock() {
        ouverts.clear();
    }
    app.exit(0);
}

fn main() {
    // Lancée par Codex comme serveur MCP, l'app ne fait que relayer les
    // outils à l'app ouverte (moteur/src/ia/mcp.rs).
    if let Ok(valeur) = std::env::var(moteur::ia::mcp::VARIABLE) {
        if let Err(erreur) = moteur::ia::mcp::servir(&valeur) {
            eprintln!("{erreur}");
        }
        return;
    }
    tauri::Builder::default()
        .manage(Arret(Arc::new(AtomicBool::new(false))))
        .manage(Terminaux::default())
        .manage(Veilleur::default())
        .manage(Fermeture::default())
        .manage(ia::Ia::default())
        .manage(pannes::Pannes::default())
        .on_window_event(|fenetre, evenement| {
            if let WindowEvent::CloseRequested { api, .. } = evenement {
                api.prevent_close();
                let app = fenetre.app_handle().clone();
                if !app.state::<Fermeture>().0.swap(true, Ordering::Relaxed) {
                    std::thread::spawn(move || fermer(&app));
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            verifier_pc,
            installer,
            arreter_installation,
            retirer_linux,
            installer_wsl,
            etat_lab,
            lancer_lab,
            arreter_lab,
            demarrer_machine,
            arreter_machine,
            remettre_a_zero,
            ouvrir_terminal,
            ecrire_terminal,
            redimensionner_terminal,
            fermer_terminal,
            veiller,
            ia::etat_ia,
            ia::brancher_ia,
            ia::essayer_ia,
            ia::modeles_ia,
            ia::regler_ia,
            ia::parler_ia,
            ia::repondre_accord,
            ia::arreter_ia,
            ia::oublier_conversation,
            ia::inventer_panne,
            pannes::catalogue_pannes,
            pannes::poser_panne,
            pannes::panne_en_cours,
            pannes::indice_panne,
            pannes::verifier_panne,
            pannes::abandonner_panne,
            pannes::fermer_panne
        ])
        .run(tauri::generate_context!())
        .expect("l'app n'a pas pu démarrer");
}
