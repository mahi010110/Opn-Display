//! Le moteur d'Opn-Display : ce qui fait tourner le lab. Il vérifie le PC,
//! installe le Linux de l'app et Incus, et fait tourner le premier lab.

mod format;
mod incus;
pub mod installation;
pub mod lab;
mod rapport;
pub mod sortie;
mod systeme;
pub mod terminal;
pub mod veille;
pub mod wslconfig;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(windows)]
mod windows;

pub use rapport::*;

/// Le nom du Linux de l'app dans WSL2.
pub const NOM_DISTRIBUTION: &str = "Opn-Display";

/// Regarde ce que le PC sait faire.
pub fn verifier() -> Rapport {
    #[cfg(windows)]
    return windows::verifier();
    #[cfg(target_os = "linux")]
    return linux::verifier();
    #[cfg(not(any(windows, target_os = "linux")))]
    return rapport::non_pris_en_charge();
}

#[cfg(all(test, windows))]
mod tests {
    /// Sur le Windows de GitHub, sans WSL2 : le rapport se fait quand même.
    #[test]
    fn le_rapport_se_fait_sous_windows() {
        let rapport = super::verifier();
        assert_eq!(rapport.plateforme, super::Plateforme::Windows);
        assert!(rapport.systeme.starts_with("Windows"));
        assert!(rapport.verifications.len() >= 4);
        eprintln!("{rapport:#?}");
    }
}
