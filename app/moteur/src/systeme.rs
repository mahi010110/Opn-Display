//! Les commandes qui tournent là où vivent les machines du lab : dans le
//! Debian de l'app sous Windows, sur le système lui-même sous Linux.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Le programme et ses arguments, tels qu'il faut les lancer d'ici.
pub fn ligne(programme: &str, args: &[&str]) -> Vec<String> {
    // Sans --exec, wsl.exe passe la ligne au shell de Debian, qui
    // interpréterait une seconde fois les $ et les guillemets.
    #[cfg(windows)]
    let debut = [
        "wsl.exe",
        "-d",
        crate::NOM_DISTRIBUTION,
        "-u",
        "root",
        "--exec",
    ]
    .as_slice();
    #[cfg(not(windows))]
    let debut: &[&str] = &[];
    debut
        .iter()
        .copied()
        .chain([programme])
        .chain(args.iter().copied())
        .map(str::to_string)
        .collect()
}

fn commande(programme: &str, args: &[&str]) -> Command {
    let ligne = ligne(programme, args);
    let mut commande = Command::new(&ligne[0]);
    commande.args(&ligne[1..]);
    #[cfg(windows)]
    crate::windows::sans_console(&mut commande);
    commande
}

/// Lance la commande et rend ce qu'elle a écrit. En cas d'échec, l'erreur est
/// ce qu'elle a dit.
pub(crate) fn lancer(programme: &str, args: &[&str]) -> Result<String, String> {
    lancer_avec(programme, args, None)
}

/// Pareil, en donnant `entree` à lire à la commande.
pub(crate) fn lancer_avec(
    programme: &str,
    args: &[&str],
    entree: Option<&[u8]>,
) -> Result<String, String> {
    let mut enfant = commande(programme, args)
        .stdin(if entree.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| pas_lance(programme, e))?;
    // L'écriture se fait à côté : une commande qui répond avant d'avoir tout
    // lu bloquerait sinon les deux côtés.
    let ecriture = entree.map(|entree| {
        let mut stdin = enfant.stdin.take();
        let entree = entree.to_vec();
        thread::spawn(move || {
            if let Some(stdin) = stdin.as_mut() {
                let _ = stdin.write_all(&entree);
            }
        })
    });
    let sortie = enfant
        .wait_with_output()
        .map_err(|e| pas_lance(programme, e))?;
    if let Some(ecriture) = ecriture {
        let _ = ecriture.join();
    }
    resultat(&sortie)
}

/// Lance la commande et passe chaque ligne qu'elle écrit, au fil de l'eau.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn suivre(
    programme: &str,
    args: &[&str],
    ligne_lue: &mut dyn FnMut(&str),
) -> Result<(), String> {
    let mut enfant = lancer_long(programme, args)?;
    let mut erreurs = enfant.stderr.take();
    let lecture_erreurs = thread::spawn(move || {
        let mut texte = Vec::new();
        if let Some(erreurs) = erreurs.as_mut() {
            let _ = erreurs.read_to_end(&mut texte);
        }
        texte
    });
    if let Some(sortie) = enfant.stdout.take() {
        for ligne in BufReader::new(sortie).lines().map_while(Result::ok) {
            ligne_lue(&ligne);
        }
    }
    let statut = enfant.wait().map_err(|e| pas_lance(programme, e))?;
    let erreurs = lecture_erreurs.join().unwrap_or_default();
    if statut.success() {
        Ok(())
    } else {
        Err(message(&erreurs, &[], statut.code()))
    }
}

/// Lance la commande et rend son code de sortie et ce qu'elle a écrit,
/// sortie puis erreurs. Passé `delai`, elle est arrêtée.
pub(crate) fn lancer_borne(
    programme: &str,
    args: &[&str],
    delai: Duration,
) -> Result<(Option<i32>, String), String> {
    let mut enfant = lancer_long(programme, args)?;
    let lire = |flux: Option<Box<dyn Read + Send>>| {
        thread::spawn(move || {
            let mut texte = Vec::new();
            if let Some(mut flux) = flux {
                let _ = flux.read_to_end(&mut texte);
            }
            texte
        })
    };
    let sortie = lire(
        enfant
            .stdout
            .take()
            .map(|f| Box::new(f) as Box<dyn Read + Send>),
    );
    let erreurs = lire(
        enfant
            .stderr
            .take()
            .map(|f| Box::new(f) as Box<dyn Read + Send>),
    );
    let debut = Instant::now();
    let statut = loop {
        if let Some(statut) = enfant.try_wait().map_err(|e| pas_lance(programme, e))? {
            break Some(statut);
        }
        if debut.elapsed() > delai {
            let _ = enfant.kill();
            let _ = enfant.wait();
            break None;
        }
        thread::sleep(Duration::from_millis(50));
    };
    let mut texte = crate::sortie::texte(&sortie.join().unwrap_or_default());
    texte.push_str(&crate::sortie::texte(&erreurs.join().unwrap_or_default()));
    Ok((statut.and_then(|s| s.code()), texte))
}

/// Lance une commande qui dure, sa sortie et ses erreurs à lire.
pub(crate) fn lancer_long(programme: &str, args: &[&str]) -> Result<Child, String> {
    commande(programme, args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| pas_lance(programme, e))
}

fn pas_lance(programme: &str, erreur: std::io::Error) -> String {
    format!("{programme} ne s'est pas lancé : {erreur}")
}

fn resultat(sortie: &Output) -> Result<String, String> {
    if sortie.status.success() {
        Ok(crate::sortie::texte(&sortie.stdout))
    } else {
        Err(message(
            &sortie.stderr,
            &sortie.stdout,
            sortie.status.code(),
        ))
    }
}

/// Ce que la commande a dit de son échec, sans le « Error: » d'Incus.
fn message(erreurs: &[u8], sortie: &[u8], code: Option<i32>) -> String {
    let erreurs = crate::sortie::texte(erreurs);
    let texte = if erreurs.trim().is_empty() {
        crate::sortie::texte(sortie)
    } else {
        erreurs
    };
    let texte = texte.trim();
    let texte = texte.strip_prefix("Error: ").unwrap_or(texte);
    if texte.is_empty() {
        match code {
            Some(code) => format!("la commande a échoué (code {code})"),
            None => "la commande a été arrêtée".into(),
        }
    } else {
        texte.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(windows))]
    #[test]
    fn lance_la_commande_et_lit_sa_sortie() {
        assert_eq!(
            lancer("echo", &["un $HOME", "deux"]).unwrap(),
            "un $HOME deux\n"
        );
        assert_eq!(
            lancer_avec("cat", &[], Some(b"lu sur l'entree")).unwrap(),
            "lu sur l'entree"
        );
        assert_eq!(
            lancer("sh", &["-c", "echo 'Error: rien ici' >&2; exit 3"]),
            Err("rien ici".into())
        );
        assert_eq!(
            lancer("sh", &["-c", "exit 4"]),
            Err("la commande a échoué (code 4)".into())
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn suit_la_sortie_ligne_par_ligne() {
        let mut lignes = Vec::new();
        suivre("printf", &["a\\nb\\n"], &mut |l| lignes.push(l.to_string())).unwrap();
        assert_eq!(lignes, ["a", "b"]);
    }

    #[cfg(not(windows))]
    #[test]
    fn dit_quand_le_programme_manque() {
        let erreur = lancer("opn-display-absent", &[]).unwrap_err();
        assert!(erreur.starts_with("opn-display-absent ne s'est pas lancé"));
    }

    #[test]
    fn la_ligne_garde_les_arguments_tels_quels() {
        let ligne = ligne("incus", &["exec", "pc1", "--", "sh", "-c", "echo \"$A\""]);
        assert_eq!(ligne.last().unwrap(), "echo \"$A\"");
        #[cfg(windows)]
        assert_eq!(
            &ligne[..6],
            ["wsl.exe", "-d", "Opn-Display", "-u", "root", "--exec"]
        );
        #[cfg(not(windows))]
        assert_eq!(ligne[0], "incus");
    }
}
