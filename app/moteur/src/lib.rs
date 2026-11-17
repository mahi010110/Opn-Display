//! Le moteur d'Opn-Display : ce qui fait tourner le lab. Pour l'instant, il
//! vérifie le PC et installe le Linux de l'app.

mod format;
pub mod installation;
mod rapport;
pub mod sortie;
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
