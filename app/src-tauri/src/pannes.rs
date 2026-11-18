//! Les pannes côté app : celle qui est posée, ses indices, sa vérification.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use moteur::ia::reglage::Issue;
use moteur::ia::{Action, Solution};
use moteur::lab;
use moteur::pannes::{self, Difficulte, Panne, ResultatTest};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{a_cote, le_lab, un_lab};

/// La panne posée. Elle est gardée dans `panne.json` jusqu'à sa fin : le
/// lab la garde aussi quand l'app est fermée.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnCours {
    panne: Panne,
    /// Le lab où elle est posée. Une panne gardée avant les labs construits
    /// était dans le Premier lab.
    #[serde(default = "premier_lab")]
    lab: String,
    posee: u64,
    indices_donnes: usize,
    #[serde(skip)]
    verification: Option<Verification>,
}

fn premier_lab() -> String {
    lab::premier_lab().id
}

#[derive(Default)]
pub struct Pannes {
    en_cours: Mutex<Option<EnCours>>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Verification {
    reparee: bool,
    tests: Vec<ResultatTest>,
    /// La cause, dite seulement une fois la panne réparée.
    explication: Option<String>,
}

/// Ce que l'écran sait de la panne posée : jamais sa cause avant la fin.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VuePanne {
    /// Le lab où elle est posée, et son nom.
    lab: String,
    nom_du_lab: String,
    titre: String,
    niveau: Difficulte,
    inventee: bool,
    posee: u64,
    tests: usize,
    indices: usize,
    indices_donnes: usize,
    /// Les indices déjà donnés, pour les remontrer quand l'app rouvre.
    indices_lus: Vec<String>,
    verification: Option<Verification>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumePanne {
    id: String,
    titre: String,
    famille: String,
    niveau: Difficulte,
}

fn fichier(app: &AppHandle) -> Option<PathBuf> {
    Some(app.path().app_config_dir().ok()?.join("panne.json"))
}

/// Écrit la panne en cours, ou efface le fichier quand elle est finie.
fn garder(app: &AppHandle, en_cours: Option<&EnCours>) {
    let Some(chemin) = fichier(app) else {
        return;
    };
    let finie = en_cours.is_none_or(|e| e.verification.as_ref().is_some_and(|v| v.reparee));
    if finie {
        let _ = fs::remove_file(&chemin);
        return;
    }
    if let (Some(dossier), Ok(texte)) = (chemin.parent(), serde_json::to_string_pretty(&en_cours)) {
        let _ = fs::create_dir_all(dossier);
        let _ = fs::write(&chemin, texte);
    }
}

/// À l'ouverture de l'app : la panne laissée en cours la dernière fois.
pub fn reprendre(app: &AppHandle) {
    let Some(chemin) = fichier(app) else {
        return;
    };
    let Ok(texte) = fs::read_to_string(&chemin) else {
        return;
    };
    match serde_json::from_str::<EnCours>(&texte) {
        Ok(en_cours) => {
            if let Ok(mut place) = app.state::<Pannes>().en_cours.lock() {
                *place = Some(en_cours);
            }
        }
        Err(_) => {
            let _ = fs::remove_file(&chemin);
        }
    }
}

fn oublier_la_panne(app: &AppHandle) {
    if let Ok(mut place) = app.state::<Pannes>().en_cours.lock() {
        *place = None;
    }
    garder(app, None);
}

/// Le lab est reparti de zéro, ou il est supprimé : la panne qu'il portait
/// n'existe plus.
pub fn oublier_la_panne_du_lab(app: &AppHandle, id: &str) {
    let dans_ce_lab = app
        .state::<Pannes>()
        .en_cours
        .lock()
        .ok()
        .is_some_and(|e| e.as_ref().is_some_and(|e| e.lab == id));
    if dans_ce_lab {
        oublier_la_panne(app);
    }
}

fn maintenant() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

impl VuePanne {
    pub(crate) fn inventee(&self) -> bool {
        self.inventee
    }
}

fn vue(app: &AppHandle, en_cours: &EnCours) -> VuePanne {
    VuePanne {
        lab: en_cours.lab.clone(),
        nom_du_lab: un_lab(app, &en_cours.lab)
            .map(|l| l.nom)
            .unwrap_or_default(),
        titre: en_cours.panne.titre.clone(),
        niveau: en_cours.panne.niveau,
        inventee: en_cours.panne.inventee,
        posee: en_cours.posee,
        tests: en_cours.panne.tests.len(),
        indices: en_cours.panne.indices.len(),
        indices_donnes: en_cours.indices_donnes,
        indices_lus: (0..en_cours.indices_donnes)
            .filter_map(|n| texte_indice(&en_cours.panne, n))
            .collect(),
        verification: en_cours.verification.clone(),
    }
}

fn texte_indice(panne: &Panne, n: usize) -> Option<String> {
    let indice = panne.indices.get(n)?;
    Some(format!(
        "Indice {} sur {}. {indice}",
        n + 1,
        panne.indices.len()
    ))
}

fn active(app: &AppHandle) -> bool {
    app.state::<Pannes>().en_cours.lock().ok().is_some_and(|e| {
        e.as_ref()
            .is_some_and(|e| !e.verification.as_ref().is_some_and(|v| v.reparee))
    })
}

/// Pose la panne, la retient, et le dit aux écrans.
pub fn poser_et_retenir(app: &AppHandle, panne: Panne) -> Result<VuePanne, String> {
    let lab = le_lab(app);
    pannes::poser(&lab, &panne, &mut |avance| {
        let _ = app.emit("lab-avance", avance);
    })?;
    let en_cours = EnCours {
        panne,
        lab: lab.id,
        posee: maintenant(),
        indices_donnes: 0,
        verification: None,
    };
    let vue = vue(app, &en_cours);
    garder(app, Some(&en_cours));
    *app.state::<Pannes>()
        .en_cours
        .lock()
        .map_err(|e| e.to_string())? = Some(en_cours);
    let _ = app.emit("panne", &vue);
    Ok(vue)
}

/// Une panne attend dans le lab ouvert.
pub(crate) fn en_cours_ici(app: &AppHandle) -> bool {
    let ouvert = le_lab(app).id;
    app.state::<Pannes>().en_cours.lock().ok().is_some_and(|e| {
        e.as_ref()
            .is_some_and(|e| e.lab == ouvert && !e.verification.as_ref().is_some_and(|v| v.reparee))
    })
}

/// Ce que l'IA doit savoir de la panne posée, selon ce qu'elle fait quand
/// on lui demande la solution. Comme à un examen, elle n'en connaît pas la
/// cause : elle ne peut pas la laisser échapper.
pub fn contexte(app: &AppHandle, solution: Solution) -> Option<String> {
    let etat = app.state::<Pannes>();
    let place = etat.en_cours.lock().ok()?;
    let en_cours = place.as_ref()?;
    if en_cours.verification.as_ref().is_some_and(|v| v.reparee) || en_cours.lab != le_lab(app).id {
        return None;
    }
    let (titre, cause) = (&en_cours.panne.titre, &en_cours.panne.explication);
    Some(match solution {
        Solution::Donnee => format!(
            "Une panne est posée dans le lab : « {titre} ». Sa cause : {cause}\n\
             Si on te demande la solution, donne la cause et explique-la. Ne la répare pas \
             toi-même, sauf si on te le demande."
        ),
        Solution::Indices => format!(
            "Une panne est posée dans le lab : « {titre} ». Sa cause, que l'utilisateur doit \
             trouver : {cause}\n\
             Ne donne jamais la cause : guide par des questions et des indices. Ne la répare pas \
             toi-même."
        ),
        Solution::Rien => format!(
            "Une panne est posée dans le lab : « {titre} ». L'utilisateur la répare seul, comme à \
             un examen : ne dis rien de la panne, ni cause ni indice, et ne la répare pas."
        ),
    })
}

/// Les pannes prêtes pour le lab ouvert.
#[tauri::command]
pub fn catalogue_pannes(app: AppHandle) -> Vec<ResumePanne> {
    pannes::catalogue(&le_lab(&app))
        .into_iter()
        .map(|p| ResumePanne {
            id: p.id,
            titre: p.titre,
            famille: p.famille,
            niveau: p.niveau,
        })
        .collect()
}

#[tauri::command]
pub async fn poser_panne(app: AppHandle, id: String) -> Result<VuePanne, String> {
    a_cote(move || {
        if active(&app) {
            return Err("Une panne est déjà posée. Répare-la ou abandonne-la d'abord.".into());
        }
        let panne = pannes::catalogue(&le_lab(&app))
            .into_iter()
            .find(|p| p.id == id)
            .ok_or("Cette panne n'est pas dans le catalogue.")?;
        poser_et_retenir(&app, panne)
    })
    .await
}

#[tauri::command]
pub fn panne_en_cours(app: AppHandle, etat: State<'_, Pannes>) -> Option<VuePanne> {
    etat.en_cours.lock().ok()?.as_ref().map(|e| vue(&app, e))
}

#[tauri::command]
pub fn indice_panne(app: AppHandle, etat: State<'_, Pannes>) -> Result<String, String> {
    if crate::ia::comportements(&app).solution == Solution::Rien {
        return Err("Pas d'indice : l'IA est réglée comme pour un examen.".into());
    }
    let mut place = etat.en_cours.lock().map_err(|e| e.to_string())?;
    let en_cours = place.as_mut().ok_or("Aucune panne n'est posée.")?;
    let texte = texte_indice(&en_cours.panne, en_cours.indices_donnes)
        .ok_or("Il n'y a plus d'indice. Demande à l'IA, ou regarde les tests.")?;
    en_cours.indices_donnes += 1;
    garder(&app, Some(en_cours));
    Ok(texte)
}

#[tauri::command]
pub async fn verifier_panne(app: AppHandle) -> Result<VuePanne, String> {
    a_cote(move || {
        let (panne, son_lab) = app
            .state::<Pannes>()
            .en_cours
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .map(|e| (e.panne.clone(), e.lab.clone()))
            .ok_or("Aucune panne n'est posée.")?;
        let lab = un_lab(&app, &son_lab)?;
        // Le bouton de la carte dit déjà « Vérification » : rien de plus à
        // montrer pendant que le lab démarre.
        lab::lancer_et_attendre(&lab, &mut |_| {})?;
        let tests = pannes::verifier(&lab, &panne);
        let reparee = tests.iter().all(|t| t.reussi);
        if reparee {
            pannes::oublier(&lab);
        }
        if panne.inventee {
            let reussis = tests.iter().filter(|t| t.reussi).count();
            let mut machines: Vec<&str> = Vec::new();
            for test in &tests {
                if !machines.contains(&test.machine.as_str()) {
                    machines.push(&test.machine);
                }
            }
            crate::ia::noter(
                &app,
                Action {
                    lieu: machines.join(", "),
                    quoi: format!(
                        "Tests de la panne : {reussis} {} sur {}",
                        if reussis > 1 { "réussis" } else { "réussi" },
                        tests.len()
                    ),
                    commande: None,
                    accord: None,
                },
            );
        }
        let etat = app.state::<Pannes>();
        let (vue, premiere_reparation) = {
            let mut place = etat.en_cours.lock().map_err(|e| e.to_string())?;
            let en_cours = place.as_mut().ok_or("Aucune panne n'est posée.")?;
            let deja = en_cours.verification.as_ref().is_some_and(|v| v.reparee);
            let indices = en_cours.indices_donnes;
            en_cours.verification = Some(Verification {
                reparee,
                tests,
                explication: reparee.then(|| panne.explication.clone()),
            });
            garder(&app, Some(en_cours));
            (vue(&app, en_cours), (reparee && !deja).then_some(indices))
        };
        // « Relancer les tests » sur une panne déjà réparée ne compte pas
        // une deuxième fois.
        if let Some(indices) = premiere_reparation {
            crate::ia::adapter(
                &app,
                Issue::Reparee {
                    titre: &panne.titre,
                    indices,
                },
            );
        }
        Ok(vue)
    })
    .await
}

/// Le lab revient comme juste avant la panne.
#[tauri::command]
pub async fn abandonner_panne(app: AppHandle) -> Result<(), String> {
    a_cote(move || {
        let posee = app
            .state::<Pannes>()
            .en_cours
            .lock()
            .ok()
            .and_then(|e| e.as_ref().map(|e| (e.lab.clone(), e.panne.titre.clone())));
        if let Some((id, titre)) = posee
            && active(&app)
        {
            pannes::revenir(&un_lab(&app, &id)?)?;
            crate::ia::adapter(&app, Issue::Abandonnee { titre: &titre });
        }
        oublier_la_panne(&app);
        Ok(())
    })
    .await
}

/// Une panne réparée quitte le panneau de l'IA.
#[tauri::command]
pub fn fermer_panne(app: AppHandle, etat: State<'_, Pannes>) {
    if let Ok(mut place) = etat.en_cours.lock()
        && place
            .as_ref()
            .is_some_and(|e| e.verification.as_ref().is_some_and(|v| v.reparee))
    {
        *place = None;
        garder(&app, None);
    }
}

pub(crate) fn deja_posee(app: &AppHandle) -> bool {
    active(app)
}
