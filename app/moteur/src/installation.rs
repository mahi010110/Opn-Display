//! L'installation du Linux de l'app. Sous Windows, c'est Debian importé dans
//! WSL2. Sous Linux, il n'y a rien à installer.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

/// Où en est une étape de l'installation. `avance` va de 0 à 1.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Etape {
    pub numero: usize,
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

/// Installe le Linux de l'app, et dit où il en est à chaque étape.
pub fn installer(options: &Options, signaler: &mut dyn FnMut(Etape)) -> Result<(), String> {
    #[cfg(windows)]
    return crate::windows::installer(options, signaler);
    #[cfg(not(windows))]
    {
        let _ = (options, signaler);
        Err("Sur ce système, l'app n'a rien à installer.".into())
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
    fn reconnait_la_compression() {
        assert_eq!(extension(&[0x1f, 0x8b, 8, 0]), "tar.gz");
        assert_eq!(extension(&[0xfd, b'7', b'z', b'X', b'Z', 0]), "tar.xz");
        assert_eq!(extension(b"debian/"), "tar");
    }
}
