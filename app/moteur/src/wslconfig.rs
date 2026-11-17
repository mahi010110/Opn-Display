//! Les réglages de WSL. `.wslconfig` règle tout WSL2 sur le PC : l'app n'y
//! change que la ligne `memory`, et seulement avec l'accord de
//! l'utilisateur. `/etc/wsl.conf` règle le seul Linux de l'app : l'app y
//! allume systemd, qui fait tourner Incus.

/// La mémoire donnée à WSL2, en Go, si le fichier la fixe.
pub fn memoire(contenu: &str) -> Option<f64> {
    valeur(contenu, "wsl2", "memory").and_then(taille_en_go)
}

/// Le même contenu, avec `memory=<go>GB` dans la section `[wsl2]`. Le reste
/// du fichier ne bouge pas.
pub fn avec_memoire(contenu: &str, go: u32) -> String {
    avec_reglage(contenu, "wsl2", "memory", &format!("memory={go}GB"))
}

/// Vrai si `/etc/wsl.conf` démarre Debian avec systemd.
pub fn systemd_actif(contenu: &str) -> bool {
    valeur(contenu, "boot", "systemd").is_some_and(|v| v.eq_ignore_ascii_case("true"))
}

/// Le même `/etc/wsl.conf`, avec systemd au démarrage.
pub fn avec_systemd(contenu: &str) -> String {
    avec_reglage(contenu, "boot", "systemd", "systemd=true")
}

fn valeur<'a>(contenu: &'a str, section_voulue: &str, cle: &str) -> Option<&'a str> {
    let mut section = String::new();
    for ligne in contenu.lines() {
        let ligne = sans_commentaire(ligne).trim();
        if let Some(nom) = entete(ligne) {
            section = nom;
        } else if section == section_voulue
            && let Some(valeur) = valeur_de(ligne, cle)
        {
            return Some(valeur);
        }
    }
    None
}

/// Pose `nouvelle` à la place de la ligne `cle` de la section, ou juste sous
/// son en-tête, ou dans une section ajoutée à la fin.
fn avec_reglage(contenu: &str, section_voulue: &str, cle: &str, nouvelle: &str) -> String {
    let fin = if contenu.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut lignes: Vec<String> = contenu.lines().map(str::to_string).collect();

    let mut section = String::new();
    let mut entete_voulu = None;
    let mut remplacee = false;
    for (i, ligne) in lignes.iter_mut().enumerate() {
        let propre = sans_commentaire(ligne).trim().to_string();
        if let Some(nom) = entete(&propre) {
            if nom == section_voulue && entete_voulu.is_none() {
                entete_voulu = Some(i);
            }
            section = nom;
        } else if section == section_voulue && valeur_de(&propre, cle).is_some() {
            *ligne = nouvelle.to_string();
            remplacee = true;
            break;
        }
    }

    if !remplacee {
        match entete_voulu {
            Some(i) => lignes.insert(i + 1, nouvelle.to_string()),
            None => {
                if lignes.last().is_some_and(|l| !l.trim().is_empty()) {
                    lignes.push(String::new());
                }
                lignes.push(format!("[{section_voulue}]"));
                lignes.push(nouvelle.to_string());
            }
        }
    }
    let mut resultat = lignes.join(fin);
    resultat.push_str(fin);
    resultat
}

/// La limite que l'app propose pour WSL2 : la mémoire du PC moins 8 Go
/// laissés à Windows, à partir de 24 Go, et seulement si WSL2 a moins.
pub fn proposition(totale_go: f64, actuelle_go: f64) -> Option<u32> {
    let totale = totale_go.round() as u32;
    if totale < 24 {
        return None;
    }
    let proposee = totale - 8;
    (actuelle_go.round() < f64::from(proposee)).then_some(proposee)
}

fn sans_commentaire(ligne: &str) -> &str {
    match ligne.find(['#', ';']) {
        Some(i) => &ligne[..i],
        None => ligne,
    }
}

fn entete(ligne: &str) -> Option<String> {
    let nom = ligne.strip_prefix('[')?.strip_suffix(']')?;
    Some(nom.trim().to_lowercase())
}

fn valeur_de<'a>(ligne: &'a str, cle: &str) -> Option<&'a str> {
    let (nom, valeur) = ligne.split_once('=')?;
    nom.trim().eq_ignore_ascii_case(cle).then(|| valeur.trim())
}

fn taille_en_go(valeur: &str) -> Option<f64> {
    let valeur = valeur.trim().to_ascii_uppercase();
    let fin_nombre = valeur
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(valeur.len());
    let nombre: f64 = valeur[..fin_nombre].parse().ok()?;
    let unite = valeur[fin_nombre..].trim();
    let facteur = match unite {
        "TB" | "T" => 1024.0,
        "GB" | "G" => 1.0,
        "MB" | "M" => 1.0 / 1024.0,
        "KB" | "K" => 1.0 / 1_048_576.0,
        _ => return None,
    };
    Some(nombre * facteur)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lit_la_memoire_dans_la_bonne_section() {
        assert_eq!(memoire("[wsl2]\nmemory=24GB\n"), Some(24.0));
        assert_eq!(
            memoire("[wsl2]\nprocessors=4\nMemory = 4096MB # moins\n"),
            Some(4.0)
        );
        assert_eq!(memoire("[experimental]\nmemory=8GB\n"), None);
        assert_eq!(memoire(""), None);
    }

    #[test]
    fn propose_une_limite_quand_elle_vaut_le_coup() {
        assert_eq!(proposition(31.8, 15.9), Some(24));
        assert_eq!(proposition(64.0, 32.0), Some(56));
        assert_eq!(proposition(32.0, 24.0), None);
        assert_eq!(proposition(16.0, 8.0), None);
    }

    #[test]
    fn ajoute_la_section_quand_elle_manque() {
        assert_eq!(avec_memoire("", 24), "[wsl2]\nmemory=24GB\n");
        assert_eq!(
            avec_memoire("[experimental]\nsparseVhd=true\n", 24),
            "[experimental]\nsparseVhd=true\n\n[wsl2]\nmemory=24GB\n"
        );
    }

    #[test]
    fn ajoute_la_ligne_sous_wsl2() {
        assert_eq!(
            avec_memoire("[wsl2]\nprocessors=4\n", 24),
            "[wsl2]\nmemory=24GB\nprocessors=4\n"
        );
    }

    #[test]
    fn allume_systemd_dans_wsl_conf() {
        assert!(!systemd_actif(""));
        assert!(!systemd_actif("[boot]\nsystemd=false\n"));
        assert!(systemd_actif("[boot]\nsystemd = true\n"));
        assert!(!systemd_actif("[user]\nsystemd=true\n"));
        assert_eq!(avec_systemd(""), "[boot]\nsystemd=true\n");
        assert_eq!(
            avec_systemd("[network]\nhostname=lab\n[boot]\nsystemd=false\n"),
            "[network]\nhostname=lab\n[boot]\nsystemd=true\n"
        );
        assert!(systemd_actif(&avec_systemd("[boot]\ncommand=echo\n")));
    }

    #[test]
    fn remplace_la_ligne_et_garde_le_reste() {
        let avant = "# mon réglage\r\n[wsl2]\r\nmemory=8GB\r\nswap=0\r\n";
        let apres = "# mon réglage\r\n[wsl2]\r\nmemory=24GB\r\nswap=0\r\n";
        assert_eq!(avec_memoire(avant, 24), apres);
    }
}
