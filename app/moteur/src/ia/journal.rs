//! Le journal de ce que l'IA a fait : dans le lab, et dans sa mémoire. L'app
//! le garde dans un fichier, et Réglages le montre.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::outils::Action;

/// Au-delà, les plus anciennes entrées partent.
pub const ENTREES_MAX: usize = 500;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Entree {
    /// En secondes depuis 1970.
    pub quand: u64,
    #[serde(flatten)]
    pub action: Action,
}

/// Les entrées, de la plus ancienne à la plus récente. Un fichier abîmé se
/// lit comme un journal vide.
pub fn lire(chemin: &Path) -> Vec<Entree> {
    std::fs::read_to_string(chemin)
        .ok()
        .and_then(|texte| serde_json::from_str(&texte).ok())
        .unwrap_or_default()
}

pub fn ajouter(chemin: &Path, entree: Entree) -> Result<(), String> {
    let mut entrees = lire(chemin);
    entrees.push(entree);
    let trop = entrees.len().saturating_sub(ENTREES_MAX);
    entrees.drain(..trop);
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier).map_err(|e| e.to_string())?;
    }
    let texte = serde_json::to_string(&entrees).map_err(|e| e.to_string())?;
    std::fs::write(chemin, texte).map_err(|e| format!("Le journal ne s'écrit pas : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ia::outils::Accord;

    fn entree(quand: u64) -> Entree {
        Entree {
            quand,
            action: Action {
                lieu: "pc1".into(),
                quoi: "Commande".into(),
                commande: Some("ip route".into()),
                accord: Some(Accord::Donne),
            },
        }
    }

    #[test]
    fn garde_les_dernieres_entrees() {
        let dossier =
            std::env::temp_dir().join(format!("opn-display-journal-{}", std::process::id()));
        let chemin = dossier.join("journal.json");
        let _ = std::fs::remove_file(&chemin);
        assert!(lire(&chemin).is_empty());
        for quand in 0..(ENTREES_MAX as u64 + 3) {
            ajouter(&chemin, entree(quand)).unwrap();
        }
        let entrees = lire(&chemin);
        assert_eq!(entrees.len(), ENTREES_MAX);
        assert_eq!(entrees[0].quand, 3);
        assert_eq!(entrees.last(), Some(&entree(ENTREES_MAX as u64 + 2)));
        let texte = std::fs::read_to_string(&chemin).unwrap();
        assert!(texte.contains("\"accord\":\"donne\""), "{texte}");
        let _ = std::fs::remove_dir_all(dossier);
    }
}
