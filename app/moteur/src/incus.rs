//! Incus fait tourner les machines du lab. L'app le commande par sa ligne de
//! commande, et par son API (`incus query`) quand il faut suivre une
//! opération.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::systeme;

/// La place des machines de l'app sur le disque : un dossier, rien à régler.
pub const POOL: &str = "opn-display";
/// L'image de toutes les machines Linux du lab, rangée dans Incus sous ce nom.
pub const IMAGE: &str = "opn-display-debian-13";
/// Le serveur d'images d'Incus, et son image de Debian 13.
const SERVEUR_IMAGES: &str = "https://images.linuxcontainers.org";
const DEBIAN: &str = "debian/13";

pub(crate) fn incus(args: &[&str]) -> Result<String, String> {
    systeme::lancer("incus", args)
}

pub(crate) fn incus_avec(args: &[&str], entree: &[u8]) -> Result<String, String> {
    systeme::lancer_avec("incus", args, Some(entree))
}

/// Ce que l'app trouve d'Incus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Etat {
    Absent,
    /// Installé, mais il ne répond pas, avec ce qu'il a dit.
    Muet(String),
    /// Il répond ; il manque la place des machines ou leur image.
    APreparer {
        version: String,
    },
    Pret {
        version: String,
    },
}

pub(crate) fn etat() -> Etat {
    if systeme::lancer("sh", &["-c", "command -v incus"]).is_err() {
        return Etat::Absent;
    }
    let version = match incus(&["query", "/1.0"]) {
        Ok(serveur) => version(&serveur),
        Err(erreur) => return Etat::Muet(erreur),
    };
    if pool_present() && image_presente() {
        Etat::Pret { version }
    } else {
        Etat::APreparer { version }
    }
}

/// La version d'Incus, lue dans la réponse de `/1.0`.
fn version(serveur: &str) -> String {
    serde_json::from_str::<Value>(serveur)
        .ok()
        .and_then(|v| {
            v["environment"]["server_version"]
                .as_str()
                .map(str::to_string)
        })
        .unwrap_or_default()
}

fn pool_present() -> bool {
    incus(&["storage", "show", POOL]).is_ok()
}

pub(crate) fn image_presente() -> bool {
    incus(&["query", &format!("/1.0/images/aliases/{IMAGE}")]).is_ok()
}

/// Attend qu'Incus réponde, le temps qu'il démarre.
pub(crate) fn attendre(secondes: u64) -> Result<String, String> {
    let mut derniere = String::new();
    for _ in 0..secondes {
        match incus(&["query", "/1.0"]) {
            Ok(serveur) => return Ok(version(&serveur)),
            Err(erreur) => derniere = erreur,
        }
        thread::sleep(Duration::from_secs(1));
    }
    Err(format!("Incus ne répond pas : {derniere}"))
}

/// Pose la place des machines, si elle manque.
pub(crate) fn preparer_pool() -> Result<(), String> {
    if !pool_present() {
        incus(&["storage", "create", POOL, "dir"])?;
    }
    Ok(())
}

/// Télécharge l'image Debian 13 des machines, en disant où en est le
/// téléchargement, de 0 à 1. Rend sa taille en octets, ou rien si
/// l'utilisateur a arrêté.
pub(crate) fn telecharger_image(
    arret: &AtomicBool,
    progres: &mut dyn FnMut(Option<f64>),
) -> Result<Option<u64>, String> {
    let demande = json!({
        "source": {
            "type": "image",
            "mode": "pull",
            "server": SERVEUR_IMAGES,
            "protocol": "simplestreams",
            "alias": DEBIAN,
        },
        "aliases": [{"name": IMAGE, "description": "Debian 13, pour les machines d'Opn-Display"}],
        "auto_update": false,
    });
    let reponse = incus(&[
        "query",
        "-X",
        "POST",
        "/1.0/images",
        "--data",
        &demande.to_string(),
    ])
    .map_err(|e| format!("Incus refuse le téléchargement : {e}"))?;
    let id = lire_operation(&reponse)?.id;
    let chemin = format!("/1.0/operations/{id}");
    loop {
        if arret.load(Ordering::Relaxed) {
            let _ = incus(&["query", "-X", "DELETE", &chemin]);
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(500));
        // Une opération finie disparaît vite : l'image dit alors si elle a
        // réussi.
        let Ok(reponse) = incus(&["query", &chemin]) else {
            break;
        };
        let operation = lire_operation(&reponse)?;
        match operation.statut {
            Statut::EnCours => progres(operation.avance),
            Statut::Reussie => break,
            Statut::Echec => return Err(echec_du_telechargement(&operation.erreur)),
        }
    }
    if !image_presente() {
        return Err("Le téléchargement de l'image s'est arrêté sans la laisser.".into());
    }
    Ok(Some(taille_image().unwrap_or(0)))
}

fn taille_image() -> Option<u64> {
    let alias: Value =
        serde_json::from_str(&incus(&["query", &format!("/1.0/images/aliases/{IMAGE}")]).ok()?)
            .ok()?;
    let empreinte = alias["target"].as_str()?;
    let image: Value =
        serde_json::from_str(&incus(&["query", &format!("/1.0/images/{empreinte}")]).ok()?).ok()?;
    image["size"].as_u64()
}

#[derive(Debug, PartialEq, Eq)]
enum Statut {
    EnCours,
    Reussie,
    Echec,
}

#[derive(Debug)]
struct Operation {
    id: String,
    statut: Statut,
    erreur: String,
    avance: Option<f64>,
}

fn lire_operation(reponse: &str) -> Result<Operation, String> {
    let valeur: Value = serde_json::from_str(reponse)
        .map_err(|_| format!("Incus a répondu autre chose qu'une opération : {reponse}"))?;
    let statut = match valeur["status_code"].as_u64() {
        Some(200) => Statut::Reussie,
        Some(code) if code >= 400 => Statut::Echec,
        _ => Statut::EnCours,
    };
    Ok(Operation {
        id: valeur["id"].as_str().unwrap_or_default().to_string(),
        statut,
        erreur: valeur["err"].as_str().unwrap_or_default().to_string(),
        avance: valeur["metadata"]["download_progress"]
            .as_str()
            .and_then(avance),
    })
}

/// L'échec dit en clair quand c'est le réseau, avec le dernier mot d'Incus.
fn echec_du_telechargement(erreur: &str) -> String {
    let reseau = [
        "Forbidden",
        "dial tcp",
        "no such host",
        "i/o timeout",
        "connection refused",
        "network is unreachable",
    ]
    .iter()
    .any(|signe| erreur.contains(signe));
    if reseau {
        let dernier_mot = erreur.rsplit(": ").next().unwrap_or(erreur);
        format!(
            "Le serveur d'images d'Incus ({SERVEUR_IMAGES}) ne répond pas. Vérifie la connexion \
             à Internet, puis réessaie. Incus dit : {dernier_mot}"
        )
    } else {
        format!("Le téléchargement de l'image a échoué : {erreur}")
    }
}

/// « rootfs: 45% (12.34MB/s) » donne 0,45.
fn avance(texte: &str) -> Option<f64> {
    let pourcent = texte.find('%')?;
    let debut = texte[..pourcent]
        .rfind(|c: char| !(c.is_ascii_digit() || c == '.'))
        .map_or(0, |i| i + 1);
    let valeur: f64 = texte[debut..pourcent].parse().ok()?;
    Some((valeur / 100.0).clamp(0.0, 1.0))
}

/// Une machine telle qu'Incus la voit.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Instance {
    pub nom: String,
    pub en_marche: bool,
    pub memoire: Option<u64>,
}

#[derive(Deserialize)]
struct InstanceListee {
    name: String,
    status: String,
    state: Option<EtatListe>,
}

#[derive(Deserialize)]
struct EtatListe {
    memory: Option<MemoireListee>,
}

#[derive(Deserialize)]
struct MemoireListee {
    usage: Option<u64>,
}

pub(crate) fn instances(projet: &str) -> Result<Vec<Instance>, String> {
    lire_instances(&incus(&["list", "--project", projet, "--format", "json"])?)
}

fn lire_instances(liste: &str) -> Result<Vec<Instance>, String> {
    let liste: Vec<InstanceListee> = serde_json::from_str(liste)
        .map_err(|e| format!("La liste des machines est illisible : {e}"))?;
    Ok(liste
        .into_iter()
        .map(|i| {
            let en_marche = i.status == "Running";
            Instance {
                nom: i.name,
                en_marche,
                memoire: i
                    .state
                    .and_then(|s| s.memory)
                    .and_then(|m| m.usage)
                    .filter(|_| en_marche),
            }
        })
        .collect())
}

pub(crate) fn projet_existe(projet: &str) -> bool {
    incus(&["project", "show", projet]).is_ok()
}

/// Un projet par lab : ses machines y ont leur nom à elles, et l'image vient
/// du projet par défaut, téléchargée une fois pour tous les labs.
pub(crate) fn creer_projet(projet: &str) -> Result<(), String> {
    incus(&[
        "project",
        "create",
        projet,
        "-c",
        "features.images=false",
        "-c",
        "features.profiles=true",
    ])?;
    incus(&[
        "profile",
        "device",
        "add",
        "default",
        "root",
        "disk",
        "path=/",
        &format!("pool={POOL}"),
        "--project",
        projet,
    ])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lit_la_version() {
        assert_eq!(
            version(r#"{"environment": {"server_version": "6.0.4"}}"#),
            "6.0.4"
        );
        assert_eq!(version("pas du json"), "");
    }

    #[test]
    fn lit_l_avance_du_telechargement() {
        assert_eq!(avance("rootfs: 45% (12.34MB/s)"), Some(0.45));
        assert_eq!(avance("metadata: 100% (1.2MB/s)"), Some(1.0));
        assert_eq!(avance("12.5%"), Some(0.125));
        assert_eq!(avance("en attente"), None);
    }

    #[test]
    fn lit_une_operation() {
        let en_cours = lire_operation(
            r#"{"id": "0ddd", "status_code": 103, "err": "",
                "metadata": {"download_progress": "rootfs: 30% (5MB/s)"}}"#,
        )
        .unwrap();
        assert_eq!(en_cours.id, "0ddd");
        assert_eq!(en_cours.statut, Statut::EnCours);
        assert_eq!(en_cours.avance, Some(0.3));

        let echec = lire_operation(
            r#"{"id": "0ddd", "status_code": 400, "err": "Failed getting remote image info", "metadata": null}"#,
        )
        .unwrap();
        assert_eq!(echec.statut, Statut::Echec);
        assert_eq!(echec.erreur, "Failed getting remote image info");
        assert!(lire_operation("Error: pas une opération").is_err());
    }

    #[test]
    fn dit_en_clair_quand_le_reseau_manque() {
        // Relevé le 17 novembre 2026, serveur d'images bloqué.
        let erreur = "Failed getting remote image info: Failed getting image: Failed parsing \
                      stream: Get \"https://images.linuxcontainers.org/streams/v1/index.json\": Forbidden";
        let message = echec_du_telechargement(erreur);
        assert!(message.starts_with("Le serveur d'images d'Incus"));
        assert!(message.ends_with("Incus dit : Forbidden"));
        assert!(echec_du_telechargement("Image not found").contains("Image not found"));
    }

    #[test]
    fn lit_la_liste_des_machines() {
        let liste = r#"[
            {"name": "pc1", "status": "Running", "state": {"memory": {"usage": 13934592}}},
            {"name": "pc2", "status": "Stopped", "state": {"memory": {"usage": 0}}},
            {"name": "routeur", "status": "Stopped", "state": null}
        ]"#;
        assert_eq!(
            lire_instances(liste).unwrap(),
            vec![
                Instance {
                    nom: "pc1".into(),
                    en_marche: true,
                    memoire: Some(13934592)
                },
                Instance {
                    nom: "pc2".into(),
                    en_marche: false,
                    memoire: None
                },
                Instance {
                    nom: "routeur".into(),
                    en_marche: false,
                    memoire: None
                },
            ]
        );
    }
}
