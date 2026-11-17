//! L'IA côté app : ses réglages, sa clé dans le trousseau du système, la
//! conversation, et l'accord de l'utilisateur.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use moteur::ia::{self, Branchement, Conversation, Demande, Execution, Inventer, Liberte, Niveau};
use moteur::pannes::{Difficulte, Panne};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::pannes::{self, VuePanne};
use crate::{a_cote, le_lab};

const SERVICE: &str = "opn-display";
const OLLAMA: &str = "http://127.0.0.1:11434/v1";
/// Au-delà, l'invention s'arrête : chaque essai raté coûte une sauvegarde
/// et un retour en arrière du lab, et un modèle local met des minutes.
const ESSAIS_D_INVENTION: usize = 3;

/// Une panne à inventer : son niveau, et les poses qui n'ont pas pris.
struct Invention {
    niveau: Difficulte,
    ratees: Arc<AtomicUsize>,
}

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
    tour(app, texte, None).await
}

/// L'IA invente une panne et la pose, sans rien montrer de ce qu'elle fait.
/// Rien quand l'utilisateur l'a arrêtée.
#[tauri::command]
pub async fn inventer_panne(
    app: AppHandle,
    niveau: Difficulte,
) -> Result<Option<VuePanne>, String> {
    if pannes::deja_posee(&app) {
        return Err("Une panne est déjà posée. Répare-la ou abandonne-la d'abord.".into());
    }
    let ratees = Arc::new(AtomicUsize::new(0));
    let invention = Invention {
        niveau,
        ratees: ratees.clone(),
    };
    tour(
        app.clone(),
        "Invente la panne et pose-la.".into(),
        Some(invention),
    )
    .await?;
    if let Some(vue) = pannes::panne_en_cours(app.state()).filter(|vue| vue.inventee()) {
        return Ok(Some(vue));
    }
    if ratees.load(Ordering::SeqCst) >= ESSAIS_D_INVENTION {
        return Err(format!(
            "L'IA n'a pas réussi à poser une panne en {ESSAIS_D_INVENTION} essais. Essaie encore, ou prends une panne du catalogue."
        ));
    }
    if app.state::<Ia>().arret.load(Ordering::SeqCst) {
        return Ok(None);
    }
    Err(
        "L'IA n'a pas réussi à poser une panne. Essaie encore, ou prends une panne du catalogue."
            .into(),
    )
}

fn consigne_d_invention(niveau: Difficulte) -> String {
    let mot = match niveau {
        Difficulte::Facile => "facile : une seule chose cassée, qu'on voit vite",
        Difficulte::Moyen => "moyen : une seule chose cassée, moins visible",
        Difficulte::Difficile => "difficile : deux choses cassées, ou une cause loin du symptôme",
    };
    format!(
        "L'utilisateur demande une panne inventée, de niveau {mot}. Regarde d'abord le lab avec \
         tes outils, puis pose la panne avec l'outil poser_une_panne. Elle doit être réaliste \
         pour un administrateur d'infrastructure, réparable depuis les terminaux du lab, et tenir \
         après un redémarrage si possible. Ne coupe jamais l'accès aux machines. Une fois la \
         panne posée, réponds seulement : prête."
    )
}

async fn tour(app: AppHandle, texte: String, invention: Option<Invention>) -> Result<(), String> {
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
        let mut consigne = ia::consigne(reglages.niveau, &ia::description_du_lab(&lab));
        if let Some(contexte) = pannes::contexte(&app_tour) {
            consigne.push_str("\n\n");
            consigne.push_str(&contexte);
        }
        let discret = invention.is_some();
        if let Some(invention) = &invention {
            consigne.push_str("\n\n");
            consigne.push_str(&consigne_d_invention(invention.niveau));
        }
        let app_accord = app_tour.clone();
        let arret_accord = arret.clone();
        let app_panne = app_tour.clone();
        let execution = Execution {
            lab,
            liberte: reglages.dans_le_lab,
            accorder: Arc::new(move |demande: &Demande| {
                demander_accord(&app_accord, &accords, &arret_accord, demande)
            }),
            inventer: invention.map(|Invention { niveau, ratees }| {
                let arret = arret.clone();
                Arc::new(move |mut panne: Panne| {
                    panne.niveau = niveau;
                    panne.famille = "Inventée".into();
                    match pannes::poser_et_retenir(&app_panne, panne) {
                        Ok(_) => Ok("La panne est posée, et ses tests échouent comme prévu. \
                                     Réponds seulement : prête."
                            .to_string()),
                        Err(erreur) => {
                            if ratees.fetch_add(1, Ordering::SeqCst) + 1 >= ESSAIS_D_INVENTION {
                                arret.store(true, Ordering::SeqCst);
                            } else {
                                let _ = app_panne.emit("lab-avance", "L'IA invente une panne");
                            }
                            Err(erreur)
                        }
                    }
                }) as Inventer
            }),
        };
        let ia_etat = app_tour.state::<Ia>();
        // L'invention a sa propre conversation : rien ne reste dans celle
        // de l'utilisateur, et rien ne s'affiche.
        let mut conversation = if discret {
            Conversation::default()
        } else {
            ia_etat
                .conversation
                .lock()
                .map_err(|e| e.to_string())?
                .take()
                .unwrap_or_default()
        };
        let resultat = ia::parler(
            &branchement,
            cle.as_deref(),
            &mut conversation,
            &consigne,
            &texte,
            &execution,
            &mut |evenement| {
                if !discret {
                    let _ = app_tour.emit("ia", evenement);
                }
            },
            &arret,
        );
        if !discret && let Ok(mut place) = ia_etat.conversation.lock() {
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
