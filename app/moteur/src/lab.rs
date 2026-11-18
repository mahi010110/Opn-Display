//! Un lab : ses machines, ses câbles, et ce qu'il faut pour le faire tourner
//! dans Incus.
//!
//! Chaque lab a son projet Incus. Un câble est une paire veth, posée par
//! l'app là où tournent les machines, dont chaque bout entre dans une
//! machine comme une carte réseau. Elle passe tout ce qu'un vrai câble
//! passe, et aucun pont ni pare-feu ne se met entre les deux.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::incus::{self, IMAGE, Instance, incus, incus_avec};
use crate::systeme;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Sorte {
    Poste,
    Serveur,
    Routeur,
    Switch,
    ParFeu,
    Stockage,
}

impl Sorte {
    /// Le mot de l'IA et de l'utilisateur.
    pub fn mot(self) -> &'static str {
        match self {
            Sorte::Poste => "poste",
            Sorte::Serveur => "serveur",
            Sorte::Routeur => "routeur",
            Sorte::Switch => "switch",
            Sorte::ParFeu => "pare-feu",
            Sorte::Stockage => "stockage",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Interface {
    pub nom: String,
    /// L'adresse posée au départ, avec son masque : « 10.0.1.10/24 ».
    pub adresse: Option<String>,
    pub passerelle: Option<String>,
}

/// Une route posée au départ : vers `destination`, passer par `passerelle`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub destination: String,
    pub passerelle: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    pub nom: String,
    pub sorte: Sorte,
    /// La place sur le schéma, en pour cent de sa largeur et de sa hauteur.
    pub x: f64,
    pub y: f64,
    pub interfaces: Vec<Interface>,
    #[serde(default)]
    pub routes: Vec<Route>,
    /// Elle fait passer les paquets d'un réseau à l'autre.
    #[serde(default)]
    pub transfere: bool,
    /// La mémoire qu'elle prendra, en octets, pour la montrer avant le
    /// lancement.
    #[serde(default)]
    pub memoire_prevue: u64,
}

/// Ce que prend un conteneur Debian du lab à vide : 13 Mo mesurés
/// (`docs/mesures.md`, partie 15), arrondis au-dessus.
pub const MEMOIRE_CONTENEUR: u64 = 16 * 1024 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Bout {
    pub machine: String,
    pub interface: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Cable {
    pub de: Bout,
    pub vers: Bout,
    /// Le réseau que porte le câble, écrit sur le schéma.
    pub reseau: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Lab {
    pub id: String,
    pub nom: String,
    pub machines: Vec<Machine>,
    pub cables: Vec<Cable>,
    /// Il entre dans le nom des câbles, qui doit tenir en 15 caractères.
    pub numero: u8,
}

fn interface(nom: &str, adresse: &str, passerelle: Option<&str>) -> Interface {
    Interface {
        nom: nom.into(),
        adresse: Some(adresse.into()),
        passerelle: passerelle.map(str::to_string),
    }
}

fn bout(machine: &str, interface: &str) -> Bout {
    Bout {
        machine: machine.into(),
        interface: interface.into(),
    }
}

/// Le premier lab : deux postes, chacun dans son réseau, et un routeur entre
/// les deux. Les adresses sont posées : pc1 joint pc2 dès le départ.
pub fn premier_lab() -> Lab {
    Lab {
        id: "premier-lab".into(),
        nom: "Premier lab".into(),
        numero: 1,
        machines: vec![
            Machine {
                nom: "pc1".into(),
                sorte: Sorte::Poste,
                x: 16.0,
                y: 46.0,
                interfaces: vec![interface("eth0", "10.0.1.10/24", Some("10.0.1.1"))],
                routes: Vec::new(),
                transfere: false,
                memoire_prevue: MEMOIRE_CONTENEUR,
            },
            Machine {
                nom: "routeur".into(),
                sorte: Sorte::Routeur,
                x: 50.0,
                y: 46.0,
                interfaces: vec![
                    interface("eth0", "10.0.1.1/24", None),
                    interface("eth1", "10.0.2.1/24", None),
                ],
                routes: Vec::new(),
                transfere: true,
                memoire_prevue: MEMOIRE_CONTENEUR,
            },
            Machine {
                nom: "pc2".into(),
                sorte: Sorte::Poste,
                x: 84.0,
                y: 46.0,
                interfaces: vec![interface("eth0", "10.0.2.10/24", Some("10.0.2.1"))],
                routes: Vec::new(),
                transfere: false,
                memoire_prevue: MEMOIRE_CONTENEUR,
            },
        ],
        cables: vec![
            Cable {
                de: bout("pc1", "eth0"),
                vers: bout("routeur", "eth0"),
                reseau: Some("10.0.1.0/24".into()),
            },
            Cable {
                de: bout("routeur", "eth1"),
                vers: bout("pc2", "eth0"),
                reseau: Some("10.0.2.0/24".into()),
            },
        ],
    }
}

/// Le projet Incus du lab.
pub fn projet(lab: &Lab) -> String {
    format!("{PREFIXE}{}", lab.id)
}

/// Le début du nom de tous les projets Incus de l'app.
pub const PREFIXE: &str = "opn-display-";

/// Les labs construits, un fichier JSON chacun dans `dossier`, du plus
/// ancien au plus récent. Un fichier illisible est laissé de côté.
pub fn labs_ranges(dossier: &Path) -> Vec<Lab> {
    let Ok(entrees) = fs::read_dir(dossier) else {
        return Vec::new();
    };
    let mut labs: Vec<Lab> = entrees
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| fs::read_to_string(e.path()).ok())
        .filter_map(|texte| serde_json::from_str(&texte).ok())
        .collect();
    labs.sort_by_key(|l| l.numero);
    labs
}

/// Range un lab construit dans `dossier`.
pub fn ranger(dossier: &Path, lab: &Lab) -> Result<(), String> {
    fs::create_dir_all(dossier).map_err(|e| format!("Le lab ne se range pas : {e}"))?;
    let texte = serde_json::to_string_pretty(lab).map_err(|e| e.to_string())?;
    fs::write(dossier.join(format!("{}.json", lab.id)), texte)
        .map_err(|e| format!("Le lab ne se range pas : {e}"))
}

/// Retire le fichier d'un lab construit.
pub fn oublier(dossier: &Path, lab: &Lab) {
    let _ = fs::remove_file(dossier.join(format!("{}.json", lab.id)));
}

/// Le plus petit numéro libre : le Premier lab a le 1, et le numéro entre
/// dans le nom des câbles.
pub fn numero_libre(labs: &[Lab]) -> Option<u8> {
    (2..=u8::MAX).find(|n| labs.iter().all(|l| l.numero != *n))
}

/// Donne au lab proposé son numéro et son identifiant.
pub fn baptiser(mut lab: Lab, numero: u8) -> Lab {
    lab.numero = numero;
    lab.id = format!("lab-{numero}");
    lab
}

/// Retire le lab de la machine : ses machines, son projet Incus et ses
/// câbles. Rien ne revient.
pub fn supprimer(lab: &Lab) -> Result<(), String> {
    let projet = projet(lab);
    if incus::projet_existe(&projet) {
        for instance in incus::instances(&projet)? {
            incus(&["delete", &instance.nom, "--force", "--project", &projet])
                .map_err(|e| format!("{} ne se retire pas : {e}", instance.nom))?;
        }
        let _ = incus(&[
            "profile",
            "device",
            "remove",
            "default",
            "root",
            "--project",
            &projet,
        ]);
        incus(&["project", "delete", &projet])
            .map_err(|e| format!("Le lab ne se retire pas : {e}"))?;
    }
    // Retirer un bout d'une paire veth retire aussi l'autre.
    let ici = interfaces_ici().unwrap_or_default();
    for rang in 0..lab.cables.len() {
        let paire = paire(lab, rang);
        if ici.contains(&paire.de) {
            let _ = systeme::lancer("ip", &["link", "del", &paire.de]);
        } else if ici.contains(&paire.vers) {
            let _ = systeme::lancer("ip", &["link", "del", &paire.vers]);
        }
    }
    Ok(())
}

/// Les deux bouts d'un câble, tels qu'ils s'appellent hors des machines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Paire {
    pub de: String,
    pub vers: String,
    pub mac_de: String,
    pub mac_vers: String,
}

/// Les adresses MAC sont fixes : elles restent les mêmes d'une session à
/// l'autre, comme sur une vraie carte.
pub(crate) fn paire(lab: &Lab, rang: usize) -> Paire {
    let cable = rang + 1;
    Paire {
        de: format!("od{}-c{cable}a", lab.numero),
        vers: format!("od{}-c{cable}b", lab.numero),
        mac_de: format!("02:4f:44:{:02x}:{cable:02x}:01", lab.numero),
        mac_vers: format!("02:4f:44:{:02x}:{cable:02x}:02", lab.numero),
    }
}

/// Le bout de câble branché sur cette interface, s'il y en a un.
pub(crate) fn bout_branche(lab: &Lab, machine: &str, interface: &str) -> Option<String> {
    lab.cables.iter().enumerate().find_map(|(rang, cable)| {
        let paire = paire(lab, rang);
        if cable.de.machine == machine && cable.de.interface == interface {
            Some(paire.de)
        } else if cable.vers.machine == machine && cable.vers.interface == interface {
            Some(paire.vers)
        } else {
            None
        }
    })
}

const ENTETE: &str = "# Posé par Opn-Display au départ du lab. Tu peux le changer.\n";

/// Le réglage réseau d'une interface, pour systemd-networkd, qui règle le
/// réseau dans l'image Debian d'Incus. Les routes de la machine vont dans
/// l'interface du réseau où se trouve leur passerelle.
pub(crate) fn reglage_reseau(machine: &Machine, interface: &Interface) -> Option<String> {
    if machine.sorte == Sorte::Switch {
        return Some(format!(
            "{ENTETE}[Match]\nName={}\n\n[Link]\nRequiredForOnline=no\n\n[Network]\nBridge={PONT}\n",
            interface.nom
        ));
    }
    let adresse = interface.adresse.as_ref()?;
    let mut texte = format!(
        "{ENTETE}[Match]\nName={}\n\n\
         [Link]\nRequiredForOnline=no\n\n\
         [Network]\nConfigureWithoutCarrier=yes\nAddress={adresse}\n",
        interface.nom
    );
    if let Some(passerelle) = &interface.passerelle {
        texte.push_str(&format!("Gateway={passerelle}\n"));
    }
    for route in &machine.routes {
        if crate::reseau::dans_le_reseau(&route.passerelle, adresse) {
            texte.push_str(&format!(
                "\n[Route]\nDestination={}\nGateway={}\n",
                route.destination, route.passerelle
            ));
        }
    }
    Some(texte)
}

/// Le pont d'un switch : tous ses ports y entrent.
const PONT: &str = "br0";

fn reglages_du_pont() -> [(&'static str, String); 2] {
    [
        (
            "br0.netdev",
            format!("{ENTETE}[NetDev]\nName={PONT}\nKind=bridge\n"),
        ),
        (
            "br0.network",
            format!(
                "{ENTETE}[Match]\nName={PONT}\n\n[Link]\nRequiredForOnline=no\n\n\
                 [Network]\nConfigureWithoutCarrier=yes\nLinkLocalAddressing=no\n"
            ),
        ),
    ]
}

const TRANSFERT: &str = "# Posé par Opn-Display : cette machine fait passer les paquets \
d'un réseau à l'autre.\nnet.ipv4.ip_forward = 1\n";

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Marche {
    /// Pas encore créée : elle le sera au premier lancement.
    Absente,
    Arretee,
    EnMarche,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EtatMachine {
    pub nom: String,
    pub marche: Marche,
    /// Ce qu'elle prend en mémoire, en octets, quand elle tourne.
    pub memoire: Option<u64>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EtatLab {
    pub lab: Lab,
    pub machines: Vec<EtatMachine>,
    /// Quand l'utilisateur l'a ouvert pour la dernière fois, en secondes
    /// depuis 1970. C'est l'app qui le sait, pas Incus.
    pub ouvert: Option<u64>,
}

fn instances(lab: &Lab) -> Result<Vec<Instance>, String> {
    let projet = projet(lab);
    if incus::projet_existe(&projet) {
        incus::instances(&projet)
    } else {
        Ok(Vec::new())
    }
}

fn etat_depuis(lab: &Lab, instances: &[Instance]) -> EtatLab {
    let machines = lab
        .machines
        .iter()
        .map(|m| {
            let instance = instances.iter().find(|i| i.nom == m.nom);
            EtatMachine {
                nom: m.nom.clone(),
                marche: match instance {
                    None => Marche::Absente,
                    Some(i) if i.en_marche => Marche::EnMarche,
                    Some(_) => Marche::Arretee,
                },
                memoire: instance.and_then(|i| i.memoire),
            }
        })
        .collect();
    EtatLab {
        lab: lab.clone(),
        machines,
        ouvert: None,
    }
}

/// Où en sont les machines du lab.
pub fn etat(lab: &Lab) -> Result<EtatLab, String> {
    Ok(etat_depuis(lab, &instances(lab)?))
}

/// Lance tout le lab. Les machines qui manquent sont créées d'abord, ce que
/// `avance` raconte.
pub fn lancer(lab: &Lab, avance: &mut dyn FnMut(&str)) -> Result<EtatLab, String> {
    let noms: Vec<&str> = lab.machines.iter().map(|m| m.nom.as_str()).collect();
    demarrer(lab, &noms, avance)
}

/// Démarre une seule machine, créée d'abord si elle manque.
pub fn demarrer_machine(
    lab: &Lab,
    nom: &str,
    avance: &mut dyn FnMut(&str),
) -> Result<EtatLab, String> {
    connue(lab, nom)?;
    demarrer(lab, &[nom], avance)
}

fn demarrer(lab: &Lab, noms: &[&str], avance: &mut dyn FnMut(&str)) -> Result<EtatLab, String> {
    let presentes = preparer(lab, avance)?;
    let a_demarrer: Vec<&str> = noms
        .iter()
        .copied()
        .filter(|nom| !presentes.iter().any(|i| i.nom == *nom && i.en_marche))
        .collect();
    if !a_demarrer.is_empty() {
        avance("Démarrage");
        let projet = projet(lab);
        let mut args = vec!["start"];
        args.extend(&a_demarrer);
        args.extend(["--project", projet.as_str()]);
        incus(&args).map_err(|e| format!("Le lab ne démarre pas : {e}"))?;
    }
    etat(lab)
}

/// Pose le projet, les câbles et les machines qui manquent. Rend les
/// machines telles qu'elles étaient avant.
fn preparer(lab: &Lab, avance: &mut dyn FnMut(&str)) -> Result<Vec<Instance>, String> {
    let projet = projet(lab);
    if !incus::projet_existe(&projet) {
        incus::creer_projet(&projet)?;
    }
    let presentes = incus::instances(&projet)?;
    poser_les_cables(lab, &presentes)?;
    for machine in &lab.machines {
        if !presentes.iter().any(|i| i.nom == machine.nom) {
            avance(&format!("Création de {}", machine.nom));
            creer_machine(lab, machine)?;
        }
    }
    Ok(presentes)
}

/// Arrête toutes les machines du lab, proprement, puis de force pour celles
/// qui traînent.
pub fn arreter(lab: &Lab) -> Result<EtatLab, String> {
    let noms: Vec<String> = instances(lab)?
        .into_iter()
        .filter(|i| i.en_marche)
        .map(|i| i.nom)
        .collect();
    arreter_les(lab, &noms)?;
    etat(lab)
}

pub fn arreter_machine(lab: &Lab, nom: &str) -> Result<EtatLab, String> {
    connue(lab, nom)?;
    arreter_les(lab, &[nom.to_string()])?;
    etat(lab)
}

fn arreter_les(lab: &Lab, noms: &[String]) -> Result<(), String> {
    if noms.is_empty() {
        return Ok(());
    }
    let projet = projet(lab);
    let mut args: Vec<&str> = vec!["stop"];
    args.extend(noms.iter().map(String::as_str));
    args.extend(["--project", projet.as_str()]);
    let mut doucement = args.clone();
    doucement.extend(["--timeout", "20"]);
    if incus(&doucement).is_err() {
        args.push("--force");
        incus(&args).map_err(|e| format!("Le lab ne s'arrête pas : {e}"))?;
    }
    Ok(())
}

/// Lance une commande sh en root dans une machine qui tourne. Elle est
/// coupée dans la machine au bout de `secondes` : arrêter seulement incus
/// laisserait tourner un ping sans fin. Rend le code de sortie (124 si elle
/// a été coupée, rien si incus lui-même a dû être arrêté) et la sortie.
pub(crate) fn commande_dans(
    lab: &Lab,
    machine: &str,
    commande: &str,
    secondes: u64,
) -> Result<(Option<i32>, String), String> {
    let projet = projet(lab);
    let delai = secondes.to_string();
    systeme::lancer_borne(
        "incus",
        &[
            "exec",
            machine,
            "--project",
            &projet,
            "--",
            "timeout",
            "-k",
            "2",
            &delai,
            "sh",
            "-c",
            commande,
        ],
        std::time::Duration::from_secs(secondes + 10),
    )
}

/// Démarre le lab s'il ne tourne pas en entier, puis attend que chaque
/// machine ait fini de démarrer : avant, son réseau n'est pas encore posé
/// et un test ou une commande échouerait pour rien.
pub fn lancer_et_attendre(lab: &Lab, avance: &mut dyn FnMut(&str)) -> Result<(), String> {
    let en_marche = etat(lab)?
        .machines
        .iter()
        .all(|m| m.marche == Marche::EnMarche);
    if en_marche {
        return Ok(());
    }
    lancer(lab, avance)?;
    avance("Démarrage des machines");
    // Juste après le démarrage, systemctl ne joint pas encore systemd :
    // on réessaie. Une machine « degraded » a fini de démarrer aussi.
    let attente = "until systemctl is-system-running --wait >/dev/null 2>&1 \
                   || [ \"$(systemctl is-system-running 2>/dev/null)\" = degraded ]; \
                   do sleep 0.3; done";
    for machine in &lab.machines {
        let _ = commande_dans(lab, &machine.nom, attente, 30);
    }
    Ok(())
}

/// Arrête tout de suite les machines en marche. Pour la fermeture de l'app.
pub fn arreter_vite(lab: &Lab) {
    let projet = projet(lab);
    if incus::projet_existe(&projet) {
        let _ = incus(&["stop", "--all", "--force", "--project", &projet]);
    }
}

/// Remet les machines comme au premier lancement. Un lab qui tournait
/// repart.
pub fn remettre_a_zero(lab: &Lab, avance: &mut dyn FnMut(&str)) -> Result<EtatLab, String> {
    let projet = projet(lab);
    let presentes = instances(lab)?;
    let tournait = presentes.iter().any(|i| i.en_marche);
    for instance in &presentes {
        avance(&format!("Retrait de {}", instance.nom));
        incus(&["delete", &instance.nom, "--force", "--project", &projet])
            .map_err(|e| format!("{} ne se retire pas : {e}", instance.nom))?;
    }
    if tournait {
        lancer(lab, avance)
    } else {
        preparer(lab, avance)?;
        etat(lab)
    }
}

fn connue(lab: &Lab, nom: &str) -> Result<(), String> {
    if lab.machines.iter().any(|m| m.nom == nom) {
        Ok(())
    } else {
        Err(format!("{nom} n'est pas une machine de ce lab."))
    }
}

fn creer_machine(lab: &Lab, machine: &Machine) -> Result<(), String> {
    let projet = projet(lab);
    let nom = machine.nom.as_str();
    // Sans ce réglage, Incus relance au démarrage de WSL les machines qui
    // tournaient, avant que l'app n'ait reposé leurs câbles.
    incus(&[
        "create",
        IMAGE,
        nom,
        "--project",
        &projet,
        "-c",
        "boot.autostart=false",
    ])
    .map_err(|e| format!("{nom} ne se crée pas : {e}"))?;

    for interface in &machine.interfaces {
        if let Some(bout) = bout_branche(lab, nom, &interface.nom) {
            incus(&[
                "config",
                "device",
                "add",
                nom,
                &interface.nom,
                "nic",
                "nictype=physical",
                &format!("parent={bout}"),
                &format!("name={}", interface.nom),
                "--project",
                &projet,
            ])
            .map_err(|e| format!("Le câble de {nom} ne se branche pas : {e}"))?;
        }
        let fichier = format!("{nom}/etc/systemd/network/{}.network", interface.nom);
        match reglage_reseau(machine, interface) {
            Some(reglage) => ecrire(&projet, &fichier, &reglage)?,
            // L'image fait du DHCP sur eth0 : sur un câble du lab, ce serait
            // du bruit que personne n'a demandé.
            None => {
                let _ = incus(&["file", "delete", &fichier, "--project", &projet]);
            }
        }
    }
    if machine.sorte == Sorte::Switch {
        for (fichier, contenu) in reglages_du_pont() {
            ecrire(
                &projet,
                &format!("{nom}/etc/systemd/network/{fichier}"),
                &contenu,
            )?;
        }
    }
    if machine.transfere {
        ecrire(
            &projet,
            &format!("{nom}/etc/sysctl.d/90-opn-display.conf"),
            TRANSFERT,
        )?;
    }
    Ok(())
}

/// Écrit un fichier dans une machine arrêtée.
fn ecrire(projet: &str, fichier: &str, contenu: &str) -> Result<(), String> {
    // Sans --mode, un fichier lu sur l'entrée naît en 0600, et
    // systemd-networkd ne peut plus le lire (mesuré).
    incus_avec(
        &[
            "file",
            "push",
            "-",
            fichier,
            "--project",
            projet,
            "--mode",
            "0644",
            "--uid",
            "0",
            "--gid",
            "0",
        ],
        contenu.as_bytes(),
    )
    .map(|_| ())
    .map_err(|e| format!("Le réglage {fichier} ne s'écrit pas : {e}"))
}

#[derive(Deserialize)]
struct Lien {
    ifname: String,
}

fn interfaces_ici() -> Result<Vec<String>, String> {
    let liste = systeme::lancer("ip", &["-j", "link", "show"])?;
    let liens: Vec<Lien> = serde_json::from_str(&liste)
        .map_err(|e| format!("La liste des interfaces est illisible : {e}"))?;
    Ok(liens.into_iter().map(|l| l.ifname).collect())
}

/// Les câbles qui manquent : ceux dont un bout n'est ni ici, ni dans une
/// machine qui tourne. Après un redémarrage de WSL, ils manquent tous.
pub(crate) fn cables_manquants(lab: &Lab, presentes: &[Instance], ici: &[String]) -> Vec<Paire> {
    let tourne = |machine: &str| presentes.iter().any(|i| i.nom == machine && i.en_marche);
    let la = |bout: &str| ici.iter().any(|nom| nom == bout);
    lab.cables
        .iter()
        .enumerate()
        .filter_map(|(rang, cable)| {
            let paire = paire(lab, rang);
            let manque = (!tourne(&cable.de.machine) && !la(&paire.de))
                || (!tourne(&cable.vers.machine) && !la(&paire.vers));
            manque.then_some(paire)
        })
        .collect()
}

fn poser_les_cables(lab: &Lab, presentes: &[Instance]) -> Result<(), String> {
    for paire in cables_manquants(lab, presentes, &interfaces_ici()?) {
        systeme::lancer(
            "ip",
            &[
                "link",
                "add",
                &paire.de,
                "address",
                &paire.mac_de,
                "type",
                "veth",
                "peer",
                "name",
                &paire.vers,
                "address",
                &paire.mac_vers,
            ],
        )
        .map_err(|e| {
            if e.contains("Operation not permitted") {
                "Pour poser les câbles, l'app doit tourner en root sous Linux, pour l'instant."
                    .to_string()
            } else {
                format!("Un câble ne se pose pas : {e}")
            }
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instance(nom: &str, en_marche: bool) -> Instance {
        Instance {
            nom: nom.into(),
            en_marche,
            memoire: None,
        }
    }

    #[test]
    fn le_premier_lab_tient_debout() {
        let lab = premier_lab();
        for cable in &lab.cables {
            for bout in [&cable.de, &cable.vers] {
                let machine = lab.machines.iter().find(|m| m.nom == bout.machine);
                assert!(
                    machine.is_some_and(|m| m.interfaces.iter().any(|i| i.nom == bout.interface)),
                    "{}:{} n'existe pas",
                    bout.machine,
                    bout.interface
                );
            }
        }
        assert_eq!(projet(&lab), "opn-display-premier-lab");
    }

    #[test]
    fn les_cables_ont_des_noms_courts_et_fixes() {
        let lab = premier_lab();
        let paire = paire(&lab, 1);
        assert_eq!(paire.de, "od1-c2a");
        assert_eq!(paire.vers, "od1-c2b");
        assert_eq!(paire.mac_de, "02:4f:44:01:02:01");
        assert!(paire.de.len() <= 15);
        assert_eq!(
            bout_branche(&lab, "routeur", "eth1").as_deref(),
            Some("od1-c2a")
        );
        assert_eq!(
            bout_branche(&lab, "pc2", "eth0").as_deref(),
            Some("od1-c2b")
        );
        assert_eq!(bout_branche(&lab, "pc2", "eth1"), None);
    }

    #[test]
    fn ecrit_le_reglage_reseau() {
        let lab = premier_lab();
        let (pc1, routeur) = (&lab.machines[0], &lab.machines[1]);
        let reglage = reglage_reseau(pc1, &pc1.interfaces[0]).unwrap();
        assert!(reglage.contains("Name=eth0\n"));
        assert!(reglage.contains("Address=10.0.1.10/24\n"));
        assert!(reglage.contains("Gateway=10.0.1.1\n"));
        assert!(reglage.contains("RequiredForOnline=no\n"));
        let reglage = reglage_reseau(routeur, &routeur.interfaces[1]).unwrap();
        assert!(!reglage.contains("Gateway"));
        let vide = Interface {
            nom: "eth2".into(),
            adresse: None,
            passerelle: None,
        };
        assert_eq!(reglage_reseau(pc1, &vide), None);
    }

    #[test]
    fn range_et_relit_les_labs() {
        let dossier = std::env::temp_dir().join(format!("opn-display-labs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dossier);
        assert!(labs_ranges(&dossier).is_empty());
        let deux = baptiser(premier_lab(), 3);
        let trois = baptiser(premier_lab(), 2);
        ranger(&dossier, &deux).unwrap();
        ranger(&dossier, &trois).unwrap();
        fs::write(dossier.join("abime.json"), "{").unwrap();
        let lus = labs_ranges(&dossier);
        assert_eq!(
            lus.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
            ["lab-2", "lab-3"]
        );
        assert_eq!(lus[1], deux);
        oublier(&dossier, &deux);
        assert_eq!(labs_ranges(&dossier).len(), 1);
        let _ = fs::remove_dir_all(&dossier);
    }

    #[test]
    fn donne_le_plus_petit_numero_libre() {
        assert_eq!(numero_libre(&[]), Some(2));
        let labs = [baptiser(premier_lab(), 2), baptiser(premier_lab(), 4)];
        assert_eq!(numero_libre(&labs), Some(3));
        assert_eq!(paire(&labs[1], 9).de, "od4-c10a");
    }

    #[test]
    fn une_route_va_dans_le_reseau_de_sa_passerelle() {
        let mut routeur = premier_lab().machines[1].clone();
        routeur.routes = vec![Route {
            destination: "10.0.3.0/24".into(),
            passerelle: "10.0.2.2".into(),
        }];
        let eth0 = reglage_reseau(&routeur, &routeur.interfaces[0]).unwrap();
        let eth1 = reglage_reseau(&routeur, &routeur.interfaces[1]).unwrap();
        assert!(!eth0.contains("[Route]"));
        assert!(eth1.contains("[Route]\nDestination=10.0.3.0/24\nGateway=10.0.2.2\n"));
    }

    #[test]
    fn les_ports_d_un_switch_entrent_dans_son_pont() {
        let mut switch = premier_lab().machines[1].clone();
        switch.sorte = Sorte::Switch;
        let port = Interface {
            nom: "eth3".into(),
            adresse: None,
            passerelle: None,
        };
        let reglage = reglage_reseau(&switch, &port).unwrap();
        assert!(reglage.contains("Name=eth3\n"));
        assert!(reglage.contains("Bridge=br0\n"));
        assert!(!reglage.contains("Address"));
        let [netdev, reseau] = reglages_du_pont();
        assert!(netdev.1.contains("Kind=bridge"));
        assert!(reseau.1.contains("Name=br0"));
    }

    #[test]
    fn retrouve_les_cables_qui_manquent() {
        let lab = premier_lab();
        let tous: Vec<String> = ["od1-c1a", "od1-c1b", "od1-c2a", "od1-c2b"]
            .map(String::from)
            .to_vec();
        let arretees = [
            instance("pc1", false),
            instance("routeur", false),
            instance("pc2", false),
        ];
        assert!(cables_manquants(&lab, &arretees, &tous).is_empty());
        assert_eq!(cables_manquants(&lab, &arretees, &[]).len(), 2);

        // pc1 tourne : son bout est chez lui, l'autre doit être ici.
        let pc1 = [
            instance("pc1", true),
            instance("routeur", false),
            instance("pc2", false),
        ];
        let sans_pc1: Vec<String> = tous[1..].to_vec();
        assert!(cables_manquants(&lab, &pc1, &sans_pc1).is_empty());
        let manquants = cables_manquants(&lab, &pc1, &tous[2..]);
        assert_eq!(manquants, vec![paire(&lab, 0)]);
    }

    #[test]
    fn dit_l_etat_de_chaque_machine() {
        let lab = premier_lab();
        let etat = etat_depuis(
            &lab,
            &[
                Instance {
                    nom: "pc1".into(),
                    en_marche: true,
                    memoire: Some(14_000_000),
                },
                instance("routeur", false),
            ],
        );
        let marches: Vec<Marche> = etat.machines.iter().map(|m| m.marche).collect();
        assert_eq!(
            marches,
            [Marche::EnMarche, Marche::Arretee, Marche::Absente]
        );
        assert_eq!(etat.machines[0].memoire, Some(14_000_000));
    }
}
