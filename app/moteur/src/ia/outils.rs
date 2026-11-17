//! Les outils de l'IA dans le lab, et l'accord de l'utilisateur.
//!
//! C'est l'app qui applique le réglage d'accord, jamais la consigne de
//! l'IA : une IA peut l'oublier, ou se le faire retirer par un texte piégé
//! (`docs/architecture.md`, partie 3).

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{Appel, Evenement};
use crate::lab::{self, Lab, Marche, Sorte};
use crate::pannes::Panne;

#[derive(Clone)]
pub struct Outil {
    pub nom: String,
    pub description: String,
    pub parametres: Value,
}

/// Ce que l'IA a le droit de faire dans le lab sans demander.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Liberte {
    Libre,
    #[default]
    AvecAccord,
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

pub fn definitions(invention: bool) -> Vec<Outil> {
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
    if invention {
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

fn nom_de_sorte(sorte: Sorte) -> &'static str {
    match sorte {
        Sorte::Poste => "poste",
        Sorte::Serveur => "serveur",
        Sorte::Routeur => "routeur",
        Sorte::Switch => "switch",
        Sorte::ParFeu => "pare-feu",
        Sorte::Stockage => "stockage",
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
            nom_de_sorte(machine.sorte)
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
}

impl Execution {
    /// Exécute un appel d'outil, après l'accord s'il le faut, et rend ce que
    /// l'IA lira.
    pub fn executer(&self, appel: &Appel, signaler: &mut dyn FnMut(Evenement)) -> String {
        let discret = self.inventer.is_some();
        if appel.nom == "poser_une_panne" {
            return self.poser_une_panne(&appel.arguments);
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
        if agit && self.liberte == Liberte::AvecAccord && !discret {
            let demande = Demande {
                id: appel.id.clone(),
                lieu: "Dans le lab".into(),
                titre,
                detail,
            };
            if !(self.accorder)(&demande) {
                signaler(Evenement::Refus {
                    id: appel.id.clone(),
                });
                return "L'utilisateur a refusé cette action.".into();
            }
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
        Execution {
            lab: lab::premier_lab(),
            liberte,
            accorder: Arc::new(move |_| oui),
            inventer: None,
        }
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
        for outil in definitions(true) {
            assert_eq!(outil.parametres["type"], "object", "{}", outil.nom);
        }
    }
}
