//! L'installation de ce qui fait tourner les machines. Sous Windows, c'est
//! Debian importé dans WSL2, puis Incus dans Debian. Sous Linux, Incus est
//! celui du système : l'app y prépare sa place et l'image des machines.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::incus;

/// La liste des distributions de WSL, tenue par Microsoft. Celle de Debian
/// est fabriquée par le projet Debian, avec son empreinte SHA-256.
pub const LISTE_DES_IMAGES: &str =
    "https://raw.githubusercontent.com/microsoft/WSL/master/distributions/DistributionInfo.json";

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum EtatEtape {
    EnCours,
    Fait,
    Saute,
    Erreur,
}

/// Les étapes, dans l'ordre. Sous Linux, il n'y a que les deux dernières.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Pas {
    Telecharger,
    Importer,
    Memoire,
    Demarrer,
    Incus,
    Image,
}

/// Où en est une étape de l'installation. `avance` va de 0 à 1.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Etape {
    pub pas: Pas,
    pub etat: EtatEtape,
    pub detail: String,
    pub avance: Option<f64>,
}

pub struct Options {
    /// Le dossier de l'app où ranger le disque de Debian.
    pub dossier: PathBuf,
    /// La nouvelle limite de mémoire de WSL2, si l'utilisateur a dit oui.
    pub memoire_go: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fin {
    Installe,
    /// L'utilisateur a arrêté l'installation : ce qu'elle avait mis dans
    /// WSL2 est reparti.
    Arretee,
}

/// Installe ce qui manque, et dit où il en est à chaque étape. `arret`
/// passe à vrai quand l'utilisateur arrête l'installation.
pub fn installer(
    options: &Options,
    signaler: &mut dyn FnMut(Etape),
    arret: &AtomicBool,
) -> Result<Fin, String> {
    #[cfg(windows)]
    let fin = crate::windows::installer(options, signaler, arret)?;
    #[cfg(target_os = "linux")]
    let fin = {
        let _ = options;
        crate::linux::preparer_incus(signaler)?;
        Fin::Installe
    };
    #[cfg(not(any(windows, target_os = "linux")))]
    let fin = {
        let _ = (options, &signaler, arret);
        return Err("Opn-Display ne tourne pas encore sur ce système.".into());
    };
    if fin == Fin::Arretee || arret.load(Ordering::Relaxed) {
        return Ok(Fin::Arretee);
    }
    telecharger_image(signaler, arret)
}

/// L'étape de l'image : Debian 13 pour les machines, téléchargé par Incus.
fn telecharger_image(signaler: &mut dyn FnMut(Etape), arret: &AtomicBool) -> Result<Fin, String> {
    let mut dire = |etat, detail: String, avance| {
        signaler(Etape {
            pas: Pas::Image,
            etat,
            detail,
            avance,
        })
    };
    if incus::image_presente() {
        dire(EtatEtape::Saute, "déjà là".into(), None);
        return Ok(Fin::Installe);
    }
    dire(EtatEtape::EnCours, "Debian 13".into(), None);
    let resultat = incus::telecharger_image(arret, &mut |avance| {
        let detail = match avance {
            Some(a) => format!("Debian 13, {} %", (a * 100.0).round()),
            None => "Debian 13".into(),
        };
        dire(EtatEtape::EnCours, detail, avance)
    });
    match resultat {
        Ok(Some(taille)) => {
            dire(
                EtatEtape::Fait,
                format!("Debian 13, {} Mo", taille >> 20),
                Some(1.0),
            );
            Ok(Fin::Installe)
        }
        Ok(None) => Ok(Fin::Arretee),
        Err(erreur) => {
            dire(EtatEtape::Erreur, erreur.clone(), None);
            Err(erreur)
        }
    }
}

/// Où en est apt-get, d'après les lignes de `APT::Status-Fd` : le
/// téléchargement compte pour la première moitié, l'installation pour la
/// seconde.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn avance_apt(ligne: &str) -> Option<f64> {
    let mut morceaux = ligne.splitn(4, ':');
    let sorte = morceaux.next()?;
    let pourcent: f64 = morceaux.nth(1)?.trim().parse().ok()?;
    let part = (pourcent / 100.0).clamp(0.0, 1.0);
    match sorte {
        "dlstatus" => Some(part / 2.0),
        "pmstatus" => Some(0.5 + part / 2.0),
        _ => None,
    }
}

/// Retire Debian de WSL2, avec tout ce qu'il contient. Les autres Linux de
/// WSL2 ne changent pas.
pub fn retirer(dossier: &Path) -> Result<(), String> {
    #[cfg(windows)]
    return crate::windows::retirer(dossier);
    #[cfg(not(windows))]
    {
        let _ = dossier;
        Err("Sur ce système, l'app n'a rien installé.".into())
    }
}

/// Lance l'installation de WSL par Windows. Windows demande l'accord de
/// l'utilisateur, puis un redémarrage.
pub fn installer_wsl() -> Result<(), String> {
    #[cfg(windows)]
    return crate::windows::installer_wsl();
    #[cfg(not(windows))]
    Err("WSL n'existe que sous Windows.".into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub url: String,
    pub sha256: String,
}

#[derive(Deserialize)]
struct Liste {
    #[serde(rename = "ModernDistributions")]
    distributions: std::collections::HashMap<String, Vec<Distribution>>,
}

#[derive(Deserialize)]
struct Distribution {
    #[serde(rename = "Default", default)]
    par_defaut: bool,
    #[serde(rename = "Amd64Url")]
    amd64: Option<Lien>,
    #[serde(rename = "Arm64Url")]
    arm64: Option<Lien>,
}

#[derive(Deserialize)]
struct Lien {
    #[serde(rename = "Url")]
    url: String,
    #[serde(rename = "Sha256")]
    sha256: String,
}

/// L'image de Debian à télécharger, pour un processeur Intel ou AMD, ou ARM.
pub fn image_debian(liste: &str, arm: bool) -> Result<Image, String> {
    let liste: Liste = serde_json::from_str(liste)
        .map_err(|e| format!("La liste des images est illisible : {e}"))?;
    let versions = liste
        .distributions
        .get("Debian")
        .ok_or("La liste des images ne contient plus Debian.")?;
    let debian = versions
        .iter()
        .find(|d| d.par_defaut)
        .or_else(|| versions.first())
        .ok_or("La liste des images ne contient plus Debian.")?;
    let lien = if arm { &debian.arm64 } else { &debian.amd64 }
        .as_ref()
        .ok_or("Debian n'a pas d'image pour ce processeur.")?;
    Ok(Image {
        url: lien.url.clone(),
        sha256: lien.sha256.to_lowercase(),
    })
}

/// L'extension qui dit à `wsl --import` comment l'archive est compressée.
pub fn extension(debut: &[u8]) -> &'static str {
    match debut {
        [0x1f, 0x8b, ..] => "tar.gz",
        [0xfd, b'7', b'z', b'X', b'Z', 0x00, ..] => "tar.xz",
        _ => "tar",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTE: &str = r#"{
        "ModernDistributions": {
            "Debian": [{
                "Name": "Debian",
                "Default": true,
                "Amd64Url": {"Url": "https://exemple.org/debian-amd64.wsl", "Sha256": "ABC123"},
                "Arm64Url": {"Url": "https://exemple.org/debian-arm64.wsl", "Sha256": "def456"}
            }],
            "Ubuntu": []
        },
        "Default": "Ubuntu",
        "Distributions": []
    }"#;

    #[test]
    fn trouve_l_image_de_debian() {
        assert_eq!(
            image_debian(LISTE, false),
            Ok(Image {
                url: "https://exemple.org/debian-amd64.wsl".into(),
                sha256: "abc123".into()
            })
        );
        assert_eq!(
            image_debian(LISTE, true).unwrap().url,
            "https://exemple.org/debian-arm64.wsl"
        );
    }

    #[test]
    fn dit_quand_debian_manque() {
        let sans = r#"{"ModernDistributions": {"Ubuntu": []}}"#;
        assert!(image_debian(sans, false).is_err());
        assert!(image_debian("pas du json", false).is_err());
    }

    #[test]
    fn suit_apt_get() {
        assert_eq!(
            avance_apt("dlstatus:3:50.0000:Retrieving file 3 of 40"),
            Some(0.25)
        );
        assert_eq!(avance_apt("pmstatus:incus:100:Installed incus"), Some(1.0));
        assert_eq!(avance_apt("pmstatus:dpkg-exec:0:Running dpkg"), Some(0.5));
        assert_eq!(avance_apt("Reading package lists..."), None);
    }

    #[test]
    fn reconnait_la_compression() {
        assert_eq!(extension(&[0x1f, 0x8b, 8, 0]), "tar.gz");
        assert_eq!(extension(&[0xfd, b'7', b'z', b'X', b'Z', 0]), "tar.xz");
        assert_eq!(extension(b"debian/"), "tar");
    }
}
