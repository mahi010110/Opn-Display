//! L'IA côté app : ses réglages, sa clé dans le trousseau du système, la
//! conversation, et l'accord de l'utilisateur.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use moteur::ia::{self, Branchement, Conversation, Demande, Execution, Liberte, Niveau};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{a_cote, le_lab};

const SERVICE: &str = "opn-display";
const OLLAMA: &str = "http://127.0.0.1:11434/v1";

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Reglages {
    #[serde(default)]
    pub branchement: Option<Branchement>,
    #[serde(default)]
    pub niveau: Niveau,
    #[serde(default)]
    pub dans_le_lab: Liberte,
    /// Le dernier essai réussi, en secondes depuis 1970.
    #[serde(default)]
    pub essai: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtatIa {
    reglages: Reglages,
    cle_rangee: bool,
    codex_present: bool,
    /// Les modèles d'Ollama, ou rien s'il ne répond pas.
    ollama: Option<Vec<String>>,
}

#[derive(Default)]
pub struct Ia {
    conversation: Mutex<Option<Conversation>>,
    accords: Arc<Mutex<HashMap<String, Sender<bool>>>>,
    arret: Arc<AtomicBool>,
    occupee: AtomicBool,
}

fn fichier(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("ia.json"))
}

fn lire_reglages(app: &AppHandle) -> Reglages {
    fichier(app)
        .ok()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|texte| serde_json::from_str(&texte).ok())
        .unwrap_or_default()
}

fn ecrire_reglages(app: &AppHandle, reglages: &Reglages) -> Result<(), String> {
    let fichier = fichier(app)?;
    if let Some(dossier) = fichier.parent() {
        std::fs::create_dir_all(dossier).map_err(|e| e.to_string())?;
    }
    let texte = serde_json::to_string_pretty(reglages).map_err(|e| e.to_string())?;
    std::fs::write(fichier, texte).map_err(|e| format!("Les réglages ne s'écrivent pas : {e}"))
}

fn cle_rangee(branchement: &Branchement) -> Option<String> {
    let nom = branchement.cle()?;
    keyring::Entry::new(SERVICE, &nom).ok()?.get_password().ok()
}

fn ranger_cle(branchement: &Branchement, cle: &str) -> Result<(), String> {
    let nom = branchement.cle().ok_or("Ce branchement n'a pas de clé.")?;
    keyring::Entry::new(SERVICE, &nom)
        .and_then(|entree| entree.set_password(cle))
        .map_err(|e| format!("Le trousseau du système refuse la clé : {e}"))
}

fn etat(app: &AppHandle) -> EtatIa {
    let reglages = lire_reglages(app);
    let cle = reglages
        .branchement
        .as_ref()
        .is_some_and(|b| cle_rangee(b).is_some());
    let ollama = ia::modeles(
        &Branchement::Ollama {
            adresse: OLLAMA.into(),
            modele: String::new(),
        },
        None,
    )
    .ok();
    EtatIa {
        reglages,
        cle_rangee: cle,
        codex_present: ia::codex_present(),
        ollama,
    }
}

#[tauri::command]
pub async fn etat_ia(app: AppHandle) -> Result<EtatIa, String> {
    a_cote(move || Ok(etat(&app))).await
}

/// Choisit le branchement, et range sa clé si elle est donnée.
#[tauri::command]
pub async fn brancher_ia(
    app: AppHandle,
    branchement: Branchement,
    cle: Option<String>,
) -> Result<EtatIa, String> {
    a_cote(move || {
        if let Some(cle) = cle.filter(|c| !c.trim().is_empty()) {
            ranger_cle(&branchement, cle.trim())?;
        }
        let mut reglages = lire_reglages(&app);
        if reglages.branchement.as_ref() != Some(&branchement) {
            reglages.essai = None;
        }
        reglages.branchement = Some(branchement);
        ecrire_reglages(&app, &reglages)?;
        if let Ok(mut conversation) = app.state::<Ia>().conversation.lock() {
            *conversation = None;
        }
        Ok(etat(&app))
    })
    .await
}

#[tauri::command]
pub async fn essayer_ia(app: AppHandle) -> Result<String, String> {
    a_cote(move || {
        let mut reglages = lire_reglages(&app);
        let branchement = reglages
            .branchement
            .clone()
            .ok_or("L'IA n'est pas branchée.")?;
        let reponse = ia::essayer(&branchement, cle_rangee(&branchement).as_deref())?;
        reglages.essai = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs());
        ecrire_reglages(&app, &reglages)?;
        Ok(reponse)
    })
    .await
}

#[tauri::command]
pub async fn modeles_ia(
    branchement: Branchement,
    cle: Option<String>,
) -> Result<Vec<String>, String> {
    a_cote(move || {
        let cle = cle
            .filter(|c| !c.trim().is_empty())
            .or_else(|| cle_rangee(&branchement));
        ia::modeles(&branchement, cle.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn regler_ia(
    app: AppHandle,
    niveau: Niveau,
    dans_le_lab: Liberte,
) -> Result<EtatIa, String> {
    a_cote(move || {
        let mut reglages = lire_reglages(&app);
        reglages.niveau = niveau;
        reglages.dans_le_lab = dans_le_lab;
        ecrire_reglages(&app, &reglages)?;
        Ok(etat(&app))
    })
    .await
}

/// Un tour de conversation. Ce que fait l'IA part par l'évènement `ia`,
/// ses demandes d'accord par `ia-accord`.
#[tauri::command]
pub async fn parler_ia(app: AppHandle, texte: String) -> Result<(), String> {
    let ia_etat = app.state::<Ia>();
    if ia_etat.occupee.swap(true, Ordering::SeqCst) {
        return Err("L'IA répond déjà.".into());
    }
    ia_etat.arret.store(false, Ordering::SeqCst);
    let (accords, arret) = (ia_etat.accords.clone(), ia_etat.arret.clone());
    let app_tour = app.clone();
    let resultat = a_cote(move || {
        let reglages = lire_reglages(&app_tour);
        let branchement = reglages
            .branchement
            .clone()
            .ok_or("L'IA n'est pas branchée. Branche-la dans Réglages.")?;
        let cle = cle_rangee(&branchement);
        if branchement.cle().is_some() && cle.is_none() {
            return Err("La clé de l'IA manque. Range-la dans Réglages.".into());
        }
        let lab = le_lab();
        let consigne = ia::consigne(reglages.niveau, &ia::description_du_lab(&lab));
        let app_accord = app_tour.clone();
        let arret_accord = arret.clone();
        let execution = Execution {
            lab,
            liberte: reglages.dans_le_lab,
            accorder: Arc::new(move |demande: &Demande| {
                demander_accord(&app_accord, &accords, &arret_accord, demande)
            }),
        };
        let ia_etat = app_tour.state::<Ia>();
        let mut conversation = ia_etat
            .conversation
            .lock()
            .map_err(|e| e.to_string())?
            .take()
            .unwrap_or_default();
        let resultat = ia::parler(
            &branchement,
            cle.as_deref(),
            &mut conversation,
            &consigne,
            &texte,
            &execution,
            &mut |evenement| {
                let _ = app_tour.emit("ia", evenement);
            },
            &arret,
        );
        if let Ok(mut place) = ia_etat.conversation.lock() {
            *place = Some(conversation);
        }
        resultat
    })
    .await;
    app.state::<Ia>().occupee.store(false, Ordering::SeqCst);
    resultat
}

fn demander_accord(
    app: &AppHandle,
    accords: &Mutex<HashMap<String, Sender<bool>>>,
    arret: &AtomicBool,
    demande: &Demande,
) -> bool {
    let (envoi, reception) = mpsc::channel();
    if let Ok(mut attente) = accords.lock() {
        attente.insert(demande.id.clone(), envoi);
    }
    let _ = app.emit("ia-accord", demande);
    loop {
        match reception.recv_timeout(Duration::from_millis(250)) {
            Ok(oui) => return oui,
            Err(RecvTimeoutError::Timeout) if !arret.load(Ordering::SeqCst) => {}
            _ => {
                if let Ok(mut attente) = accords.lock() {
                    attente.remove(&demande.id);
                }
                return false;
            }
        }
    }
}

#[tauri::command]
pub fn repondre_accord(ia_etat: State<'_, Ia>, id: String, oui: bool) {
    let envoi = ia_etat.accords.lock().ok().and_then(|mut a| a.remove(&id));
    if let Some(envoi) = envoi {
        let _ = envoi.send(oui);
    }
}

/// Arrête l'IA : elle finit l'appel en cours, et ce qu'elle demandait est
/// refusé.
#[tauri::command]
pub fn arreter_ia(ia_etat: State<'_, Ia>) {
    ia_etat.arret.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub fn oublier_conversation(ia_etat: State<'_, Ia>) {
    if let Ok(mut conversation) = ia_etat.conversation.lock() {
        *conversation = None;
    }
}
