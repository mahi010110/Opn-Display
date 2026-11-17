use serde::Serialize;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::format::{go, texte_go};
use crate::incus;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Plateforme {
    Windows,
    Linux,
    Autre,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Etat {
    Ok,
    Manque,
    Inconnu,
}

/// Une ligne de « Ton PC » : ce qui a été regardé, et ce qu'on a trouvé.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Verification {
    pub nom: String,
    pub etat: Etat,
    pub detail: String,
}

impl Verification {
    pub(crate) fn ok(nom: &str, detail: impl Into<String>) -> Self {
        Self::nouvelle(nom, Etat::Ok, detail)
    }

    pub(crate) fn manque(nom: &str, detail: impl Into<String>) -> Self {
        Self::nouvelle(nom, Etat::Manque, detail)
    }

    pub(crate) fn inconnu(nom: &str, detail: impl Into<String>) -> Self {
        Self::nouvelle(nom, Etat::Inconnu, detail)
    }

    fn nouvelle(nom: &str, etat: Etat, detail: impl Into<String>) -> Self {
        Verification {
            nom: nom.to_string(),
            etat,
            detail: detail.into(),
        }
    }
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Fabricant {
    Intel,
    Amd,
    Autre,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Processeur {
    pub nom: String,
    pub fabricant: Fabricant,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Memoire {
    pub totale_go: f64,
    pub libre_go: f64,
    /// Ce que WSL2 peut prendre, réglé ou par défaut. Absent hors Windows.
    pub wsl_go: Option<f64>,
    /// La limite que l'app propose pour WSL2, quand elle vaut le coup.
    pub proposition_go: Option<u32>,
}

/// Ce qui empêche de lancer un lab, et qui demande un geste hors de l'app.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Blocage {
    /// Le processeur sait faire, le BIOS l'a coupée.
    VirtualisationCoupee,
    /// Le processeur ne propose pas la virtualisation.
    SansVirtualisation,
    /// WSL2 n'est pas installé, ou pas encore activé.
    WslAbsent,
    /// Sous Linux, Incus n'est pas installé.
    IncusAbsent,
    /// Sous Linux, Incus ne répond pas, ou le compte n'y a pas accès.
    IncusMuet,
    SystemeNonPrisEnCharge,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum LinuxApp {
    /// Sous Windows, avant l'installation.
    AInstaller,
    /// Sous Windows, Debian est installé dans WSL2.
    Installe,
    /// Sous Linux, l'app utilise le système lui-même.
    Systeme,
    Aucun,
}

/// Où en est Incus, qui fait tourner les machines du lab.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Machines {
    /// Incus manque. Sous Windows, l'installation le pose.
    AInstaller,
    /// Incus est installé, mais il ne répond pas.
    Muettes,
    /// Incus répond ; il manque la place des machines ou leur image.
    APreparer,
    Pretes,
}

/// La ligne de « Ton PC » qui dit où en est Incus.
pub(crate) fn machines(etat: &incus::Etat, sous_windows: bool) -> (Machines, Verification) {
    const NOM: &str = "Les machines du lab";
    match etat {
        incus::Etat::Pret { version } => (
            Machines::Pretes,
            Verification::ok(NOM, format!("Incus {version}")),
        ),
        incus::Etat::APreparer { version } => (
            Machines::APreparer,
            Verification::inconnu(NOM, format!("Incus {version}, encore à préparer")),
        ),
        incus::Etat::Absent if sous_windows => (
            Machines::AInstaller,
            Verification::inconnu(NOM, "Incus, à installer"),
        ),
        incus::Etat::Absent => (
            Machines::AInstaller,
            Verification::manque(NOM, "Incus n'est pas installé"),
        ),
        incus::Etat::Muet(_) => (
            Machines::Muettes,
            Verification::manque(NOM, "Incus ne répond pas"),
        ),
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Rapport {
    pub plateforme: Plateforme,
    pub systeme: String,
    pub verifications: Vec<Verification>,
    pub blocage: Option<Blocage>,
    pub processeur: Processeur,
    pub memoire: Memoire,
    pub linux_app: LinuxApp,
    pub machines: Machines,
    pub noyau: Option<String>,
    pub kvm: Option<bool>,
    pub wifi_simule: Option<bool>,
}

/// Ce que le PC a, avant toute vérification propre à un système.
pub(crate) struct Machine {
    pub processeur: Processeur,
    pub totale_go: f64,
    pub libre_go: f64,
}

pub(crate) fn machine() -> Machine {
    let systeme = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::nothing()),
    );
    let Ok(systeme) = systeme else {
        return Machine {
            processeur: Processeur {
                nom: String::new(),
                fabricant: Fabricant::Autre,
            },
            totale_go: 0.0,
            libre_go: 0.0,
        };
    };
    let cpu = systeme.cpus().first();
    let nom = cpu
        .map(|c| c.brand().trim().to_string())
        .unwrap_or_default();
    let vendeur = cpu.map(|c| c.vendor_id().to_string()).unwrap_or_default();
    Machine {
        processeur: Processeur {
            nom,
            fabricant: fabricant(&vendeur),
        },
        totale_go: go(systeme.total_memory()),
        libre_go: go(systeme.available_memory()),
    }
}

pub(crate) fn fabricant(vendeur: &str) -> Fabricant {
    match vendeur {
        "GenuineIntel" => Fabricant::Intel,
        "AuthenticAMD" => Fabricant::Amd,
        _ => Fabricant::Autre,
    }
}

impl Machine {
    pub(crate) fn ligne_memoire(&self) -> Verification {
        Verification::ok(
            "Mémoire",
            format!(
                "{} Go, {} libres",
                texte_go(self.totale_go.round()),
                texte_go(self.libre_go)
            ),
        )
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
pub(crate) fn non_pris_en_charge() -> Rapport {
    let machine = machine();
    Rapport {
        plateforme: Plateforme::Autre,
        systeme: String::new(),
        verifications: vec![machine.ligne_memoire()],
        blocage: Some(Blocage::SystemeNonPrisEnCharge),
        memoire: Memoire {
            totale_go: machine.totale_go,
            libre_go: machine.libre_go,
            wsl_go: None,
            proposition_go: None,
        },
        processeur: machine.processeur,
        linux_app: LinuxApp::Aucun,
        machines: Machines::AInstaller,
        noyau: None,
        kvm: None,
        wifi_simule: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnait_le_fabricant() {
        assert_eq!(fabricant("AuthenticAMD"), Fabricant::Amd);
        assert_eq!(fabricant("GenuineIntel"), Fabricant::Intel);
        assert_eq!(fabricant("ARM"), Fabricant::Autre);
    }

    #[test]
    fn la_machine_a_de_la_memoire() {
        let machine = machine();
        assert!(machine.totale_go > 0.0);
        assert!(machine.libre_go <= machine.totale_go);
    }
}
