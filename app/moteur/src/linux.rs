//! Sous Linux, l'app utilise le système lui-même, et son Incus.

use std::fs;
use std::path::Path;

use sysinfo::System;

use crate::incus;
use crate::installation::{Etape, EtatEtape, Pas};
use crate::rapport::{
    Blocage, LinuxApp, Memoire, Plateforme, Rapport, Verification, machine, machines,
};

pub(crate) fn verifier() -> Rapport {
    let machine = machine();
    let systeme = System::long_os_version().unwrap_or_else(|_| "Linux".into());
    let noyau = System::kernel_version().unwrap_or_default();

    let drapeaux = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let processeur_capable = drapeaux
        .lines()
        .filter(|l| l.starts_with("flags"))
        .any(|l| l.split_whitespace().any(|f| f == "vmx" || f == "svm"));
    let kvm_existe = Path::new("/dev/kvm").exists();
    let kvm_ouvert = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/kvm")
        .is_ok();
    let wifi = module_present(&Path::new("/lib/modules").join(&noyau), "mac80211_hwsim");

    let mut verifications = vec![
        Verification::ok(&systeme, format!("noyau {noyau}")),
        machine.ligne_memoire(),
    ];
    verifications.push(if kvm_existe {
        Verification::ok("La virtualisation", "active")
    } else if processeur_capable {
        Verification::manque(
            "La virtualisation",
            "coupée dans le BIOS, ou le module kvm n'est pas chargé",
        )
    } else {
        Verification::manque("La virtualisation", "ton processeur ne la propose pas ici")
    });
    verifications.push(if kvm_ouvert {
        Verification::ok("Machines virtuelles", "oui, KVM répond")
    } else if kvm_existe {
        Verification::manque(
            "Machines virtuelles",
            "KVM existe, mais ton compte n'y a pas accès",
        )
    } else {
        Verification::manque("Machines virtuelles", "non, sans KVM")
    });
    verifications.push(if wifi {
        Verification::ok("WiFi simulé", "oui")
    } else {
        Verification::manque("WiFi simulé", "non, le module mac80211_hwsim manque")
    });
    let etat = incus::etat();
    let (etat_machines, ligne) = machines(&etat, false);
    verifications.push(ligne);
    let blocage = match etat {
        incus::Etat::Absent => Some(Blocage::IncusAbsent),
        incus::Etat::Muet(_) => Some(Blocage::IncusMuet),
        _ => None,
    };

    Rapport {
        plateforme: Plateforme::Linux,
        systeme,
        verifications,
        blocage,
        memoire: Memoire {
            totale_go: machine.totale_go,
            libre_go: machine.libre_go,
            wsl_go: None,
            proposition_go: None,
        },
        processeur: machine.processeur,
        linux_app: LinuxApp::Systeme,
        machines: etat_machines,
        noyau: Some(noyau),
        kvm: Some(kvm_ouvert),
        wifi_simule: Some(wifi),
    }
}

/// Prépare l'Incus du système : la place des machines. L'image vient après.
pub(crate) fn preparer_incus(signaler: &mut dyn FnMut(Etape)) -> Result<(), String> {
    let mut dire = |etat, detail: String| {
        signaler(Etape {
            pas: Pas::Incus,
            etat,
            detail,
            avance: None,
        })
    };
    dire(EtatEtape::EnCours, "la place des machines".into());
    match incus::attendre(5).and_then(|version| incus::preparer_pool().map(|_| version)) {
        Ok(version) => {
            dire(EtatEtape::Fait, format!("Incus {version}"));
            Ok(())
        }
        Err(erreur) => {
            dire(EtatEtape::Erreur, erreur.clone());
            Err(erreur)
        }
    }
}

/// Cherche un module du noyau, compilé à part ou dans le noyau.
pub(crate) fn module_present(dossier: &Path, nom: &str) -> bool {
    let integre = fs::read_to_string(dossier.join("modules.builtin"))
        .is_ok_and(|liste| liste.lines().any(|l| l.ends_with(&format!("/{nom}.ko"))));
    integre || fichier_present(dossier, &format!("{nom}.ko"))
}

fn fichier_present(dossier: &Path, debut: &str) -> bool {
    let Ok(entrees) = fs::read_dir(dossier) else {
        return false;
    };
    entrees.flatten().any(|entree| {
        let chemin = entree.path();
        if entree.file_type().is_ok_and(|t| t.is_dir()) {
            fichier_present(&chemin, debut)
        } else {
            entree.file_name().to_string_lossy().starts_with(debut)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trouve_un_module_compile_a_part() {
        let dossier = std::env::temp_dir().join("opn-display-modules-test");
        let sous = dossier.join("kernel/drivers/net/wireless/virtual");
        fs::create_dir_all(&sous).unwrap();
        fs::write(sous.join("mac80211_hwsim.ko.xz"), b"").unwrap();
        assert!(module_present(&dossier, "mac80211_hwsim"));
        assert!(!module_present(&dossier, "kvm_amd"));
        fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn trouve_un_module_integre() {
        let dossier = std::env::temp_dir().join("opn-display-modules-integres");
        fs::create_dir_all(&dossier).unwrap();
        fs::write(
            dossier.join("modules.builtin"),
            "kernel/drivers/net/wireless/virtual/mac80211_hwsim.ko\n",
        )
        .unwrap();
        assert!(module_present(&dossier, "mac80211_hwsim"));
        fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn le_rapport_se_fait_sans_erreur() {
        let rapport = verifier();
        assert_eq!(rapport.plateforme, Plateforme::Linux);
        assert_eq!(rapport.verifications.len(), 6);
    }
}
