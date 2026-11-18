//! Les outils de l'IA dans le lab, et l'accord de l'utilisateur.
//!
//! C'est l'app qui applique le réglage d'accord, jamais la consigne de
//! l'IA : une IA peut l'oublier, ou se le faire retirer par un texte piégé
//! (`docs/architecture.md`, partie 3).

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::memoire::Liste;
use super::{Appel, Evenement};
use crate::lab::{self, Lab, Marche};
use crate::pannes::Panne;
use crate::proposition::{MACHINES_MAX, Proposition};

#[derive(Clone)]
pub struct Outil {
    pub nom: String,
    pub description: String,
    pub parametres: Value,
}

/// Ce que fait l'IA, qui décide de ses outils.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Elle parle avec l'utilisateur, dans le lab ouvert.
    Lab,
    /// La même chose, et elle peut écrire dans sa mémoire.
    LabEtMemoire,
    /// Elle invente une panne et la pose.
    Invention,
    /// Elle propose un lab à construire, sur l'écran Nouveau lab.
    Proposition,
    /// Elle parle d'elle-même, en une phrase. Ses outils restent décrits :
    /// Anthropic refuse une conversation qui a appelé des outils sans qu'ils
    /// soient décrits. Mais l'app refuse chaque appel.
    Parole,
}

/// Ce que l'IA a le droit de faire dans le lab sans demander.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Liberte {
    Libre,
    #[default]
    AvecAccord,
}

/// Ce qu'a répondu l'utilisateur, pour le journal.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Accord {
    Donne,
    Refuse,
    /// Elle est libre dans le lab : elle n'a pas demandé.
    SansDemander,
}

/// Ce que l'IA a fait, pour son journal.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    /// Une machine, un lab, ou sa mémoire.
    pub lieu: String,
    pub quoi: String,
    #[serde(default)]
    pub commande: Option<String>,
    #[serde(default)]
    pub accord: Option<Accord>,
}

/// Une action que l'IA veut faire, montrée à l'utilisateur avant.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Demande {
    pub id: String,
    pub lieu: String,
    pub titre: String,
    pub detail: String,
}

/// Une commande de l'IA s'arrête au bout de ce temps, dans la machine.
const DELAI_COMMANDE: u64 = 30;
/// Ce que l'IA lit d'une sortie, au plus.
const SORTIE_MAX: usize = 6000;
/// Ce que l'écran montre d'un résultat.
const APERCU_MAX: usize = 1500;

pub fn definitions(mode: Mode) -> Vec<Outil> {
    if mode == Mode::Proposition {
        return vec![proposer_un_lab()];
    }
    let machine = json!({"type": "string", "description": "Le nom de la machine, par exemple pc1"});
    let mut outils = vec![
        Outil {
            nom: "etat_du_lab".into(),
            description: "Décrit le lab : ses machines, si elles tournent, leurs interfaces, \
                          leurs adresses et leurs câbles."
                .into(),
            parametres: json!({"type": "object", "properties": {}}),
        },
        Outil {
            nom: "lancer_commande".into(),
            description: format!(
                "Lance une commande shell (sh) en root dans une machine du lab qui tourne, et rend \
                 son code de sortie et ce qu'elle a écrit. La commande est coupée au bout de \
                 {DELAI_COMMANDE} secondes."
            ),
            parametres: json!({
                "type": "object",
                "properties": {"machine": machine, "commande": {"type": "string", "description": "La commande, par exemple ping -c 3 10.0.2.10"}},
                "required": ["machine", "commande"],
            }),
        },
        Outil {
            nom: "demarrer_machine".into(),
            description: "Démarre une machine du lab.".into(),
            parametres: json!({"type": "object", "properties": {"machine": machine}, "required": ["machine"]}),
        },
        Outil {
            nom: "arreter_machine".into(),
            description: "Arrête une machine du lab.".into(),
            parametres: json!({"type": "object", "properties": {"machine": machine}, "required": ["machine"]}),
        },
    ];
    if matches!(mode, Mode::LabEtMemoire | Mode::Parole) {
        outils.push(retenir());
    }
    if mode == Mode::Invention {
        let commande = json!({
            "type": "object",
            "properties": {"machine": {"type": "string"}, "commande": {"type": "string"}},
            "required": ["machine", "commande"],
        });
        outils.push(Outil {
            nom: "poser_une_panne".into(),
            description: "Pose une panne dans le lab. L'app sauvegarde le lab, lance les commandes de \
                          pose, puis les tests : au moins un test doit échouer, sinon la panne n'a pas \
                          pris et le lab revient comme avant."
                .into(),
            parametres: json!({
                "type": "object",
                "properties": {
                    "titre": {"type": "string", "description": "Ce que l'utilisateur va constater, jamais la cause. Par exemple : pc1 ne joint plus pc2."},
                    "pose": {"type": "array", "items": commande, "description": "Les commandes qui cassent quelque chose, dans l'ordre. Préfère une panne qui tient après un redémarrage."},
                    "tests": {"type": "array", "items": {
                        "type": "object",
                        "properties": {"nom": {"type": "string"}, "machine": {"type": "string"}, "commande": {"type": "string"}},
                        "required": ["nom", "machine", "commande"],
                    }, "description": "Des commandes qui réussissent (code 0) quand tout marche, et dont au moins une échoue tant que la panne est là. Donne une fin à chaque commande (ping -c 2 -W 1)."},
                    "indices": {"type": "array", "items": {"type": "string"}, "description": "Trois indices, du plus vague au plus précis, sans jamais donner la cause."},
                    "explication": {"type": "string", "description": "La cause, et comment la réparer."},
                },
                "required": ["titre", "pose", "tests", "indices", "explication"],
            }),
        });
    }
    outils
}

fn retenir() -> Outil {
    Outil {
        nom: "retenir".into(),
        description: "Garde une ligne dans ta mémoire, pour les prochaines sessions. \
                      L'utilisateur la voit et peut la corriger."
            .into(),
        parametres: json!({
            "type": "object",
            "properties": {
                "liste": {"type": "string", "enum": ["toi", "notes"], "description": "toi : ce que tu sais de l'utilisateur. notes : le reste, son PC, ses labs, ce qui s'est passé."},
                "texte": {"type": "string", "description": "Une phrase courte, qui servira la prochaine fois. Par exemple : Tu prépares le titre AIS."},
                "remplace": {"type": "string", "description": "Une ligne de ta mémoire qui ne sert plus, recopiée exactement : elle est remplacée par texte."},
            },
            "required": ["liste", "texte"],
        }),
    }
}

fn proposer_un_lab() -> Outil {
    let bout = json!({
        "type": "object",
        "properties": {
            "machine": {"type": "string"},
            "adresse": {"type": "string", "description": "L'adresse de ce bout du câble avec son masque, par exemple 10.0.1.10/24. Rien pour un switch."},
        },
        "required": ["machine"],
    });
    Outil {
        nom: "proposer_un_lab".into(),
        description: format!(
            "Propose un lab à l'utilisateur, qui le verra en schéma avant de le lancer. L'app le \
             vérifie : un refus dit quoi corriger, et tu proposes à nouveau. Les interfaces se \
             nomment toutes seules, eth0, eth1…, dans l'ordre des câbles. {MACHINES_MAX} machines \
             au plus."
        ),
        parametres: json!({
            "type": "object",
            "properties": {
                "nom": {"type": "string", "description": "Un nom court pour le lab, par exemple Deux réseaux et un routeur"},
                "commentaire": {"type": "string", "description": "Une ou deux phrases pour l'utilisateur : ce que tu as choisi, et ce qu'il peut te demander de changer."},
                "machines": {"type": "array", "items": {
                    "type": "object",
                    "properties": {
                        "nom": {"type": "string", "description": "Des minuscules, des chiffres et des tirets, 15 au plus : pc1, srv-web, r1, sw1"},
                        "sorte": {"type": "string", "enum": ["poste", "serveur", "routeur", "switch"]},
                        "passerelle": {"type": "string", "description": "La passerelle par défaut : l'adresse d'un routeur branché dans son réseau, par exemple 10.0.1.1"},
                        "routes": {"type": "array", "items": {
                            "type": "object",
                            "properties": {
                                "destination": {"type": "string", "description": "Un réseau, par exemple 10.0.3.0/24"},
                                "passerelle": {"type": "string", "description": "L'adresse d'un routeur branché dans un réseau de cette machine"},
                            },
                            "required": ["destination", "passerelle"],
                        }},
                    },
                    "required": ["nom", "sorte"],
                }},
                "cables": {"type": "array", "items": {
                    "type": "object",
                    "properties": {"de": bout, "vers": bout},
                    "required": ["de", "vers"],
                }},
            },
            "required": ["nom", "commentaire", "machines", "cables"],
        }),
    }
}

/// Le lab tel que l'IA le lit : ses machines, leur état et leurs câbles.
pub fn description_du_lab(le_lab: &Lab) -> String {
    let etat = lab::etat(le_lab).ok();
    let mut texte = format!("Lab « {} ».\n", le_lab.nom);
    for machine in &le_lab.machines {
        let marche = etat
            .as_ref()
            .and_then(|e| e.machines.iter().find(|m| m.nom == machine.nom))
            .map(|m| m.marche);
        let dit = match marche {
            Some(Marche::EnMarche) => "en marche",
            Some(Marche::Arretee) => "arrêtée",
            Some(Marche::Absente) | None => "pas encore créée",
        };
        texte.push_str(&format!(
            "- {} ({}), {dit}",
            machine.nom,
            machine.sorte.mot()
        ));
        for interface in &machine.interfaces {
            texte.push_str(&format!(
                ", {} {}",
                interface.nom,
                interface.adresse.as_deref().unwrap_or("sans adresse")
            ));
            if let Some(passerelle) = &interface.passerelle {
                texte.push_str(&format!(" passerelle {passerelle}"));
            }
        }
        texte.push('\n');
    }
    for cable in &le_lab.cables {
        texte.push_str(&format!(
            "- câble : {} {} vers {} {}{}\n",
            cable.de.machine,
            cable.de.interface,
            cable.vers.machine,
            cable.vers.interface,
            cable
                .reseau
                .as_deref()
                .map(|r| format!(", réseau {r}"))
                .unwrap_or_default()
        ));
    }
    texte
}

/// Pose une panne inventée par l'IA, et rend ce qu'elle lira.
pub type Inventer = Arc<dyn Fn(Panne) -> Result<String, String> + Send + Sync>;

/// Reçoit le lab proposé et ce que l'IA en dit à l'utilisateur.
pub type Proposer = Arc<dyn Fn(Proposition, String) -> Result<String, String> + Send + Sync>;

/// Garde une ligne dans la mémoire de l'IA, ou en remplace une, et rend ce
/// qu'elle lira.
pub type Memoriser = Arc<dyn Fn(Liste, &str, Option<&str>) -> Result<String, String> + Send + Sync>;

/// Écrit dans le journal ce que l'IA a fait.
pub type Noter = Arc<dyn Fn(Action) + Send + Sync>;

/// Ce qu'il faut pour exécuter les outils : le lab, le réglage, et de quoi
/// demander l'accord de l'utilisateur.
#[derive(Clone)]
pub struct Execution {
    pub lab: Lab,
    pub liberte: Liberte,
    /// Montre la demande et attend la réponse de l'utilisateur.
    pub accorder: Arc<dyn Fn(&Demande) -> bool + Send + Sync>,
    /// Pendant que l'IA invente une panne : elle a l'outil pour la poser,
    /// et rien de ce qu'elle fait ne s'affiche, pour ne pas donner la
    /// réponse. L'utilisateur a demandé la panne : il n'y a pas d'accord à
    /// demander.
    pub inventer: Option<Inventer>,
    /// Sur l'écran Nouveau lab : elle n'a que l'outil pour proposer un lab,
    /// et rien ne s'affiche dans le panneau.
    pub proposer: Option<Proposer>,
    /// Sa mémoire, quand l'utilisateur ne l'a pas coupée.
    pub memoriser: Option<Memoriser>,
    pub noter: Option<Noter>,
    /// Elle parle d'elle-même : pas d'outil.
    pub parole: bool,
}

impl Execution {
    /// Ce qu'il faut pour agir dans `lab`, sans mémoire ni journal.
    pub fn dans(
        lab: Lab,
        liberte: Liberte,
        accorder: Arc<dyn Fn(&Demande) -> bool + Send + Sync>,
    ) -> Execution {
        Execution {
            lab,
            liberte,
            accorder,
            inventer: None,
            proposer: None,
            memoriser: None,
            noter: None,
            parole: false,
        }
    }

    pub fn mode(&self) -> Mode {
        if self.proposer.is_some() {
            Mode::Proposition
        } else if self.inventer.is_some() {
            Mode::Invention
        } else if self.parole {
            Mode::Parole
        } else if self.memoriser.is_some() {
            Mode::LabEtMemoire
        } else {
            Mode::Lab
        }
    }

    fn noter(&self, action: Action) {
        if let Some(noter) = &self.noter {
            noter(action);
        }
    }

    /// Exécute un appel d'outil, après l'accord s'il le faut, et rend ce que
    /// l'IA lira.
    pub fn executer(&self, appel: &Appel, signaler: &mut dyn FnMut(Evenement)) -> String {
        let discret = matches!(self.mode(), Mode::Invention | Mode::Proposition);
        if appel.nom == "poser_une_panne" {
            return self.poser_une_panne(&appel.arguments);
        }
        if appel.nom == "proposer_un_lab" {
            return self.proposer_un_lab(&appel.arguments);
        }
        match self.mode() {
            Mode::Proposition => return format!("L'outil {} n'existe pas ici.", appel.nom),
            Mode::Parole => return "Tu n'as pas d'outil ici : réponds en une phrase.".into(),
            Mode::Lab | Mode::LabEtMemoire | Mode::Invention => {}
        }
        if appel.nom == "retenir" {
            return self.retenir(appel, signaler);
        }
        let mut rien = |_: Evenement| {};
        let signaler: &mut dyn FnMut(Evenement) = if discret { &mut rien } else { signaler };
        let texte = |nom: &str| {
            appel.arguments[nom]
                .as_str()
                .unwrap_or_default()
                .to_string()
        };
        let machine = texte("machine");
        let (titre, detail) = match appel.nom.as_str() {
            "etat_du_lab" => ("Lire l'état du lab".to_string(), String::new()),
            "lancer_commande" => (format!("Commande sur {machine}"), texte("commande")),
            "demarrer_machine" => (format!("Démarrer {machine}"), String::new()),
            "arreter_machine" => (format!("Arrêter {machine}"), String::new()),
            autre => {
                return format!("L'outil {autre} n'existe pas.");
            }
        };
        let agit = appel.nom != "etat_du_lab";
        if agit && !machine.is_empty() && !self.lab.machines.iter().any(|m| m.nom == machine) {
            return format!("La machine {machine} n'existe pas dans ce lab.");
        }
        signaler(Evenement::Outil {
            id: appel.id.clone(),
            titre: titre.clone(),
            detail: detail.clone(),
        });
        // Ce qu'elle fait en inventant une panne ne s'écrit pas : le journal
        // donnerait la réponse.
        let action = |accord| Action {
            lieu: machine.clone(),
            quoi: match appel.nom.as_str() {
                "demarrer_machine" => "Démarrer la machine".into(),
                "arreter_machine" => "Arrêter la machine".into(),
                _ => "Commande".into(),
            },
            commande: (appel.nom == "lancer_commande").then(|| texte("commande")),
            accord: Some(accord),
        };
        if agit && self.liberte == Liberte::AvecAccord && !discret {
            let demande = Demande {
                id: appel.id.clone(),
                lieu: "Dans le lab".into(),
                titre,
                detail,
            };
            if !(self.accorder)(&demande) {
                self.noter(action(Accord::Refuse));
                signaler(Evenement::Refus {
                    id: appel.id.clone(),
                });
                return "L'utilisateur a refusé cette action.".into();
            }
            self.noter(action(Accord::Donne));
        } else if agit && !discret {
            self.noter(action(Accord::SansDemander));
        }
        let resultat = match appel.nom.as_str() {
            "etat_du_lab" => Ok(description_du_lab(&self.lab)),
            "lancer_commande" => self.commande(&machine, &texte("commande")),
            "demarrer_machine" => lab::demarrer_machine(&self.lab, &machine, &mut |_| {})
                .map(|_| format!("{machine} démarre.")),
            "arreter_machine" => {
                lab::arreter_machine(&self.lab, &machine).map(|_| format!("{machine} est arrêtée."))
            }
            _ => unreachable!(),
        };
        let (ok, contenu) = match resultat {
            Ok(contenu) => (true, contenu),
            Err(erreur) => (false, erreur),
        };
        signaler(Evenement::Resultat {
            id: appel.id.clone(),
            ok,
            apercu: borner(&contenu, APERCU_MAX),
        });
        contenu
    }

    fn retenir(&self, appel: &Appel, signaler: &mut dyn FnMut(Evenement)) -> String {
        let Some(memoriser) = &self.memoriser else {
            return "L'utilisateur a coupé ta mémoire : ne retiens rien.".into();
        };
        let liste = match appel.arguments["liste"].as_str() {
            Some("toi") => Liste::Toi,
            Some("notes") => Liste::Notes,
            _ => return "liste vaut toi ou notes.".into(),
        };
        let texte = appel.arguments["texte"].as_str().unwrap_or_default();
        let remplace = appel.arguments["remplace"]
            .as_str()
            .filter(|r| !r.trim().is_empty());
        signaler(Evenement::Outil {
            id: appel.id.clone(),
            titre: "Retenir".into(),
            detail: texte.trim().to_string(),
        });
        let resultat = memoriser(liste, texte, remplace);
        if resultat.is_ok() {
            self.noter(Action {
                lieu: "sa mémoire".into(),
                quoi: format!("Retenu : « {} »", texte.trim().trim_end_matches('.')),
                commande: None,
                accord: None,
            });
        }
        let (ok, contenu) = match resultat {
            Ok(contenu) => (true, contenu),
            Err(erreur) => (false, erreur),
        };
        signaler(Evenement::Resultat {
            id: appel.id.clone(),
            ok,
            apercu: contenu.clone(),
        });
        contenu
    }

    fn poser_une_panne(&self, arguments: &Value) -> String {
        let Some(inventer) = &self.inventer else {
            return "Tu ne peux poser une panne que quand l'utilisateur en demande une.".into();
        };
        let mut panne: Panne = match serde_json::from_value(json!({
            "id": "inventee",
            "titre": arguments["titre"],
            "pose": arguments["pose"],
            "tests": arguments["tests"],
            "indices": arguments.get("indices").cloned().unwrap_or(json!([])),
            "explication": arguments["explication"],
        })) {
            Ok(panne) => panne,
            Err(e) => return format!("La panne est mal décrite : {e}. Recommence."),
        };
        panne.inventee = true;
        match inventer(panne) {
            Ok(message) => message,
            Err(erreur) => format!("{erreur} Essaie une autre panne."),
        }
    }

    fn proposer_un_lab(&self, arguments: &Value) -> String {
        let Some(proposer) = &self.proposer else {
            return "Tu ne peux proposer un lab que sur l'écran Nouveau lab.".into();
        };
        let proposition: Proposition = match serde_json::from_value(arguments.clone()) {
            Ok(proposition) => proposition,
            Err(e) => return format!("Le lab est mal décrit : {e}. Recommence."),
        };
        let commentaire = arguments["commentaire"]
            .as_str()
            .unwrap_or_default()
            .trim()
            .to_string();
        match proposer(proposition, commentaire) {
            Ok(message) => message,
            Err(erreur) => format!("{erreur} Corrige, puis propose à nouveau."),
        }
    }

    fn commande(&self, machine: &str, commande: &str) -> Result<String, String> {
        if commande.trim().is_empty() {
            return Err("La commande est vide.".into());
        }
        let (code, sortie) = lab::commande_dans(&self.lab, machine, commande, DELAI_COMMANDE)?;
        let sortie = borner(sortie.trim_end(), SORTIE_MAX);
        Ok(match code {
            Some(124) | None => {
                format!("Coupée au bout de {DELAI_COMMANDE} secondes.\n{sortie}")
            }
            Some(code) => format!("Code de sortie : {code}\n{sortie}"),
        })
    }
}

/// Garde le début et la fin d'un texte trop long.
pub(crate) fn borner(texte: &str, max: usize) -> String {
    let nombre = texte.chars().count();
    if nombre <= max {
        return texte.to_string();
    }
    let debut: String = texte.chars().take(max / 3).collect();
    let fin: String = texte.chars().skip(nombre - max * 2 / 3).collect();
    format!(
        "{debut}\n[… {} caractères coupés …]\n{fin}",
        nombre - debut.chars().count() - fin.chars().count()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn execution(liberte: Liberte, oui: bool) -> Execution {
        Execution::dans(lab::premier_lab(), liberte, Arc::new(move |_| oui))
    }

    fn journal(execution: &mut Execution) -> Arc<std::sync::Mutex<Vec<Action>>> {
        let ecrit: Arc<std::sync::Mutex<Vec<Action>>> = Arc::default();
        let garde = ecrit.clone();
        execution.noter = Some(Arc::new(move |action| garde.lock().unwrap().push(action)));
        ecrit
    }

    #[test]
    fn un_refus_s_ecrit_dans_le_journal() {
        let mut execution = execution(Liberte::AvecAccord, false);
        let ecrit = journal(&mut execution);
        let appel = Appel {
            id: "a1".into(),
            nom: "lancer_commande".into(),
            arguments: json!({"machine": "pc1", "commande": "ip route"}),
        };
        execution.executer(&appel, &mut |_| {});
        assert_eq!(
            ecrit.lock().unwrap().as_slice(),
            [Action {
                lieu: "pc1".into(),
                quoi: "Commande".into(),
                commande: Some("ip route".into()),
                accord: Some(Accord::Refuse),
            }]
        );
    }

    #[test]
    fn ce_qu_elle_fait_en_inventant_ne_s_ecrit_pas() {
        let mut execution = execution(Liberte::AvecAccord, false);
        execution.inventer = Some(Arc::new(|_| Ok(String::new())));
        let ecrit = journal(&mut execution);
        let appel = Appel {
            id: "a".into(),
            nom: "lancer_commande".into(),
            arguments: json!({"machine": "inconnue", "commande": "ls"}),
        };
        execution.executer(&appel, &mut |_| {});
        assert!(ecrit.lock().unwrap().is_empty());
    }

    #[test]
    fn elle_retient_seulement_quand_sa_memoire_est_branchee() {
        use crate::ia::memoire::Memoire;
        let appel = Appel {
            id: "m".into(),
            nom: "retenir".into(),
            arguments: json!({"liste": "toi", "texte": "Tu prépares le titre AIS."}),
        };
        let sans = execution(Liberte::AvecAccord, true);
        assert_eq!(sans.mode(), Mode::Lab);
        assert!(definitions(Mode::Lab).iter().all(|o| o.nom != "retenir"));
        assert!(
            sans.executer(&appel, &mut |_| {})
                .contains("coupé ta mémoire")
        );

        let memoire: Arc<std::sync::Mutex<Memoire>> = Arc::default();
        let garde = memoire.clone();
        let mut avec = execution(Liberte::AvecAccord, true);
        avec.memoriser = Some(Arc::new(move |liste, texte, remplace| {
            garde.lock().unwrap().retenir(liste, texte, remplace)
        }));
        let ecrit = journal(&mut avec);
        assert_eq!(avec.mode(), Mode::LabEtMemoire);
        assert!(
            definitions(Mode::LabEtMemoire)
                .iter()
                .any(|o| o.nom == "retenir")
        );
        let mut vus = Vec::new();
        assert_eq!(avec.executer(&appel, &mut |e| vus.push(e)), "Retenu.");
        assert_eq!(memoire.lock().unwrap().toi, ["Tu prépares le titre AIS."]);
        assert!(matches!(&vus[1], Evenement::Resultat { ok: true, .. }));
        assert_eq!(ecrit.lock().unwrap()[0].lieu, "sa mémoire");
    }

    #[test]
    fn quand_elle_parle_d_elle_meme_elle_n_a_pas_d_outil() {
        let mut execution = execution(Liberte::Libre, true);
        execution.parole = true;
        assert_eq!(execution.mode(), Mode::Parole);
        // Décrits pour que l'historique reste valable, refusés à l'appel.
        assert!(
            definitions(Mode::Parole)
                .iter()
                .any(|o| o.nom == "lancer_commande")
        );
        let appel = Appel {
            id: "p".into(),
            nom: "arreter_machine".into(),
            arguments: json!({"machine": "pc1"}),
        };
        assert!(
            execution
                .executer(&appel, &mut |_| {})
                .contains("pas d'outil")
        );
    }

    #[test]
    fn borne_les_sorties_trop_longues() {
        assert_eq!(borner("court", 10), "court");
        let long = "a".repeat(100);
        let borne = borner(&long, 30);
        assert!(borne.starts_with(&"a".repeat(10)));
        assert!(borne.contains("[… 70 caractères coupés …]"));
    }

    #[test]
    fn un_refus_n_execute_rien() {
        let appel = Appel {
            id: "a1".into(),
            nom: "arreter_machine".into(),
            arguments: json!({"machine": "pc1"}),
        };
        let mut vus = Vec::new();
        let rendu = execution(Liberte::AvecAccord, false).executer(&appel, &mut |e| vus.push(e));
        assert_eq!(rendu, "L'utilisateur a refusé cette action.");
        assert!(matches!(vus[0], Evenement::Outil { .. }));
        assert_eq!(vus[1], Evenement::Refus { id: "a1".into() });
    }

    #[test]
    fn refuse_ce_qui_n_existe_pas() {
        let mut rien = |_| {};
        let inconnu = Appel {
            id: "a".into(),
            nom: "effacer_le_disque".into(),
            arguments: json!({}),
        };
        assert!(
            execution(Liberte::Libre, true)
                .executer(&inconnu, &mut rien)
                .contains("n'existe pas")
        );
        let ailleurs = Appel {
            id: "b".into(),
            nom: "lancer_commande".into(),
            arguments: json!({"machine": "serveur-inconnu", "commande": "ls"}),
        };
        assert_eq!(
            execution(Liberte::Libre, true).executer(&ailleurs, &mut rien),
            "La machine serveur-inconnu n'existe pas dans ce lab."
        );
    }

    #[test]
    fn les_outils_ont_un_schema() {
        for mode in [
            Mode::Lab,
            Mode::LabEtMemoire,
            Mode::Invention,
            Mode::Proposition,
        ] {
            for outil in definitions(mode) {
                assert_eq!(outil.parametres["type"], "object", "{}", outil.nom);
            }
        }
    }

    #[test]
    fn la_proposition_passe_au_lab_construit() {
        let recu: Arc<std::sync::Mutex<Option<(Proposition, String)>>> = Arc::default();
        let garde = recu.clone();
        let mut execution = execution(Liberte::AvecAccord, false);
        execution.proposer = Some(Arc::new(move |proposition, commentaire| {
            *garde.lock().unwrap() = Some((proposition, commentaire));
            Ok("proposé".into())
        }));
        assert_eq!(execution.mode(), Mode::Proposition);
        let appel = Appel {
            id: "p".into(),
            nom: "proposer_un_lab".into(),
            arguments: json!({
                "nom": "Un poste",
                "commentaire": "Un seul poste, sans réseau.",
                "machines": [{"nom": "pc1", "sorte": "poste"}],
                "cables": [],
            }),
        };
        let mut rien = |_: Evenement| {};
        assert_eq!(execution.executer(&appel, &mut rien), "proposé");
        let (proposition, commentaire) = recu.lock().unwrap().take().unwrap();
        assert_eq!(proposition.machines[0].nom, "pc1");
        assert_eq!(commentaire, "Un seul poste, sans réseau.");
        let lancer = Appel {
            id: "q".into(),
            nom: "lancer_commande".into(),
            arguments: json!({"machine": "pc1", "commande": "ls"}),
        };
        assert!(
            execution
                .executer(&lancer, &mut rien)
                .contains("n'existe pas ici")
        );
    }
}
