//! Un lab que l'IA propose : ce qu'elle décrit, vérifié par l'app, puis posé
//! sur le schéma. L'IA ne nomme ni les interfaces ni les câbles : l'app les
//! déduit, dans l'ordre des câbles.

use std::collections::HashMap;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

use crate::lab::{Bout, Cable, Interface, Lab, MEMOIRE_CONTENEUR, Machine, Route, Sorte};
use crate::reseau::Adresse;

/// Assez pour un lab de cours, et un schéma qui reste lisible.
pub const MACHINES_MAX: usize = 12;
const PORTS_MAX: usize = 16;
const INTERFACES_MAX: usize = 8;
/// Un nom de machine devient son nom d'hôte.
const NOM_MAX: usize = 15;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Proposition {
    pub nom: String,
    #[serde(default)]
    pub machines: Vec<MachineProposee>,
    #[serde(default)]
    pub cables: Vec<CablePropose>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MachineProposee {
    pub nom: String,
    pub sorte: String,
    /// La passerelle par défaut.
    #[serde(default)]
    pub passerelle: Option<String>,
    #[serde(default)]
    pub routes: Vec<Route>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BoutPropose {
    pub machine: String,
    #[serde(default)]
    pub adresse: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CablePropose {
    pub de: BoutPropose,
    pub vers: BoutPropose,
}

/// Vérifie la proposition et en fait un lab, posé sur le schéma. Chaque
/// refus dit quoi corriger : il repart vers l'IA.
pub fn construire(proposition: &Proposition) -> Result<Lab, String> {
    let nom = proposition.nom.trim();
    if nom.is_empty() {
        return Err("Le lab n'a pas de nom.".into());
    }
    let nombre = proposition.machines.len();
    if nombre == 0 {
        return Err("Le lab n'a aucune machine.".into());
    }
    if nombre > MACHINES_MAX {
        return Err(format!(
            "{nombre} machines, c'est trop : {MACHINES_MAX} au plus."
        ));
    }
    let mut machines: Vec<Machine> = Vec::new();
    for proposee in &proposition.machines {
        let nom = proposee.nom.trim().to_string();
        valider_nom(&nom)?;
        if machines.iter().any(|m| m.nom == nom) {
            return Err(format!("Deux machines s'appellent {nom}."));
        }
        let sorte = lire_sorte(&proposee.sorte, &nom)?;
        machines.push(Machine {
            nom,
            sorte,
            x: 0.0,
            y: 0.0,
            interfaces: Vec::new(),
            routes: Vec::new(),
            transfere: sorte == Sorte::Routeur,
            memoire_prevue: MEMOIRE_CONTENEUR,
        });
    }

    let mut cables = Vec::new();
    for propose in &proposition.cables {
        if propose.de.machine.trim() == propose.vers.machine.trim() {
            return Err(format!(
                "Un câble relie {} à elle-même.",
                propose.de.machine.trim()
            ));
        }
        let de = brancher(&mut machines, &propose.de)?;
        let vers = brancher(&mut machines, &propose.vers)?;
        cables.push(Cable {
            de,
            vers,
            reseau: None,
        });
    }
    adresses_uniques(&machines)?;
    let reseaux = reseaux(&machines, &cables)?;
    for cable in &mut cables {
        cable.reseau = reseaux
            .get(&(cable.de.machine.clone(), cable.de.interface.clone()))
            .copied()
            .flatten()
            .map(|r| r.to_string());
    }

    for proposee in &proposition.machines {
        let nom = proposee.nom.trim();
        let passerelle = proposee
            .passerelle
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty());
        if let Some(passerelle) = passerelle {
            let ip = passerelle_valide(&machines, &reseaux, nom, passerelle, "La passerelle")?;
            let machine = machines.iter_mut().find(|m| m.nom == nom);
            if let Some(interface) = machine.and_then(|m| {
                m.interfaces.iter_mut().find(|i| {
                    i.adresse
                        .as_deref()
                        .and_then(Adresse::lire)
                        .is_some_and(|a| a.contient(ip))
                })
            }) {
                interface.passerelle = Some(ip.to_string());
            }
        }
        let mut routes = Vec::new();
        for route in &proposee.routes {
            let destination = Adresse::lire(&route.destination).ok_or_else(|| {
                format!(
                    "La destination {} d'une route de {nom} n'est pas un réseau avec son masque, comme 10.0.3.0/24.",
                    route.destination
                )
            })?;
            if destination.reseau() != destination {
                return Err(format!(
                    "La destination {destination} d'une route de {nom} doit être un réseau : {}.",
                    destination.reseau()
                ));
            }
            let ip = passerelle_valide(
                &machines,
                &reseaux,
                nom,
                route.passerelle.trim(),
                "La passerelle d'une route",
            )?;
            routes.push(Route {
                destination: destination.to_string(),
                passerelle: ip.to_string(),
            });
        }
        if let Some(machine) = machines.iter_mut().find(|m| m.nom == nom) {
            machine.routes = routes;
        }
    }

    placer(&mut machines, &cables);
    Ok(Lab {
        id: String::new(),
        nom: nom.to_string(),
        machines,
        cables,
        numero: 0,
    })
}

/// Retire une machine du lab proposé, avec ses câbles, les ports qu'ils
/// prenaient en face, et les passerelles qui passaient par elle.
pub fn retirer(lab: &Lab, nom: &str) -> Lab {
    let mut lab = lab.clone();
    let partantes: Vec<Ipv4Addr> = lab
        .machines
        .iter()
        .filter(|m| m.nom == nom)
        .flat_map(|m| &m.interfaces)
        .filter_map(|i| i.adresse.as_deref().and_then(Adresse::lire))
        .map(|a| a.ip)
        .collect();
    let (partis, restes): (Vec<Cable>, Vec<Cable>) = lab
        .cables
        .into_iter()
        .partition(|c| c.de.machine == nom || c.vers.machine == nom);
    lab.cables = restes;
    lab.machines.retain(|m| m.nom != nom);
    let passe_par_elle = |ip: &str| ip.parse().is_ok_and(|ip| partantes.contains(&ip));
    for machine in &mut lab.machines {
        machine.interfaces.retain(|i| {
            !partis.iter().any(|c| {
                [&c.de, &c.vers]
                    .iter()
                    .any(|b| b.machine == machine.nom && b.interface == i.nom)
            })
        });
        for interface in &mut machine.interfaces {
            if interface.passerelle.as_deref().is_some_and(passe_par_elle) {
                interface.passerelle = None;
            }
        }
        machine.routes.retain(|r| !passe_par_elle(&r.passerelle));
    }
    placer(&mut lab.machines, &lab.cables);
    lab
}

fn valider_nom(nom: &str) -> Result<(), String> {
    let bon = !nom.is_empty()
        && nom.len() <= NOM_MAX
        && nom.starts_with(|c: char| c.is_ascii_lowercase())
        && !nom.ends_with('-')
        && nom
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if bon {
        Ok(())
    } else {
        Err(format!(
            "« {nom} » ne peut pas nommer une machine : des minuscules sans accent, des chiffres \
             et des tirets, {NOM_MAX} au plus, en commençant par une lettre."
        ))
    }
}

fn lire_sorte(texte: &str, machine: &str) -> Result<Sorte, String> {
    match texte.trim().to_lowercase().as_str() {
        "poste" => Ok(Sorte::Poste),
        "serveur" => Ok(Sorte::Serveur),
        "routeur" => Ok(Sorte::Routeur),
        "switch" => Ok(Sorte::Switch),
        "pare-feu" | "parefeu" | "firewall" => Err(format!(
            "{machine} : les pare-feu n'existent pas encore dans l'app. À cette étape : poste, \
             serveur, routeur ou switch. Un routeur Debian peut filtrer avec nftables."
        )),
        autre => Err(format!(
            "{machine} : « {autre} » n'est pas une sorte de machine. À cette étape : poste, \
             serveur, routeur ou switch."
        )),
    }
}

fn brancher(machines: &mut [Machine], bout: &BoutPropose) -> Result<Bout, String> {
    let nom = bout.machine.trim();
    let machine = machines
        .iter_mut()
        .find(|m| m.nom == nom)
        .ok_or_else(|| format!("Un câble mène à {nom}, qui n'est pas dans le lab."))?;
    let switch = machine.sorte == Sorte::Switch;
    let maximum = if switch { PORTS_MAX } else { INTERFACES_MAX };
    if machine.interfaces.len() >= maximum {
        return Err(format!("{nom} a trop de câbles : {maximum} au plus."));
    }
    let texte = bout
        .adresse
        .as_deref()
        .map(str::trim)
        .filter(|a| !a.is_empty());
    let adresse = match texte {
        Some(texte) if switch => {
            return Err(format!(
                "Un switch n'a pas d'adresse : retire {texte} de {nom}."
            ));
        }
        Some(texte) => {
            let lue = Adresse::lire(texte).ok_or_else(|| {
                format!(
                    "{texte}, sur {nom}, n'est pas une adresse avec son masque, comme 10.0.1.10/24."
                )
            })?;
            if !(8..=30).contains(&lue.masque) {
                return Err(format!(
                    "Le masque de {texte}, sur {nom}, doit aller de /8 à /30."
                ));
            }
            if lue.reservee() {
                return Err(format!(
                    "{texte}, sur {nom}, est l'adresse du réseau ou celle de diffusion : prends-en une autre."
                ));
            }
            Some(lue.to_string())
        }
        None => None,
    };
    let interface = format!("eth{}", machine.interfaces.len());
    machine.interfaces.push(Interface {
        nom: interface.clone(),
        adresse,
        passerelle: None,
    });
    Ok(Bout {
        machine: machine.nom.clone(),
        interface,
    })
}

fn adresses_uniques(machines: &[Machine]) -> Result<(), String> {
    let mut vues: HashMap<Ipv4Addr, &str> = HashMap::new();
    for machine in machines {
        for interface in &machine.interfaces {
            if let Some(adresse) = interface.adresse.as_deref().and_then(Adresse::lire)
                && let Some(autre) = vues.insert(adresse.ip, &machine.nom)
            {
                return Err(format!(
                    "{} est donnée deux fois, à {autre} et à {}.",
                    adresse.ip, machine.nom
                ));
            }
        }
    }
    Ok(())
}

type Prise = (String, String);

/// Le réseau de chaque prise (machine, interface). Un câble relie ses deux
/// bouts dans un même réseau, et un switch y met tous ses ports. Toutes les
/// adresses d'un réseau doivent y être, et deux réseaux séparés ne
/// partagent pas la même plage.
fn reseaux(
    machines: &[Machine],
    cables: &[Cable],
) -> Result<HashMap<Prise, Option<Adresse>>, String> {
    let prises: Vec<Prise> = machines
        .iter()
        .flat_map(|m| m.interfaces.iter().map(|i| (m.nom.clone(), i.nom.clone())))
        .collect();
    let rang: HashMap<&Prise, usize> = prises.iter().enumerate().map(|(r, p)| (p, r)).collect();
    let mut parent: Vec<usize> = (0..prises.len()).collect();
    fn chef(parent: &mut [usize], i: usize) -> usize {
        let mut i = i;
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let mut unir = |a: usize, b: usize| {
        let (a, b) = (chef(&mut parent, a), chef(&mut parent, b));
        parent[a] = b;
    };
    for cable in cables {
        let de = rang[&(cable.de.machine.clone(), cable.de.interface.clone())];
        let vers = rang[&(cable.vers.machine.clone(), cable.vers.interface.clone())];
        unir(de, vers);
    }
    for machine in machines.iter().filter(|m| m.sorte == Sorte::Switch) {
        let ports: Vec<usize> = machine
            .interfaces
            .iter()
            .map(|i| rang[&(machine.nom.clone(), i.nom.clone())])
            .collect();
        for paire in ports.windows(2) {
            unir(paire[0], paire[1]);
        }
    }

    // Pour chaque réseau : sa plage, et la première machine qui l'a donnée.
    let mut plages: HashMap<usize, (Adresse, &str)> = HashMap::new();
    for (r, (machine, interface)) in prises.iter().enumerate() {
        let adresse = machines
            .iter()
            .find(|m| &m.nom == machine)
            .and_then(|m| m.interfaces.iter().find(|i| &i.nom == interface))
            .and_then(|i| i.adresse.as_deref())
            .and_then(Adresse::lire);
        let Some(adresse) = adresse else { continue };
        let groupe = chef(&mut parent, r);
        match plages.get(&groupe) {
            Some((plage, autre)) if *plage != adresse.reseau() => {
                return Err(format!(
                    "{autre} et {machine} sont dans le même réseau, mais {} n'est pas dans {plage}.",
                    adresse
                ));
            }
            Some(_) => {}
            None => {
                plages.insert(groupe, (adresse.reseau(), machine));
            }
        }
    }
    let mut vues: Vec<Adresse> = Vec::new();
    for (plage, _) in plages.values() {
        if vues.contains(plage) {
            return Err(format!(
                "Deux réseaux séparés utilisent {plage} : donne une autre plage à l'un des deux."
            ));
        }
        vues.push(*plage);
    }
    Ok(prises
        .iter()
        .enumerate()
        .map(|(r, prise)| {
            let groupe = chef(&mut parent, r);
            (prise.clone(), plages.get(&groupe).map(|(plage, _)| *plage))
        })
        .collect())
}

/// Une passerelle doit être l'adresse d'un routeur branché dans un réseau
/// de la machine.
fn passerelle_valide(
    machines: &[Machine],
    reseaux: &HashMap<Prise, Option<Adresse>>,
    nom: &str,
    texte: &str,
    quoi: &str,
) -> Result<Ipv4Addr, String> {
    let ip: Ipv4Addr = texte
        .parse()
        .map_err(|_| format!("{quoi} de {nom}, {texte}, n'est pas une adresse IPv4."))?;
    let machine = machines
        .iter()
        .find(|m| m.nom == nom)
        .ok_or_else(|| format!("{nom} n'est pas dans le lab."))?;
    if machine.sorte == Sorte::Switch {
        return Err(format!(
            "Un switch n'a ni passerelle ni route : retire celles de {nom}."
        ));
    }
    let plage = machine
        .interfaces
        .iter()
        .filter_map(|i| {
            let a = Adresse::lire(i.adresse.as_deref()?)?;
            a.contient(ip).then_some((i, a))
        })
        .next();
    let Some((interface, adresse)) = plage else {
        return Err(format!(
            "{quoi} de {nom}, {ip}, n'est dans aucun de ses réseaux."
        ));
    };
    if adresse.ip == ip {
        return Err(format!("{nom} ne peut pas être sa propre passerelle."));
    }
    let reseau = reseaux
        .get(&(nom.to_string(), interface.nom.clone()))
        .copied()
        .flatten();
    let proprietaire = machines.iter().find_map(|m| {
        m.interfaces
            .iter()
            .find(|i| {
                i.adresse
                    .as_deref()
                    .and_then(Adresse::lire)
                    .is_some_and(|a| a.ip == ip)
            })
            .map(|i| (m, i))
    });
    let Some((routeur, sa_prise)) = proprietaire else {
        return Err(format!(
            "{quoi} de {nom}, {ip}, n'est l'adresse d'aucune machine du lab."
        ));
    };
    if routeur.sorte != Sorte::Routeur {
        return Err(format!(
            "{quoi} de {nom} est {}, qui n'est pas un routeur.",
            routeur.nom
        ));
    }
    let son_reseau = reseaux
        .get(&(routeur.nom.clone(), sa_prise.nom.clone()))
        .copied()
        .flatten();
    if son_reseau != reseau {
        return Err(format!(
            "{quoi} de {nom} est {}, qui n'est pas branché dans son réseau.",
            routeur.nom
        ));
    }
    Ok(ip)
}

fn arrondi(valeur: f64) -> f64 {
    (valeur * 10.0).round() / 10.0
}

/// Les routeurs et les switchs sur une rangée, en suivant leurs câbles ; les
/// autres machines dessous, chacune sous celle du réseau où elle est
/// branchée. Les places sont en pour cent du schéma.
fn placer(machines: &mut [Machine], cables: &[Cable]) {
    let nombre = machines.len();
    let rang: HashMap<String, usize> = machines
        .iter()
        .enumerate()
        .map(|(r, m)| (m.nom.clone(), r))
        .collect();
    let mut voisins: Vec<Vec<usize>> = vec![Vec::new(); nombre];
    for cable in cables {
        let (Some(&a), Some(&b)) = (rang.get(&cable.de.machine), rang.get(&cable.vers.machine))
        else {
            continue;
        };
        if !voisins[a].contains(&b) {
            voisins[a].push(b);
        }
        if !voisins[b].contains(&a) {
            voisins[b].push(a);
        }
    }
    let du_reseau: Vec<bool> = machines
        .iter()
        .map(|m| matches!(m.sorte, Sorte::Routeur | Sorte::Switch))
        .collect();
    let liens_reseau = |r: usize| voisins[r].iter().filter(|&&v| du_reseau[v]).count();

    let mut rangee: Vec<usize> = Vec::new();
    let mut vue = vec![false; nombre];
    for depart in 0..nombre {
        if !du_reseau[depart] || vue[depart] {
            continue;
        }
        // Le groupe de machines du réseau reliées à `depart`, parcouru depuis
        // un bout pour qu'une chaîne reste droite.
        let mut groupe = vec![depart];
        let mut dans_groupe = vec![false; nombre];
        dans_groupe[depart] = true;
        let mut suivant = 0;
        while suivant < groupe.len() {
            for &v in &voisins[groupe[suivant]] {
                if du_reseau[v] && !dans_groupe[v] {
                    dans_groupe[v] = true;
                    groupe.push(v);
                }
            }
            suivant += 1;
        }
        let bout = *groupe
            .iter()
            .min_by_key(|&&r| (liens_reseau(r), r))
            .unwrap_or(&depart);
        let mut pile = vec![bout];
        while let Some(r) = pile.pop() {
            if vue[r] {
                continue;
            }
            vue[r] = true;
            rangee.push(r);
            for &v in voisins[r].iter().rev() {
                if du_reseau[v] && !vue[v] {
                    pile.push(v);
                }
            }
        }
    }

    let mut dessous: Vec<Vec<usize>> = vec![Vec::new(); nombre];
    let mut seules: Vec<usize> = Vec::new();
    for r in 0..nombre {
        if du_reseau[r] {
            continue;
        }
        match voisins[r].iter().find(|&&v| du_reseau[v]) {
            Some(&v) => dessous[v].push(r),
            None => seules.push(r),
        }
    }
    let largeurs: Vec<usize> = rangee.iter().map(|&r| dessous[r].len().max(1)).collect();
    let total = largeurs.iter().sum::<usize>() + seules.len();
    let deux_rangees = !rangee.is_empty() && rangee.len() < nombre;
    let (haut, bas) = if deux_rangees {
        (26.0, 76.0)
    } else {
        (46.0, 46.0)
    };
    let unite = 80.0 / total.max(1) as f64;
    let mut debut = 0.0;
    for (&r, &largeur) in rangee.iter().zip(&largeurs) {
        machines[r].x = arrondi(10.0 + (debut + largeur as f64 / 2.0) * unite);
        machines[r].y = haut;
        for (k, &h) in dessous[r].iter().enumerate() {
            machines[h].x = arrondi(10.0 + (debut + k as f64 + 0.5) * unite);
            machines[h].y = bas;
        }
        debut += largeur as f64;
    }
    for &h in &seules {
        machines[h].x = arrondi(10.0 + (debut + 0.5) * unite);
        machines[h].y = bas;
        debut += 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn machine(nom: &str, sorte: &str, passerelle: Option<&str>) -> MachineProposee {
        MachineProposee {
            nom: nom.into(),
            sorte: sorte.into(),
            passerelle: passerelle.map(str::to_string),
            routes: Vec::new(),
        }
    }

    fn cable(
        de: &str,
        adresse_de: Option<&str>,
        vers: &str,
        adresse_vers: Option<&str>,
    ) -> CablePropose {
        CablePropose {
            de: BoutPropose {
                machine: de.into(),
                adresse: adresse_de.map(str::to_string),
            },
            vers: BoutPropose {
                machine: vers.into(),
                adresse: adresse_vers.map(str::to_string),
            },
        }
    }

    /// Deux réseaux, chacun derrière son switch, et un routeur entre les deux.
    fn deux_reseaux() -> Proposition {
        Proposition {
            nom: "Deux réseaux".into(),
            machines: vec![
                machine("r1", "routeur", None),
                machine("sw1", "switch", None),
                machine("sw2", "switch", None),
                machine("pc1", "poste", Some("10.0.1.1")),
                machine("pc2", "poste", Some("10.0.1.1")),
                machine("srv1", "serveur", Some("10.0.2.1")),
            ],
            cables: vec![
                cable("r1", Some("10.0.1.1/24"), "sw1", None),
                cable("r1", Some("10.0.2.1/24"), "sw2", None),
                cable("pc1", Some("10.0.1.10/24"), "sw1", None),
                cable("pc2", Some("10.0.1.11/24"), "sw1", None),
                cable("srv1", Some("10.0.2.10/24"), "sw2", None),
            ],
        }
    }

    #[test]
    fn construit_un_lab_avec_ses_interfaces_et_ses_reseaux() {
        let lab = construire(&deux_reseaux()).unwrap();
        let r1 = &lab.machines[0];
        assert!(r1.transfere);
        assert_eq!(r1.interfaces.len(), 2);
        assert_eq!(r1.interfaces[1].nom, "eth1");
        assert_eq!(r1.interfaces[1].adresse.as_deref(), Some("10.0.2.1/24"));
        let sw1 = &lab.machines[1];
        assert_eq!(sw1.interfaces.len(), 3);
        assert!(sw1.interfaces.iter().all(|i| i.adresse.is_none()));
        let pc1 = lab.machines.iter().find(|m| m.nom == "pc1").unwrap();
        assert_eq!(pc1.interfaces[0].passerelle.as_deref(), Some("10.0.1.1"));
        assert!(!pc1.transfere);
        assert_eq!(lab.cables[2].de.interface, "eth0");
        assert_eq!(lab.cables[2].vers.interface, "eth1");
        assert_eq!(lab.cables[2].reseau.as_deref(), Some("10.0.1.0/24"));
        assert_eq!(lab.cables[1].reseau.as_deref(), Some("10.0.2.0/24"));
    }

    #[test]
    fn place_le_reseau_en_haut_et_les_postes_dessous() {
        let lab = construire(&deux_reseaux()).unwrap();
        let place = |nom: &str| {
            let m = lab.machines.iter().find(|m| m.nom == nom).unwrap();
            (m.x, m.y)
        };
        let (sw1, r1, sw2) = (place("sw1"), place("r1"), place("sw2"));
        assert!(sw1.0 < r1.0 && r1.0 < sw2.0, "{sw1:?} {r1:?} {sw2:?}");
        assert_eq!((sw1.1, r1.1, sw2.1), (26.0, 26.0, 26.0));
        let (pc1, pc2, srv1) = (place("pc1"), place("pc2"), place("srv1"));
        assert_eq!((pc1.1, pc2.1, srv1.1), (76.0, 76.0, 76.0));
        assert!(pc1.0 < sw1.0 && sw1.0 < pc2.0);
        assert_eq!(srv1.0, sw2.0);
        for (x, _) in [sw1, r1, sw2, pc1, pc2, srv1] {
            assert!((10.0..=90.0).contains(&x));
        }
    }

    #[test]
    fn un_routeur_et_deux_postes_ressemblent_au_premier_lab() {
        let proposition = Proposition {
            nom: "Deux postes".into(),
            machines: vec![
                machine("pc1", "poste", Some("10.0.1.1")),
                machine("routeur", "routeur", None),
                machine("pc2", "poste", Some("10.0.2.1")),
            ],
            cables: vec![
                cable("pc1", Some("10.0.1.10/24"), "routeur", Some("10.0.1.1/24")),
                cable("routeur", Some("10.0.2.1/24"), "pc2", Some("10.0.2.10/24")),
            ],
        };
        let lab = construire(&proposition).unwrap();
        let xy: Vec<(f64, f64)> = lab.machines.iter().map(|m| (m.x, m.y)).collect();
        assert_eq!(xy, vec![(30.0, 76.0), (50.0, 26.0), (70.0, 76.0)]);
    }

    #[test]
    fn des_postes_seuls_tiennent_sur_une_rangee() {
        let proposition = Proposition {
            nom: "Deux postes reliés".into(),
            machines: vec![machine("a", "poste", None), machine("b", "poste", None)],
            cables: vec![cable("a", Some("10.0.0.1/30"), "b", Some("10.0.0.2/30"))],
        };
        let lab = construire(&proposition).unwrap();
        assert_eq!(
            lab.machines.iter().map(|m| (m.x, m.y)).collect::<Vec<_>>(),
            vec![(30.0, 46.0), (70.0, 46.0)]
        );
    }

    #[test]
    fn une_chaine_de_routeurs_reste_dans_l_ordre() {
        let mut proposition = Proposition {
            nom: "Trois routeurs".into(),
            machines: vec![
                machine("r2", "routeur", None),
                machine("r1", "routeur", None),
                machine("r3", "routeur", None),
            ],
            cables: vec![
                cable("r1", Some("10.0.12.1/24"), "r2", Some("10.0.12.2/24")),
                cable("r2", Some("10.0.23.2/24"), "r3", Some("10.0.23.3/24")),
            ],
        };
        proposition.machines[1].routes = vec![Route {
            destination: "10.0.23.0/24".into(),
            passerelle: "10.0.12.2".into(),
        }];
        let lab = construire(&proposition).unwrap();
        let x = |nom: &str| lab.machines.iter().find(|m| m.nom == nom).unwrap().x;
        assert!(x("r1") < x("r2") && x("r2") < x("r3"));
        let r1 = lab.machines.iter().find(|m| m.nom == "r1").unwrap();
        assert_eq!(r1.routes[0].destination, "10.0.23.0/24");
    }

    fn refus(proposition: &Proposition) -> String {
        construire(proposition).unwrap_err()
    }

    #[test]
    fn refuse_ce_qui_ne_tournerait_pas() {
        let mut p = deux_reseaux();
        p.machines[4].nom = "pc1".into();
        assert!(refus(&p).contains("Deux machines s'appellent pc1"));

        let mut p = deux_reseaux();
        p.machines[3].nom = "PC_1".into();
        assert!(refus(&p).contains("ne peut pas nommer une machine"));

        let mut p = deux_reseaux();
        p.machines[1].sorte = "pare-feu".into();
        assert!(refus(&p).contains("les pare-feu n'existent pas encore"));

        let mut p = deux_reseaux();
        p.cables[0].vers.adresse = Some("10.0.1.2/24".into());
        assert!(refus(&p).contains("Un switch n'a pas d'adresse"));

        let mut p = deux_reseaux();
        p.cables[3].de.adresse = Some("10.0.9.11/24".into());
        assert!(refus(&p).contains("sont dans le même réseau"));

        let mut p = deux_reseaux();
        p.cables[3].de.adresse = Some("10.0.1.10/24".into());
        assert!(refus(&p).contains("est donnée deux fois"));

        let mut p = deux_reseaux();
        p.cables[3].de.adresse = Some("10.0.1.255/24".into());
        assert!(refus(&p).contains("diffusion"));

        let mut p = deux_reseaux();
        p.machines[3].passerelle = Some("10.0.1.11".into());
        assert!(refus(&p).contains("qui n'est pas un routeur"));

        let mut p = deux_reseaux();
        p.machines[3].passerelle = Some("10.0.2.1".into());
        assert!(refus(&p).contains("n'est dans aucun de ses réseaux"));

        let mut p = deux_reseaux();
        p.machines[3].passerelle = Some("10.0.1.254".into());
        assert!(refus(&p).contains("n'est l'adresse d'aucune machine"));

        let mut p = deux_reseaux();
        p.machines[3].routes = vec![Route {
            destination: "10.0.2.5/24".into(),
            passerelle: "10.0.1.1".into(),
        }];
        assert!(refus(&p).contains("doit être un réseau : 10.0.2.0/24"));

        let mut p = deux_reseaux();
        p.cables.push(cable("pc1", None, "pc1", None));
        assert!(refus(&p).contains("à elle-même"));

        let mut p = deux_reseaux();
        p.cables.push(cable("pc1", None, "pc9", None));
        assert!(refus(&p).contains("pc9, qui n'est pas dans le lab"));

        let mut p = deux_reseaux();
        p.machines = (0..13)
            .map(|n| machine(&format!("pc{n}"), "poste", None))
            .collect();
        p.cables.clear();
        assert!(refus(&p).contains("13 machines, c'est trop"));
    }

    #[test]
    fn deux_reseaux_separes_ne_partagent_pas_une_plage() {
        let proposition = Proposition {
            nom: "Doublon".into(),
            machines: vec![
                machine("a", "poste", None),
                machine("b", "poste", None),
                machine("c", "poste", None),
                machine("d", "poste", None),
            ],
            cables: vec![
                cable("a", Some("10.0.0.1/24"), "b", Some("10.0.0.2/24")),
                cable("c", Some("10.0.0.3/24"), "d", Some("10.0.0.4/24")),
            ],
        };
        assert!(refus(&proposition).contains("Deux réseaux séparés utilisent 10.0.0.0/24"));
    }

    #[test]
    fn retirer_une_machine_emporte_ses_cables_et_ses_passerelles() {
        let lab = construire(&deux_reseaux()).unwrap();
        let sans_pc2 = retirer(&lab, "pc2");
        assert_eq!(sans_pc2.machines.len(), 5);
        assert_eq!(sans_pc2.cables.len(), 4);
        let sw1 = sans_pc2.machines.iter().find(|m| m.nom == "sw1").unwrap();
        assert_eq!(sw1.interfaces.len(), 2);

        let sans_r1 = retirer(&lab, "r1");
        let pc1 = sans_r1.machines.iter().find(|m| m.nom == "pc1").unwrap();
        assert_eq!(pc1.interfaces[0].passerelle, None);
        assert!(
            sans_r1
                .cables
                .iter()
                .all(|c| c.de.machine != "r1" && c.vers.machine != "r1")
        );
    }

    #[test]
    fn se_lit_comme_l_ia_l_ecrit() {
        let texte = r#"{
            "nom": "Un switch et deux postes",
            "machines": [
                {"nom": "sw1", "sorte": "switch"},
                {"nom": "pc1", "sorte": "poste"},
                {"nom": "pc2", "sorte": "poste"}
            ],
            "cables": [
                {"de": {"machine": "pc1", "adresse": "192.168.1.10/24"}, "vers": {"machine": "sw1"}},
                {"de": {"machine": "pc2", "adresse": "192.168.1.11/24"}, "vers": {"machine": "sw1"}}
            ]
        }"#;
        let proposition: Proposition = serde_json::from_str(texte).unwrap();
        let lab = construire(&proposition).unwrap();
        assert_eq!(lab.cables[0].reseau.as_deref(), Some("192.168.1.0/24"));
        assert_eq!(lab.machines[0].x, 50.0);
    }
}
