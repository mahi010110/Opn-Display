//! L'IA côté app : ses réglages, sa clé dans le trousseau du système, la
//! conversation, et l'accord de l'utilisateur.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use moteur::ia::journal::{self, Entree};
use moteur::ia::memoire::{Liste, Memoire};
use moteur::ia::reglage::Issue;
use moteur::ia::{
    self, Action, Branchement, Comportements, Conversation, Demande, Evenement, Execution, Facon,
    Initiative, Inventer, Memoriser, Niveau, Noter, Proposer, Reglage,
};
use moteur::lab::Lab;
use moteur::pannes::{Difficulte, Panne};
use moteur::proposition::{self, MACHINES_MAX, Proposition};
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

/// Un lab à proposer : ce que l'IA a proposé, les propositions refusées, et
/// ce qu'elle a écrit hors de l'outil, souvent une question.
#[derive(Default)]
struct Plan {
    recu: Arc<Mutex<Option<(Lab, String)>>>,
    ratees: Arc<AtomicUsize>,
    ecrit: Arc<Mutex<String>>,
}

/// Ce qu'on demande à l'IA.
enum Travail {
    /// Parler avec l'utilisateur, dans le panneau du lab ouvert.
    Discussion,
    /// Lui proposer de l'aide quand il bloque sur une panne, en une phrase.
    Initiative,
    Invention(Invention),
    Proposition(Plan),
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Reglages {
    #[serde(default)]
    pub branchement: Option<Branchement>,
    /// Comment elle aide l'utilisateur. Un « dansLeLab » laissé par une
    /// version d'avant est ignoré : agir sans demander se choisit
    /// maintenant « un par un ».
    #[serde(flatten)]
    pub reglage: Reglage,
    /// Le dernier essai réussi, en secondes depuis 1970.
    #[serde(default)]
    pub essai: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtatIa {
    reglages: Reglages,
    /// Ce qu'elle fait en ce moment, quelle que soit la façon de régler.
    comportements: Comportements,
    /// Le niveau qui décide en ce moment, sauf réglée un par un.
    niveau_du_moment: Option<Niveau>,
    cle_rangee: bool,
    codex_present: bool,
    /// Les modèles d'Ollama, ou rien s'il ne répond pas.
    ollama: Option<Vec<String>>,
}

#[derive(Default)]
pub struct Ia {
    conversation: Mutex<Option<Conversation>>,
    /// La conversation de l'écran Nouveau lab : on y redemande un
    /// changement, et l'IA se souvient du lab qu'elle a proposé.
    proposition: Mutex<Option<Conversation>>,
    accords: Arc<Mutex<HashMap<String, Sender<bool>>>>,
    arret: Arc<AtomicBool>,
    occupee: AtomicBool,
    /// La mémoire et le journal s'écrivent un à la fois : l'IA et
    /// l'utilisateur peuvent y toucher en même temps.
    fichiers: Mutex<()>,
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

fn maintenant() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

/// Ce que l'IA fait en ce moment, pour les pannes et l'accord.
pub(crate) fn comportements(app: &AppHandle) -> Comportements {
    lire_reglages(app).reglage.comportements()
}

/// Une panne est finie : l'IA qui s'adapte change peut-être de niveau.
pub(crate) fn adapter(app: &AppHandle, issue: Issue) {
    let mut reglages = lire_reglages(app);
    if reglages.reglage.facon == Facon::SAdapte {
        reglages.reglage = reglages.reglage.apres(&issue);
        let _ = ecrire_reglages(app, &reglages);
    }
}

fn chemin_memoire(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("memoire.json"))
}

fn lire_memoire(app: &AppHandle) -> Memoire {
    chemin_memoire(app)
        .ok()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|texte| serde_json::from_str(&texte).ok())
        .unwrap_or_default()
}

/// Change la mémoire, un changement à la fois, et l'écrit.
fn changer_memoire<T>(
    app: &AppHandle,
    changement: impl FnOnce(&mut Memoire) -> Result<T, String>,
) -> Result<T, String> {
    let ia_etat = app.state::<Ia>();
    let _garde = ia_etat.fichiers.lock().map_err(|e| e.to_string())?;
    let mut memoire = lire_memoire(app);
    let resultat = changement(&mut memoire)?;
    let chemin = chemin_memoire(app)?;
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier).map_err(|e| e.to_string())?;
    }
    let texte = serde_json::to_string_pretty(&memoire).map_err(|e| e.to_string())?;
    std::fs::write(chemin, texte).map_err(|e| format!("La mémoire ne s'écrit pas : {e}"))?;
    Ok(resultat)
}

fn chemin_journal(app: &AppHandle) -> Option<PathBuf> {
    Some(app.path().app_data_dir().ok()?.join("journal.json"))
}

/// Écrit dans le journal ce que l'IA a fait. Un journal qui ne s'écrit pas
/// n'arrête pas l'IA.
pub(crate) fn noter(app: &AppHandle, action: Action) {
    let Some(chemin) = chemin_journal(app) else {
        return;
    };
    let ia_etat = app.state::<Ia>();
    let Ok(_garde) = ia_etat.fichiers.lock() else {
        return;
    };
    let _ = journal::ajouter(
        &chemin,
        Entree {
            quand: maintenant(),
            action,
        },
    );
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
        comportements: reglages.reglage.comportements(),
        niveau_du_moment: reglages.reglage.niveau_du_moment(),
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
        reglages.essai = Some(maintenant());
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

/// Change la façon de régler l'IA, son niveau, ou ses comportements un par
/// un.
#[tauri::command]
pub async fn regler_ia(
    app: AppHandle,
    facon: Facon,
    niveau: Niveau,
    un_par_un: Comportements,
) -> Result<EtatIa, String> {
    a_cote(move || {
        let mut reglages = lire_reglages(&app);
        reglages.reglage = reglages.reglage.changer(facon, niveau, un_par_un);
        ecrire_reglages(&app, &reglages)?;
        Ok(etat(&app))
    })
    .await
}

#[tauri::command]
pub async fn memoire_ia(app: AppHandle) -> Result<Memoire, String> {
    a_cote(move || Ok(lire_memoire(&app))).await
}

/// L'utilisateur coupe ou rebranche sa mémoire.
#[tauri::command]
pub async fn activer_memoire(app: AppHandle, active: bool) -> Result<Memoire, String> {
    a_cote(move || {
        changer_memoire(&app, |memoire| {
            memoire.active = active;
            Ok(memoire.clone())
        })
    })
    .await
}

/// Une ligne corrigée par l'utilisateur. Vide, elle part.
#[tauri::command]
pub async fn corriger_memoire(
    app: AppHandle,
    liste: Liste,
    rang: usize,
    texte: String,
) -> Result<Memoire, String> {
    a_cote(move || {
        changer_memoire(&app, |memoire| {
            memoire.corriger(liste, rang, &texte)?;
            Ok(memoire.clone())
        })
    })
    .await
}

#[tauri::command]
pub async fn retirer_de_la_memoire(
    app: AppHandle,
    liste: Liste,
    rang: usize,
) -> Result<Memoire, String> {
    a_cote(move || {
        changer_memoire(&app, |memoire| {
            memoire.retirer(liste, rang);
            Ok(memoire.clone())
        })
    })
    .await
}

#[tauri::command]
pub async fn effacer_memoire(app: AppHandle) -> Result<Memoire, String> {
    a_cote(move || {
        changer_memoire(&app, |memoire| {
            memoire.effacer();
            Ok(memoire.clone())
        })
    })
    .await
}

/// Le journal, du plus récent au plus ancien.
#[tauri::command]
pub async fn journal_ia(app: AppHandle) -> Result<Vec<Entree>, String> {
    a_cote(move || {
        let mut entrees = chemin_journal(&app)
            .map(|chemin| journal::lire(&chemin))
            .unwrap_or_default();
        entrees.reverse();
        Ok(entrees)
    })
    .await
}

/// Un tour de conversation. Ce que fait l'IA part par l'évènement `ia`,
/// ses demandes d'accord par `ia-accord`.
#[tauri::command]
pub async fn parler_ia(app: AppHandle, texte: String) -> Result<(), String> {
    tour(app, texte, Travail::Discussion).await
}

/// L'utilisateur bloque sur une panne : l'IA lui propose de l'aide, en une
/// phrase, si elle a le droit de parler d'elle-même. Le panneau décide quand.
#[tauri::command]
pub async fn aider_ia(app: AppHandle) -> Result<(), String> {
    if comportements(&app).initiative != Initiative::QuandTuBloques || !pannes::en_cours_ici(&app) {
        return Ok(());
    }
    tour(
        app,
        "(Ce message vient de l'app, pas de l'utilisateur.) L'utilisateur cherche la panne \
         depuis plus de dix minutes sans t'écrire. Propose-lui ton aide en une seule phrase \
         courte, sans donner la cause."
            .into(),
        Travail::Initiative,
    )
    .await
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
        Travail::Invention(invention),
    )
    .await?;
    if let Some(vue) = pannes::panne_en_cours(app.clone(), app.state()).filter(|vue| vue.inventee())
    {
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

/// Le lab que propose l'IA, et ce qu'elle en dit. Sans lab, elle a posé
/// une question.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropositionLab {
    lab: Option<Lab>,
    commentaire: String,
}

/// L'IA propose un lab d'après la description de l'utilisateur. Rien quand
/// l'utilisateur l'a arrêtée. `actuel` est le lab proposé tel qu'il est à
/// l'écran : l'utilisateur a pu en retirer des machines, et l'IA ne doit pas
/// les remettre.
#[tauri::command]
pub async fn proposer_lab(
    app: AppHandle,
    description: String,
    actuel: Option<Lab>,
) -> Result<Option<PropositionLab>, String> {
    let demande = description.trim().to_string();
    if demande.is_empty() {
        return Err("Décris le lab que tu veux.".into());
    }
    let texte = match actuel {
        Some(actuel) => format!(
            "Le lab proposé en ce moment, tel que l'utilisateur le voit (il a pu retirer des \
             machines) :\n{}\n\nSa demande : {demande}",
            serde_json::to_string(&proposition::en_proposition(&actuel))
                .map_err(|e| e.to_string())?
        ),
        None => demande,
    };
    let plan = Plan::default();
    let (recu, ratees, ecrit) = (plan.recu.clone(), plan.ratees.clone(), plan.ecrit.clone());
    tour(app.clone(), texte, Travail::Proposition(plan)).await?;
    let ecrit = ecrit
        .lock()
        .map(|e| e.trim().to_string())
        .unwrap_or_default();
    if let Some((lab, commentaire)) = recu.lock().map_err(|e| e.to_string())?.take() {
        noter(
            &app,
            Action {
                lieu: "Nouveau lab".into(),
                quoi: format!(
                    "Lab « {} » proposé, {} machines",
                    lab.nom,
                    lab.machines.len()
                ),
                commande: None,
                accord: None,
            },
        );
        return Ok(Some(PropositionLab {
            lab: Some(lab),
            commentaire,
        }));
    }
    if ratees.load(Ordering::SeqCst) >= ESSAIS_D_INVENTION {
        return Err(format!(
            "L'IA n'a pas réussi à proposer un lab qui tourne, en {ESSAIS_D_INVENTION} essais. \
             Reformule ta demande, ou demande plus simple."
        ));
    }
    if app.state::<Ia>().arret.load(Ordering::SeqCst) {
        return Ok(None);
    }
    if !ecrit.is_empty() {
        return Ok(Some(PropositionLab {
            lab: None,
            commentaire: ecrit,
        }));
    }
    Err("L'IA n'a pas proposé de lab. Reformule ta demande.".into())
}

/// L'écran Nouveau lab repart de zéro.
#[tauri::command]
pub fn oublier_proposition(ia_etat: State<'_, Ia>) {
    if let Ok(mut conversation) = ia_etat.proposition.lock() {
        *conversation = None;
    }
}

/// Le lab ouvert change : la discussion du panneau repart de zéro.
pub(crate) fn oublier(app: &AppHandle) {
    if let Ok(mut conversation) = app.state::<Ia>().conversation.lock() {
        *conversation = None;
    }
}

fn consigne_de_proposition() -> String {
    format!(
        "Tu es l'IA d'Opn-Display, une app pour apprendre l'administration d'infrastructures. Tu \
         parles français, tu tutoies, sans emoji. L'utilisateur décrit un lab à construire : \
         propose-le avec l'outil proposer_un_lab. Il le verra en schéma avant de le lancer.\n\
         Pour l'instant, l'app fait tourner des postes, des serveurs et des routeurs Debian, et \
         des switchs simples, sans VLAN. Pas encore de pare-feu, de Windows, d'accès à Internet \
         ni de logiciel à installer : si la demande en a besoin, propose ce qui s'en approche et \
         dis-le dans le commentaire.\n\
         Donne une adresse à chaque bout de câble, sauf côté switch, une passerelle aux postes et \
         aux serveurs, et des routes aux routeurs, pour que toutes les machines se joignent, sauf \
         si l'utilisateur demande autre chose. Des noms courts : pc1, srv1, r1, sw1. \
         {MACHINES_MAX} machines au plus.\n\
         Si l'utilisateur demande un changement, propose le lab entier à nouveau. Une fois le lab \
         proposé, réponds seulement : proposé."
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

async fn tour(app: AppHandle, texte: String, travail: Travail) -> Result<(), String> {
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
        let lab = le_lab(&app_tour);
        let comportements = reglages.reglage.comportements();
        let memoire = lire_memoire(&app_tour);
        let parole = matches!(travail, Travail::Initiative);
        let consigne = match &travail {
            Travail::Proposition(_) => consigne_de_proposition(),
            Travail::Discussion | Travail::Initiative | Travail::Invention(_) => {
                // L'invention a sa conversation à part : la mémoire de
                // l'utilisateur n'y sert pas.
                let pour_la_memoire = match travail {
                    Travail::Invention(_) => None,
                    _ => memoire.pour_la_consigne(),
                };
                let mut consigne = ia::consigne(
                    comportements.solution,
                    &ia::description_du_lab(&lab),
                    pour_la_memoire.as_deref(),
                );
                if let Some(contexte) = pannes::contexte(&app_tour, comportements.solution) {
                    consigne.push_str("\n\n");
                    consigne.push_str(&contexte);
                }
                if let Travail::Invention(invention) = &travail {
                    consigne.push_str("\n\n");
                    consigne.push_str(&consigne_d_invention(invention.niveau));
                }
                consigne
            }
        };
        let (invention, plan) = match travail {
            Travail::Discussion | Travail::Initiative => (None, None),
            Travail::Invention(invention) => (Some(invention), None),
            Travail::Proposition(plan) => (None, Some(plan)),
        };
        let discussion = invention.is_none() && plan.is_none();
        let ecrit = plan.as_ref().map(|p| p.ecrit.clone());
        let app_accord = app_tour.clone();
        let arret_accord = arret.clone();
        let app_panne = app_tour.clone();
        let arret_plan = arret.clone();
        let nom_du_lab = lab.nom.clone();
        let app_memoire = app_tour.clone();
        let memoriser: Memoriser = Arc::new(move |liste, texte, remplace| {
            changer_memoire(&app_memoire, |memoire| {
                memoire.retenir(liste, texte, remplace)
            })
        });
        let app_journal = app_tour.clone();
        let noter_ici: Noter = Arc::new(move |action| noter(&app_journal, action));
        let execution = Execution {
            lab,
            liberte: comportements.dans_le_lab,
            memoriser: (discussion && !parole && memoire.active).then_some(memoriser),
            noter: Some(noter_ici),
            parole,
            accorder: Arc::new(move |demande: &Demande| {
                demander_accord(&app_accord, &accords, &arret_accord, demande)
            }),
            proposer: plan.map(|Plan { recu, ratees, .. }| {
                Arc::new(move |proposee: Proposition, commentaire: String| {
                    match proposition::construire(&proposee) {
                        Ok(lab) => {
                            if let Ok(mut place) = recu.lock() {
                                *place = Some((lab, commentaire));
                            }
                            Ok(
                                "Le lab est proposé à l'utilisateur, qui le voit en schéma. \
                                Réponds seulement : proposé."
                                    .to_string(),
                            )
                        }
                        Err(erreur) => {
                            if ratees.fetch_add(1, Ordering::SeqCst) + 1 >= ESSAIS_D_INVENTION {
                                arret_plan.store(true, Ordering::SeqCst);
                            }
                            Err(erreur)
                        }
                    }
                }) as Proposer
            }),
            inventer: invention.map(|Invention { niveau, ratees }| {
                let arret = arret.clone();
                Arc::new(move |mut panne: Panne| {
                    panne.niveau = niveau;
                    panne.famille = "Inventée".into();
                    let titre = panne.titre.clone();
                    match pannes::poser_et_retenir(&app_panne, panne) {
                        Ok(_) => {
                            for quoi in [
                                "Lab sauvegardé avant la panne".to_string(),
                                format!(
                                    "Panne inventée posée : « {} »",
                                    titre.trim_end_matches('.')
                                ),
                            ] {
                                noter(
                                    &app_panne,
                                    Action {
                                        lieu: nom_du_lab.clone(),
                                        quoi,
                                        commande: None,
                                        accord: None,
                                    },
                                );
                            }
                            Ok("La panne est posée, et ses tests échouent comme prévu. \
                                Réponds seulement : prête."
                                .to_string())
                        }
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
        // L'invention a sa propre conversation, qui ne reste pas : rien ne
        // passe dans celle de l'utilisateur, et rien ne s'affiche. La
        // proposition garde la sienne, pour qu'on y demande un changement.
        let gardee = if discussion {
            Some(&ia_etat.conversation)
        } else if ecrit.is_some() {
            Some(&ia_etat.proposition)
        } else {
            None
        };
        let mut conversation = match gardee {
            Some(place) => place
                .lock()
                .map_err(|e| e.to_string())?
                .take()
                .unwrap_or_default(),
            None => Conversation::default(),
        };
        let resultat = ia::parler(
            &branchement,
            cle.as_deref(),
            &mut conversation,
            &consigne,
            &texte,
            &execution,
            &mut |evenement| {
                if discussion {
                    let _ = app_tour.emit("ia", evenement);
                } else if let (Some(ecrit), Evenement::Texte { texte }) = (&ecrit, &evenement)
                    && let Ok(mut place) = ecrit.lock()
                {
                    *place = texte.clone();
                }
            },
            &arret,
        );
        if let Some(place) = gardee
            && let Ok(mut place) = place.lock()
        {
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
