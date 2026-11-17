//! Les pannes. Celles du catalogue sont écrites à l'avance, une liste par
//! lab (`moteur/pannes/<lab>.json`). L'IA peut aussi en inventer, dans le
//! même format. Une panne se pose par des commandes, se vérifie par des
//! tests, et se défait par l'instantané du lab pris juste avant.

use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::ia::outils::borner;
use crate::incus::incus;
use crate::lab::{self, Lab};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Difficulte {
    #[default]
    Facile,
    Moyen,
    Difficile,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Commande {
    pub machine: String,
    pub commande: String,
}

/// Un test réussit quand sa commande rend 0.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Test {
    pub nom: String,
    pub machine: String,
    pub commande: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Panne {
    pub id: String,
    /// Ce qui ne marche plus, jamais la cause.
    pub titre: String,
    #[serde(default)]
    pub famille: String,
    #[serde(default)]
    pub niveau: Difficulte,
    pub pose: Vec<Commande>,
    pub tests: Vec<Test>,
    #[serde(default)]
    pub indices: Vec<String>,
    pub explication: String,
    #[serde(default)]
    pub inventee: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResultatTest {
    pub nom: String,
    pub machine: String,
    pub commande: String,
    pub reussi: bool,
    pub sortie: String,
}

const INSTANTANE: &str = "avant-panne";
/// Un test, ou une commande de pose, ne dure pas plus.
const DELAI: u64 = 30;

/// Les pannes écrites à l'avance pour ce lab.
pub fn catalogue(lab: &Lab) -> Vec<Panne> {
    let texte = match lab.id.as_str() {
        "premier-lab" => include_str!("../pannes/premier-lab.json"),
        _ => "[]",
    };
    serde_json::from_str(texte).unwrap_or_default()
}

/// Pose la panne : le lab est sauvegardé, la panne posée, puis ses tests
/// lancés. Si aucun n'échoue, elle n'a pas pris, et le lab revient comme
/// avant.
pub fn poser(lab: &Lab, panne: &Panne, avance: &mut dyn FnMut(&str)) -> Result<(), String> {
    for test in &panne.tests {
        if !lab.machines.iter().any(|m| m.nom == test.machine) {
            return Err(format!(
                "Un test vise {}, qui n'est pas dans ce lab.",
                test.machine
            ));
        }
    }
    if panne.tests.is_empty() {
        return Err("Une panne sans test ne peut pas se vérifier.".into());
    }
    lab::lancer_et_attendre(lab, avance)?;
    avance("Sauvegarde du lab");
    sauvegarder(lab)?;
    avance("Pose de la panne");
    for commande in &panne.pose {
        let resultat = lab::commande_dans(lab, &commande.machine, &commande.commande, DELAI);
        let rate = match &resultat {
            Ok((Some(0), _)) => None,
            Ok((_, sortie)) => Some(sortie.trim().to_string()),
            Err(erreur) => Some(erreur.clone()),
        };
        if let Some(raison) = rate {
            revenir(lab)?;
            return Err(format!(
                "La panne ne s'est pas posée sur {} : {}. Le lab est revenu comme avant.",
                commande.machine,
                borner(&raison, 300)
            ));
        }
    }
    // Le temps que systemd-networkd et le noyau appliquent le changement.
    thread::sleep(Duration::from_secs(2));
    if verifier(lab, panne).iter().all(|r| r.reussi) {
        revenir(lab)?;
        return Err(
            "La panne n'a pas pris : tous ses tests passent encore. Le lab est revenu comme avant."
                .into(),
        );
    }
    Ok(())
}

/// Lance les tests de la panne, et dit lesquels passent.
pub fn verifier(lab: &Lab, panne: &Panne) -> Vec<ResultatTest> {
    panne
        .tests
        .iter()
        .map(|test| {
            let (reussi, sortie) =
                match lab::commande_dans(lab, &test.machine, &test.commande, DELAI) {
                    Ok((code, sortie)) => (code == Some(0), sortie),
                    Err(erreur) => (false, erreur),
                };
            ResultatTest {
                nom: test.nom.clone(),
                machine: test.machine.clone(),
                commande: test.commande.clone(),
                reussi,
                sortie: borner(sortie.trim(), 600),
            }
        })
        .collect()
}

fn machines(lab: &Lab) -> Vec<String> {
    lab::etat(lab)
        .map(|e| {
            e.machines
                .into_iter()
                .filter(|m| m.marche != lab::Marche::Absente)
                .map(|m| m.nom)
                .collect()
        })
        .unwrap_or_default()
}

fn sauvegarder(lab: &Lab) -> Result<(), String> {
    let projet = lab::projet(lab);
    for machine in machines(lab) {
        let _ = incus(&[
            "snapshot",
            "delete",
            &machine,
            INSTANTANE,
            "--project",
            &projet,
        ]);
        incus(&[
            "snapshot",
            "create",
            &machine,
            INSTANTANE,
            "--project",
            &projet,
        ])
        .map_err(|e| format!("Le lab ne se sauvegarde pas : {e}"))?;
    }
    Ok(())
}

/// Remet le lab comme juste avant la panne.
pub fn revenir(lab: &Lab) -> Result<(), String> {
    let projet = lab::projet(lab);
    for machine in machines(lab) {
        incus(&[
            "snapshot",
            "restore",
            &machine,
            INSTANTANE,
            "--project",
            &projet,
        ])
        .map_err(|e| format!("{machine} ne revient pas en arrière : {e}"))?;
    }
    oublier(lab);
    Ok(())
}

/// La panne est finie : la sauvegarde ne sert plus.
pub fn oublier(lab: &Lab) {
    let projet = lab::projet(lab);
    for machine in machines(lab) {
        let _ = incus(&[
            "snapshot",
            "delete",
            &machine,
            INSTANTANE,
            "--project",
            &projet,
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_catalogue_du_premier_lab_tient_debout() {
        let lab = lab::premier_lab();
        let pannes = catalogue(&lab);
        assert_eq!(pannes.len(), 5);
        for panne in &pannes {
            assert!(!panne.tests.is_empty(), "{}", panne.id);
            assert!(!panne.indices.is_empty(), "{}", panne.id);
            assert!(!panne.explication.is_empty(), "{}", panne.id);
            for machine in panne
                .pose
                .iter()
                .map(|c| &c.machine)
                .chain(panne.tests.iter().map(|t| &t.machine))
            {
                assert!(
                    lab.machines.iter().any(|m| &m.nom == machine),
                    "{} vise {machine}",
                    panne.id
                );
            }
            // Le titre dit ce qui ne marche plus, jamais la cause.
            for mot in ["ip_forward", "passerelle", "masque", "eth1", "adresse"] {
                assert!(!panne.titre.contains(mot), "{} dit sa cause", panne.id);
            }
        }
    }

    #[test]
    fn une_panne_inventee_se_lit_sans_les_champs_facultatifs() {
        let panne: Panne = serde_json::from_str(
            r#"{"id": "x", "titre": "t", "pose": [], "explication": "e",
                "tests": [{"nom": "n", "machine": "pc1", "commande": "true"}]}"#,
        )
        .unwrap();
        assert_eq!(panne.niveau, Difficulte::Facile);
        assert!(panne.indices.is_empty());
    }
}
