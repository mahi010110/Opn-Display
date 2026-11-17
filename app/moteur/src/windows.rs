//! Sous Windows, le Linux de l'app est Debian, importé dans WSL2 sous le nom
//! « Opn-Display ».

use std::fs;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::{Digest, Sha256};

use crate::format::texte_go;
use crate::installation::{
    Etape, EtatEtape, Fin, Image, LISTE_DES_IMAGES, Options, extension, image_debian,
};
use crate::rapport::{
    Blocage, Fabricant, LinuxApp, Memoire, Plateforme, Rapport, Verification, machine,
};
use crate::sortie::{self, VersionWsl};
use crate::{NOM_DISTRIBUTION, wslconfig};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const PF_VIRT_FIRMWARE_ENABLED: u32 = 21;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn IsProcessorFeaturePresent(feature: u32) -> i32;
}

/// Cherche le module du WiFi simulé, compilé à part ou dans le noyau de WSL.
const CHERCHER_HWSIM: &str = r#"m=/lib/modules/$(uname -r); grep -qs '/mac80211_hwsim.ko$' "$m/modules.builtin" || find "$m" -name 'mac80211_hwsim.ko*' 2>/dev/null | grep -q ."#;

fn commande(programme: &str) -> Command {
    let mut commande = Command::new(programme);
    // Sans ce drapeau, chaque appel à wsl.exe ou reg.exe ouvre une console
    // qui clignote devant l'app.
    commande.creation_flags(CREATE_NO_WINDOW);
    // Sinon wsl.exe répond en UTF-16.
    commande.env("WSL_UTF8", "1");
    commande
}

fn lancer(programme: &str, args: &[&str]) -> Option<Output> {
    commande(programme).args(args).output().ok()
}

fn texte_sortie(sortie: &Output) -> String {
    sortie::texte(&sortie.stdout)
}

fn texte_erreur(sortie: &Output) -> String {
    let erreur = sortie::texte(&sortie.stderr);
    if erreur.trim().is_empty() {
        texte_sortie(sortie)
    } else {
        erreur
    }
    .trim()
    .to_string()
}

fn dans_debian(commande_linux: &[&str]) -> Option<Output> {
    let mut args = vec!["-d", NOM_DISTRIBUTION, "-u", "root", "--"];
    args.extend_from_slice(commande_linux);
    lancer("wsl.exe", &args)
}

fn reponse_debian(commande_linux: &[&str]) -> Option<String> {
    dans_debian(commande_linux)
        .filter(|s| s.status.success())
        .map(|s| texte_sortie(&s).trim().to_string())
        .filter(|texte| !texte.is_empty())
}

fn reussit_dans_debian(commande_linux: &[&str]) -> bool {
    dans_debian(commande_linux).is_some_and(|s| s.status.success())
}

pub(crate) fn verifier() -> Rapport {
    let machine = machine();
    let totale = machine.totale_go;
    let (build, edition) = version_windows();
    let windows11 = build.is_none_or(|b| b >= 22000);
    let nom_windows = if windows11 {
        "Windows 11"
    } else {
        "Windows 10"
    };

    let mut verifications = vec![
        if windows11 {
            Verification::ok(nom_windows, edition.clone())
        } else {
            Verification::manque(nom_windows, "les machines virtuelles demandent Windows 11")
        },
        machine.ligne_memoire(),
    ];

    let mut blocage = None;
    match virtualisation(machine.processeur.fabricant) {
        Virtualisation::Active => {
            verifications.push(Verification::ok("La virtualisation", "active"))
        }
        Virtualisation::Disponible => {
            verifications.push(Verification::manque(
                "La virtualisation",
                "pas encore activée pour WSL2",
            ));
            blocage = Some(Blocage::WslAbsent);
        }
        Virtualisation::Coupee => {
            verifications.push(Verification::manque(
                "La virtualisation",
                "coupée dans le BIOS de ton PC",
            ));
            blocage = Some(Blocage::VirtualisationCoupee);
        }
        Virtualisation::Absente => {
            verifications.push(Verification::manque(
                "La virtualisation",
                "ton processeur ne la propose pas",
            ));
            blocage = Some(Blocage::SansVirtualisation);
        }
    }

    let wsl = version_wsl();
    match &wsl {
        Some(version) => verifications.push(Verification::ok("WSL2", detail_wsl(version))),
        None => {
            verifications.push(Verification::manque("WSL2", "pas installé, ou trop ancien"));
            blocage = blocage.or(Some(Blocage::WslAbsent));
        }
    }

    let installe = wsl.is_some() && debian_installe();
    let wsl_go = memoire_wsl(totale);
    let mut noyau = wsl.as_ref().and_then(|v| v.noyau.clone());
    let (mut kvm, mut wifi) = (None, None);

    if installe {
        let os = reponse_debian(&["sh", "-c", ". /etc/os-release && echo \"$PRETTY_NAME\""]);
        verifications.push(match os {
            Some(nom) => Verification::ok("Le Linux de l'app", nom),
            None => Verification::manque("Le Linux de l'app", "installé, mais il ne répond pas"),
        });
        if let Some(version) = reponse_debian(&["uname", "-r"]) {
            noyau = Some(version);
        }
        let a_kvm = reussit_dans_debian(&["test", "-e", "/dev/kvm"]);
        let a_wifi = reussit_dans_debian(&["sh", "-c", CHERCHER_HWSIM]);
        verifications.push(if a_kvm {
            Verification::ok("Machines virtuelles", "oui, KVM répond dans WSL2")
        } else {
            Verification::manque("Machines virtuelles", "non, KVM ne répond pas dans WSL2")
        });
        verifications.push(ligne_wifi(a_wifi, noyau.as_deref()));
        verifications.push(Verification::ok(
            "Mémoire de WSL2",
            format!(
                "{} Go sur {}",
                texte_go(wsl_go.round()),
                texte_go(totale.round())
            ),
        ));
        kvm = Some(a_kvm);
        wifi = Some(a_wifi);
    } else if blocage.is_none() {
        verifications.push(if windows11 {
            Verification::inconnu("Machines virtuelles", "à vérifier une fois Debian installé")
        } else {
            Verification::manque("Machines virtuelles", "elles demandent Windows 11")
        });
        verifications.push(match noyau.as_deref() {
            Some(n) if !sortie::noyau_au_moins(n, 6, 18) => ligne_wifi(false, Some(n)),
            _ => Verification::inconnu("WiFi simulé", "à vérifier une fois Debian installé"),
        });
    }

    let linux_app = if installe {
        LinuxApp::Installe
    } else if blocage.is_some() {
        LinuxApp::Aucun
    } else {
        LinuxApp::AInstaller
    };

    Rapport {
        plateforme: Plateforme::Windows,
        systeme: format!("{nom_windows} {edition}").trim().to_string(),
        verifications,
        blocage,
        memoire: Memoire {
            totale_go: totale,
            libre_go: machine.libre_go,
            wsl_go: Some(wsl_go),
            proposition_go: wslconfig::proposition(totale, wsl_go),
        },
        processeur: machine.processeur,
        linux_app,
        noyau,
        kvm,
        wifi_simule: wifi,
    }
}

fn detail_wsl(version: &VersionWsl) -> String {
    match &version.noyau {
        Some(noyau) => format!("version {}, noyau {}", version.wsl, noyau),
        None => format!("version {}", version.wsl),
    }
}

fn ligne_wifi(present: bool, noyau: Option<&str>) -> Verification {
    match (present, noyau) {
        (true, _) => Verification::ok("WiFi simulé", "oui"),
        (false, Some(n)) if !sortie::noyau_au_moins(n, 6, 18) => Verification::manque(
            "WiFi simulé",
            format!(
                "non, il demande le noyau 6.18 de WSL, tu as le {}",
                sortie::noyau_court(n)
            ),
        ),
        (false, _) => Verification::manque("WiFi simulé", "non, le module mac80211_hwsim manque"),
    }
}

fn version_windows() -> (Option<u32>, String) {
    let cle = r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion";
    let lire = |nom: &str| {
        lancer("reg.exe", &["query", cle, "/v", nom])
            .and_then(|s| sortie::valeur_reg(&texte_sortie(&s), nom))
    };
    let build = lire("CurrentBuild").and_then(|b| b.parse().ok());
    let edition = lire("EditionID")
        .map(|e| sortie::edition_windows(&e))
        .unwrap_or_default();
    (build, edition)
}

fn version_wsl() -> Option<VersionWsl> {
    let reponse = lancer("wsl.exe", &["--version"])?;
    if !reponse.status.success() {
        return None;
    }
    sortie::version_wsl(&texte_sortie(&reponse))
}

fn debian_installe() -> bool {
    lancer("wsl.exe", &["--list", "--quiet"]).is_some_and(|s| {
        sortie::distributions(&texte_sortie(&s))
            .iter()
            .any(|d| d == NOM_DISTRIBUTION)
    })
}

fn fichier_wslconfig() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(|dossier| PathBuf::from(dossier).join(".wslconfig"))
}

/// Ce que WSL2 peut prendre : la valeur du fichier, sinon la moitié de la
/// mémoire du PC, sa valeur par défaut.
fn memoire_wsl(totale: f64) -> f64 {
    fichier_wslconfig()
        .and_then(|f| fs::read_to_string(f).ok())
        .and_then(|contenu| wslconfig::memoire(&contenu))
        .unwrap_or(totale / 2.0)
}

enum Virtualisation {
    Active,
    Disponible,
    Coupee,
    Absente,
}

#[cfg(target_arch = "x86_64")]
fn virtualisation(fabricant: Fabricant) -> Virtualisation {
    if hyperviseur_present() {
        return Virtualisation::Active;
    }
    // SAFETY : fonction de kernel32 qui ne prend qu'un nombre.
    if unsafe { IsProcessorFeaturePresent(PF_VIRT_FIRMWARE_ENABLED) } != 0 {
        return Virtualisation::Disponible;
    }
    if processeur_capable(fabricant) {
        Virtualisation::Coupee
    } else {
        Virtualisation::Absente
    }
}

// Sur un PC à processeur ARM, on laisse WSL dire s'il démarre.
#[cfg(not(target_arch = "x86_64"))]
fn virtualisation(_: Fabricant) -> Virtualisation {
    Virtualisation::Active
}

/// Quand Hyper-V tourne (et donc WSL2), Windows lui-même est une machine
/// virtuelle : le bit 31 de CPUID le dit.
#[cfg(target_arch = "x86_64")]
#[allow(unused_unsafe)]
fn hyperviseur_present() -> bool {
    // SAFETY : CPUID existe sur tout processeur x86_64.
    let feuille = unsafe { std::arch::x86_64::__cpuid(1) };
    feuille.ecx & (1 << 31) != 0
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_unsafe)]
fn processeur_capable(fabricant: Fabricant) -> bool {
    // SAFETY : CPUID existe sur tout processeur x86_64.
    match fabricant {
        Fabricant::Amd => unsafe { std::arch::x86_64::__cpuid(0x8000_0001) }.ecx & (1 << 2) != 0,
        _ => unsafe { std::arch::x86_64::__cpuid(1) }.ecx & (1 << 5) != 0,
    }
}

pub(crate) fn installer(
    options: &Options,
    signaler: &mut dyn FnMut(Etape),
    arret: &AtomicBool,
) -> Result<Fin, String> {
    let arrete = || arret.load(Ordering::Relaxed);
    let mut importe_ici = false;
    let mut dire = |numero: usize, etat: EtatEtape, detail: String, avance: Option<f64>| {
        signaler(Etape {
            numero,
            etat,
            detail,
            avance,
        })
    };

    if debian_installe() {
        dire(0, EtatEtape::Saute, "Debian est déjà là".into(), None);
        dire(1, EtatEtape::Saute, String::new(), None);
    } else {
        dire(0, EtatEtape::EnCours, "la liste des images".into(), None);
        let agent = agent();
        let archive = lire_texte(&agent, LISTE_DES_IMAGES)
            .and_then(|liste| image_debian(&liste, cfg!(target_arch = "aarch64")))
            .and_then(|image| {
                fs::create_dir_all(&options.dossier)
                    .map_err(|e| format!("Le dossier de l'app est inaccessible : {e}"))?;
                telecharger(
                    &agent,
                    &image,
                    &options.dossier,
                    arret,
                    &mut |recus, total| {
                        let detail = match total {
                            Some(total) => format!("{} Mo sur {}", recus >> 20, total >> 20),
                            None => format!("{} Mo", recus >> 20),
                        };
                        dire(
                            0,
                            EtatEtape::EnCours,
                            detail,
                            total.map(|t| recus as f64 / t as f64),
                        );
                    },
                )
            });
        let archive = match archive {
            Ok(Some(archive)) => archive,
            Ok(None) => return Ok(Fin::Arretee),
            Err(erreur) => {
                dire(0, EtatEtape::Erreur, erreur.clone(), None);
                return Err(erreur);
            }
        };
        let taille = fs::metadata(&archive).map(|m| m.len() >> 20).unwrap_or(0);
        dire(0, EtatEtape::Fait, format!("{taille} Mo"), Some(1.0));

        dire(1, EtatEtape::EnCours, String::new(), None);
        let disque = options.dossier.join("disque");
        let import = fs::create_dir_all(&disque)
            .map_err(|e| e.to_string())
            .and_then(|_| {
                commande("wsl.exe")
                    .args(["--import", NOM_DISTRIBUTION])
                    .arg(&disque)
                    .arg(&archive)
                    .args(["--version", "2"])
                    .output()
                    .map_err(|e| e.to_string())
            });
        let _ = fs::remove_file(&archive);
        match import {
            Ok(sortie) if sortie.status.success() => {
                importe_ici = true;
                dire(1, EtatEtape::Fait, String::new(), None)
            }
            Ok(sortie) => {
                let erreur = format!("WSL a refusé l'import : {}", texte_erreur(&sortie));
                dire(1, EtatEtape::Erreur, erreur.clone(), None);
                return Err(erreur);
            }
            Err(e) => {
                let erreur = format!("wsl.exe ne s'est pas lancé : {e}");
                dire(1, EtatEtape::Erreur, erreur.clone(), None);
                return Err(erreur);
            }
        }
    }

    if arrete() {
        return defaire(importe_ici, &options.dossier);
    }

    match options.memoire_go {
        Some(go) => {
            let ligne = format!("memory={go}GB");
            dire(2, EtatEtape::EnCours, ligne.clone(), None);
            if let Err(erreur) = regler_memoire(go) {
                dire(2, EtatEtape::Erreur, erreur.clone(), None);
                return Err(erreur);
            }
            // La nouvelle limite ne vaut qu'après un redémarrage de WSL2.
            let _ = lancer("wsl.exe", &["--shutdown"]);
            dire(2, EtatEtape::Fait, ligne, None);
        }
        None => dire(2, EtatEtape::Saute, String::new(), None),
    }

    if arrete() {
        return defaire(importe_ici, &options.dossier);
    }

    dire(3, EtatEtape::EnCours, String::new(), None);
    match dans_debian(&["uname", "-r"]) {
        Some(sortie) if sortie.status.success() => {
            dire(
                3,
                EtatEtape::Fait,
                format!("noyau {}", texte_sortie(&sortie).trim()),
                None,
            );
            Ok(Fin::Installe)
        }
        Some(sortie) => {
            let erreur = format!("Debian ne démarre pas : {}", texte_erreur(&sortie));
            dire(3, EtatEtape::Erreur, erreur.clone(), None);
            Err(erreur)
        }
        None => {
            let erreur = "wsl.exe ne s'est pas lancé.".to_string();
            dire(3, EtatEtape::Erreur, erreur.clone(), None);
            Err(erreur)
        }
    }
}

/// Arrêté par l'utilisateur : ce que cette installation a mis dans WSL2
/// repart. La mémoire de WSL2, s'il a dit oui, reste réglée.
fn defaire(importe_ici: bool, dossier: &Path) -> Result<Fin, String> {
    if importe_ici {
        retirer(dossier)?;
    }
    Ok(Fin::Arretee)
}

pub(crate) fn retirer(dossier: &Path) -> Result<(), String> {
    if debian_installe() {
        let sortie = commande("wsl.exe")
            .args(["--unregister", NOM_DISTRIBUTION])
            .output()
            .map_err(|e| format!("wsl.exe ne s'est pas lancé : {e}"))?;
        if !sortie.status.success() {
            return Err(format!(
                "WSL n'a pas retiré Debian : {}",
                texte_erreur(&sortie)
            ));
        }
    }
    let _ = fs::remove_dir_all(dossier.join("disque"));
    for reste in [
        "debian.telechargement",
        "debian.tar.gz",
        "debian.tar.xz",
        "debian.tar",
    ] {
        let _ = fs::remove_file(dossier.join(reste));
    }
    Ok(())
}

pub(crate) fn installer_wsl() -> Result<(), String> {
    // wsl.exe demande lui-même l'accord de l'administrateur.
    let sortie = commande("wsl.exe")
        .args(["--install", "--no-distribution"])
        .output()
        .map_err(|e| format!("wsl.exe ne s'est pas lancé : {e}"))?;
    if sortie.status.success() {
        Ok(())
    } else {
        Err(texte_erreur(&sortie))
    }
}

fn agent() -> ureq::Agent {
    use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
    // Les certificats de Windows, comme le navigateur : ils suivent aussi
    // ceux qu'ajoute un réseau d'entreprise ou d'école.
    ureq::Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .into()
}

fn lire_texte(agent: &ureq::Agent, url: &str) -> Result<String, String> {
    let reponse = agent
        .get(url)
        .call()
        .map_err(|e| format!("La liste des images ne répond pas : {e}"))?;
    reponse
        .into_body()
        .read_to_string()
        .map_err(|e| format!("La liste des images est incomplète : {e}"))
}

fn telecharger(
    agent: &ureq::Agent,
    image: &Image,
    dossier: &Path,
    arret: &AtomicBool,
    progres: &mut dyn FnMut(u64, Option<u64>),
) -> Result<Option<PathBuf>, String> {
    let reponse = agent
        .get(&image.url)
        .call()
        .map_err(|e| format!("Le téléchargement de Debian a échoué : {e}"))?;
    let total = reponse
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    let mut lecteur = reponse.into_body().into_reader();

    let provisoire = dossier.join("debian.telechargement");
    let mut fichier =
        fs::File::create(&provisoire).map_err(|e| format!("Écriture impossible : {e}"))?;
    let mut empreinte = Sha256::new();
    let mut tampon = vec![0u8; 1 << 16];
    let mut debut = Vec::with_capacity(8);
    let (mut recus, mut dernier) = (0u64, 0u64);
    loop {
        let lus = lecteur
            .read(&mut tampon)
            .map_err(|e| format!("Le téléchargement s'est coupé : {e}"))?;
        if lus == 0 {
            break;
        }
        if arret.load(Ordering::Relaxed) {
            drop(fichier);
            let _ = fs::remove_file(&provisoire);
            return Ok(None);
        }
        let morceau = &tampon[..lus];
        if debut.len() < 8 {
            debut.extend_from_slice(&morceau[..lus.min(8 - debut.len())]);
        }
        empreinte.update(morceau);
        fichier
            .write_all(morceau)
            .map_err(|e| format!("Écriture impossible : {e}"))?;
        recus += lus as u64;
        if recus - dernier >= 1 << 20 {
            dernier = recus;
            progres(recus, total);
        }
    }
    drop(fichier);

    let obtenue: String = empreinte
        .finalize()
        .iter()
        .map(|o| format!("{o:02x}"))
        .collect();
    if obtenue != image.sha256 {
        let _ = fs::remove_file(&provisoire);
        return Err(
            "L'image téléchargée ne correspond pas à son empreinte. Elle a été effacée.".into(),
        );
    }
    let archive = dossier.join(format!("debian.{}", extension(&debut)));
    fs::rename(&provisoire, &archive).map_err(|e| format!("Écriture impossible : {e}"))?;
    Ok(Some(archive))
}

fn regler_memoire(go: u32) -> Result<(), String> {
    let fichier = fichier_wslconfig().ok_or("Le dossier de ton compte Windows est introuvable.")?;
    let avant = fs::read_to_string(&fichier).unwrap_or_default();
    if fichier.exists() {
        let copie = fichier.with_file_name(".wslconfig.avant-opn-display");
        if !copie.exists() {
            fs::copy(&fichier, &copie)
                .map_err(|e| format!("Copie de .wslconfig impossible : {e}"))?;
        }
    }
    fs::write(&fichier, wslconfig::avec_memoire(&avant, go))
        .map_err(|e| format!("Écriture de .wslconfig impossible : {e}"))
}
