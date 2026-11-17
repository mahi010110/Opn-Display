//! Lire ce que répondent wsl.exe, reg.exe et le Linux de l'app.

/// Le texte d'une sortie de programme. wsl.exe écrit en UTF-16 quand il
/// ignore WSL_UTF8 (versions anciennes), le reste est en UTF-8.
pub fn texte(octets: &[u8]) -> String {
    let texte = if semble_utf16(octets) {
        let unites: Vec<u16> = octets
            .chunks_exact(2)
            .map(|paire| u16::from_le_bytes([paire[0], paire[1]]))
            .collect();
        char::decode_utf16(unites)
            .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect()
    } else {
        String::from_utf8_lossy(octets).into_owned()
    };
    texte.replace(['\u{feff}', '\0'], "")
}

fn semble_utf16(octets: &[u8]) -> bool {
    if octets.len() < 2 || !octets.len().is_multiple_of(2) {
        return false;
    }
    let zeros = octets
        .iter()
        .skip(1)
        .step_by(2)
        .filter(|&&o| o == 0)
        .count();
    zeros * 2 >= octets.len() / 2
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionWsl {
    pub wsl: String,
    pub noyau: Option<String>,
}

/// La réponse de `wsl.exe --version`. Les libellés changent avec la langue
/// de Windows, l'ordre des lignes non : WSL d'abord, le noyau ensuite.
pub fn version_wsl(sortie: &str) -> Option<VersionWsl> {
    let mut lignes = sortie.lines().map(str::trim).filter(|l| !l.is_empty());
    let wsl = numero(lignes.next()?)?;
    let noyau = lignes.next().and_then(numero);
    Some(VersionWsl { wsl, noyau })
}

/// Le premier nombre à points d'une ligne : « Version WSL : 2.6.1.0 » donne
/// « 2.6.1.0 ».
fn numero(ligne: &str) -> Option<String> {
    ligne
        .split(|c: char| c.is_whitespace() || c == ':')
        .find(|mot| mot.starts_with(|c: char| c.is_ascii_digit()) && mot.contains('.'))
        .map(str::to_string)
}

/// Vrai si le noyau est au moins `majeur.mineur`. « 6.6.87.2-1 » n'est pas
/// au moins 6.18.
pub fn noyau_au_moins(noyau: &str, majeur: u32, mineur: u32) -> bool {
    let mut parties = noyau
        .split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty())
        .map(|p| p.parse::<u32>().unwrap_or(0));
    let (Some(a), Some(b)) = (parties.next(), parties.next()) else {
        return false;
    };
    (a, b) >= (majeur, mineur)
}

/// « 6.6.87.2-microsoft-standard-WSL2 » donne « 6.6 ».
pub fn noyau_court(noyau: &str) -> String {
    noyau.split('.').take(2).collect::<Vec<_>>().join(".")
}

/// Les noms rendus par `wsl.exe --list --quiet`.
pub fn distributions(sortie: &str) -> Vec<String> {
    sortie
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

/// Une valeur lue par `reg.exe query … /v Nom` :
/// `    EditionID    REG_SZ    Core`.
pub fn valeur_reg(sortie: &str, nom: &str) -> Option<String> {
    sortie.lines().find_map(|ligne| {
        let mut mots = ligne.split_whitespace();
        if mots.next()? != nom {
            return None;
        }
        mots.next()?;
        let valeur = mots.collect::<Vec<_>>().join(" ");
        (!valeur.is_empty()).then_some(valeur)
    })
}

/// Le nom français de l'édition de Windows.
pub fn edition_windows(edition_id: &str) -> String {
    match edition_id {
        "Core" | "CoreSingleLanguage" | "CoreCountrySpecific" | "CoreN" => "Famille",
        "Professional" | "ProfessionalN" => "Pro",
        "ProfessionalWorkstation" => "Pro pour stations de travail",
        "Education" | "ProfessionalEducation" => "Éducation",
        "Enterprise" | "EnterpriseS" => "Entreprise",
        autre => autre,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn en_utf16(texte: &str) -> Vec<u8> {
        texte.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    #[test]
    fn lit_l_utf16_de_wsl() {
        assert_eq!(texte(&en_utf16("Opn-Display\r\n")), "Opn-Display\r\n");
        assert_eq!(
            texte("Version WSL : 2.6.1.0".as_bytes()),
            "Version WSL : 2.6.1.0"
        );
    }

    #[test]
    fn lit_la_version_de_wsl_en_anglais_et_en_francais() {
        let anglais = "WSL version: 2.6.1.0\nKernel version: 6.6.87.2-1\nWSLg version: 1.0.66\n";
        let francais = "Version WSL : 2.6.1.0\r\nVersion du noyau : 6.6.87.2-1\r\n";
        let attendu = VersionWsl {
            wsl: "2.6.1.0".into(),
            noyau: Some("6.6.87.2-1".into()),
        };
        assert_eq!(version_wsl(anglais), Some(attendu.clone()));
        assert_eq!(version_wsl(francais), Some(attendu));
        assert_eq!(version_wsl("Usage: wsl.exe [Argument]"), None);
    }

    #[test]
    fn compare_les_noyaux() {
        assert!(!noyau_au_moins("6.6.87.2-1", 6, 18));
        assert!(noyau_au_moins("6.18.4.1-microsoft-standard-WSL2", 6, 18));
        assert!(noyau_au_moins("7.0.1", 6, 18));
        assert!(!noyau_au_moins("inconnu", 6, 18));
        assert_eq!(noyau_court("6.6.87.2-microsoft-standard-WSL2"), "6.6");
    }

    #[test]
    fn lit_la_liste_des_distributions() {
        assert_eq!(
            distributions("Ubuntu\r\nOpn-Display\r\n\r\n"),
            vec!["Ubuntu", "Opn-Display"]
        );
    }

    #[test]
    fn lit_une_valeur_du_registre() {
        let sortie = "\r\nHKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\r\n    EditionID    REG_SZ    Core\r\n\r\n";
        assert_eq!(valeur_reg(sortie, "EditionID"), Some("Core".into()));
        assert_eq!(valeur_reg(sortie, "CurrentBuild"), None);
    }

    #[test]
    fn traduit_l_edition() {
        assert_eq!(edition_windows("Core"), "Famille");
        assert_eq!(edition_windows("Professional"), "Pro");
        assert_eq!(edition_windows("IoTEnterprise"), "IoTEnterprise");
    }
}
