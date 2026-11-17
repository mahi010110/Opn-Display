//! L'abonnement ChatGPT, par Codex, l'outil officiel installé sur le PC.
//!
//! Codex fait la boucle de l'agent lui-même. Ses outils à lui sont coupés
//! (le shell, les fichiers, le web), il travaille en lecture seule, et ses
//! seuls outils sont ceux de l'app, par le relais MCP (`mcp.rs`). Lu dans
//! son code le 17 novembre 2026 : `codex-rs/exec/src/cli.rs`,
//! `exec_events.rs` et le schéma de sa configuration.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use super::outils::Execution;
use super::{Evenement, mcp};

/// Un tour de Codex ne dure pas plus.
const PATIENCE: Duration = Duration::from_secs(600);

/// Les noms sous lesquels Codex s'installe : un programme, ou le lanceur
/// que pose npm sous Windows.
#[cfg(windows)]
const NOMS: [&str; 2] = ["codex.exe", "codex.cmd"];
#[cfg(not(windows))]
const NOMS: [&str; 1] = ["codex"];

fn chemin() -> Option<PathBuf> {
    let dossiers = std::env::var_os("PATH")?;
    std::env::split_paths(&dossiers).find_map(|dossier| {
        NOMS.iter()
            .map(|nom| dossier.join(nom))
            .find(|chemin| chemin.is_file())
    })
}

pub(crate) fn present() -> bool {
    chemin().is_some()
}

/// Les réglages de Codex pour un tour : ses outils à lui coupés, ceux de
/// l'app branchés. Une valeur qui n'est pas du TOML est lue telle quelle.
fn reglages(outils: Option<(&str, &str)>) -> Vec<String> {
    let mut liste: Vec<String> = [
        "sandbox_mode=read-only",
        "features.shell_tool=false",
        "features.unified_exec=false",
        "features.apply_patch_freeform=false",
        "features.view_image=false",
        "features.web_search=false",
        "web_search=disabled",
    ]
    .iter()
    .map(|r| r.to_string())
    .collect();
    if let Some((programme, relais)) = outils {
        liste.push(format!("mcp_servers.opn.command={programme}"));
        liste.push(format!("mcp_servers.opn.env.{}={relais}", mcp::VARIABLE));
    }
    liste
        .into_iter()
        .flat_map(|reglage| ["-c".to_string(), reglage])
        .collect()
}

/// Un essai : Codex répond à une phrase, sans outil.
pub(crate) fn essayer(modele: &str) -> Result<String, String> {
    let codex = chemin().ok_or("Codex n'est pas installé sur ce PC.")?;
    let mut commande = Command::new(&codex);
    commande
        .args(["exec", "--json", "--skip-git-repo-check", "--ephemeral"])
        .args(reglages(None));
    if !modele.trim().is_empty() {
        commande.args(["--model", modele.trim()]);
    }
    commande
        .arg("Réponds seulement : prête.")
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    crate::windows::sans_console(&mut commande);
    let mut enfant = commande
        .spawn()
        .map_err(|e| format!("Codex ne s'est pas lancé : {e}"))?;
    let (envoi, reception) = mpsc::channel();
    if let Some(sortie) = enfant.stdout.take() {
        thread::spawn(move || {
            for ligne in BufReader::new(sortie).lines().map_while(Result::ok) {
                let _ = envoi.send(ligne);
            }
        });
    }
    let debut = Instant::now();
    let mut incident = String::new();
    while debut.elapsed() < Duration::from_secs(120) {
        match reception.recv_timeout(Duration::from_millis(200)) {
            Ok(ligne) => match lire_ligne(&ligne) {
                Lu::Texte(texte) => {
                    let _ = enfant.kill();
                    return Ok(texte);
                }
                Lu::Echec(message) => {
                    let _ = enfant.kill();
                    return Err(message);
                }
                Lu::Incident(message) => incident = message,
                _ => {}
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let _ = enfant.kill();
    Err(format!(
        "Codex n'a pas répondu. Es-tu connecté à ChatGPT dans Codex ? {incident}"
    ))
}

enum Signal {
    Ia(Evenement),
    Ligne(String),
    Fin,
}

/// Ce qu'une ligne de `codex exec --json` change au tour.
#[derive(Debug, PartialEq)]
enum Lu {
    Session(String),
    Texte(String),
    /// Codex essaie d'agir hors des outils de l'app.
    HorsOutils(String),
    /// Un incident qu'il surmonte souvent seul, comme une reconnexion.
    Incident(String),
    Echec(String),
    Rien,
}

fn lire_ligne(ligne: &str) -> Lu {
    let Ok(evenement) = serde_json::from_str::<Value>(ligne) else {
        return Lu::Rien;
    };
    let item = &evenement["item"];
    match evenement["type"].as_str().unwrap_or_default() {
        "thread.started" => evenement["thread_id"]
            .as_str()
            .map(|id| Lu::Session(id.to_string()))
            .unwrap_or(Lu::Rien),
        "item.started" | "item.completed"
            if matches!(
                item["type"].as_str(),
                Some("command_execution" | "file_change" | "web_search")
            ) =>
        {
            Lu::HorsOutils(item["type"].as_str().unwrap_or_default().to_string())
        }
        "item.completed" if item["type"] == "agent_message" => {
            Lu::Texte(item["text"].as_str().unwrap_or_default().to_string())
        }
        "turn.failed" => Lu::Echec(
            evenement["error"]["message"]
                .as_str()
                .unwrap_or("Codex a échoué.")
                .to_string(),
        ),
        "error" => Lu::Incident(
            evenement["message"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        ),
        _ => Lu::Rien,
    }
}

pub(crate) fn parler(
    modele: &str,
    session: &mut Option<String>,
    consigne: &str,
    texte: &str,
    execution: &Execution,
    signaler: &mut dyn FnMut(Evenement),
    arret: &AtomicBool,
) -> Result<(), String> {
    let codex = chemin().ok_or("Codex n'est pas installé sur ce PC.")?;
    let programme = std::env::current_exe()
        .map_err(|e| format!("L'app ne trouve pas son propre programme : {e}"))?;

    let (envoi, reception) = mpsc::channel::<Signal>();
    let envoi_outils = envoi.clone();
    let execution = execution.clone();
    let relais = mcp::ecouter(move |appel| {
        let envoi = envoi_outils.clone();
        execution.executer(&appel, &mut |e| {
            let _ = envoi.send(Signal::Ia(e));
        })
    })?;

    let mut commande = Command::new(&codex);
    commande.arg("exec");
    if let Some(id) = session.as_deref() {
        commande.args(["resume", id]);
    }
    commande
        .args(["--json", "--skip-git-repo-check"])
        .args(reglages(Some((
            &programme.to_string_lossy(),
            &relais.valeur,
        ))));
    if !modele.trim().is_empty() {
        commande.args(["--model", modele.trim()]);
    }
    commande
        .arg("-")
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    crate::windows::sans_console(&mut commande);
    let mut enfant = commande
        .spawn()
        .map_err(|e| format!("Codex ne s'est pas lancé : {e}"))?;

    let demande = if session.is_some() {
        texte.to_string()
    } else {
        format!("{consigne}\n\n{texte}")
    };
    if let Some(mut entree) = enfant.stdin.take() {
        entree
            .write_all(demande.as_bytes())
            .map_err(|e| format!("Codex ne lit pas la demande : {e}"))?;
    }
    if let Some(sortie) = enfant.stdout.take() {
        let envoi = envoi.clone();
        thread::spawn(move || {
            for ligne in BufReader::new(sortie).lines().map_while(Result::ok) {
                let _ = envoi.send(Signal::Ligne(ligne));
            }
            let _ = envoi.send(Signal::Fin);
        });
    }
    let mut erreurs = enfant.stderr.take();
    let lecture_erreurs = thread::spawn(move || {
        let mut texte = String::new();
        if let Some(erreurs) = erreurs.as_mut() {
            let _ = erreurs.read_to_string(&mut texte);
        }
        texte
    });
    drop(envoi);

    let debut = Instant::now();
    let mut echec = None;
    let mut dernier_incident = String::new();
    loop {
        if arret.load(Ordering::Relaxed) || debut.elapsed() > PATIENCE {
            let _ = enfant.kill();
            if !arret.load(Ordering::Relaxed) {
                echec = Some("Codex n'a pas fini à temps.".to_string());
            }
            break;
        }
        match reception.recv_timeout(Duration::from_millis(200)) {
            Ok(Signal::Ia(evenement)) => signaler(evenement),
            Ok(Signal::Ligne(ligne)) => match lire_ligne(&ligne) {
                Lu::Session(id) => *session = Some(id),
                Lu::Texte(texte) if !texte.trim().is_empty() => signaler(Evenement::Texte {
                    texte: texte.trim().to_string(),
                }),
                Lu::HorsOutils(quoi) => {
                    let _ = enfant.kill();
                    echec = Some(format!(
                        "Codex a voulu agir hors des outils de l'app ({quoi}). L'app l'a arrêté."
                    ));
                    break;
                }
                Lu::Echec(message) => echec = Some(message),
                Lu::Incident(message) => dernier_incident = message,
                _ => {}
            },
            Ok(Signal::Fin) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let statut = enfant.wait().ok();
    // Les derniers évènements des outils arrivent parfois après la fin.
    while let Ok(Signal::Ia(evenement)) = reception.try_recv() {
        signaler(evenement);
    }
    drop(relais);
    let erreurs = lecture_erreurs.join().unwrap_or_default();
    if let Some(echec) = echec {
        return Err(echec);
    }
    if statut.is_some_and(|s| !s.success()) && !arret.load(Ordering::Relaxed) {
        let fin: Vec<&str> = erreurs.lines().rev().take(3).collect();
        let fin: Vec<&str> = fin.into_iter().rev().collect();
        let raison = if dernier_incident.is_empty() {
            fin.join(" ")
        } else {
            dernier_incident
        };
        return Err(format!(
            "Codex s'est arrêté. Es-tu connecté à ChatGPT dans Codex ? {raison}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lit_les_evenements_de_codex() {
        assert_eq!(
            lire_ligne(
                r#"{"type":"thread.started","thread_id":"0199a213-81c0-7800-8aa1-bbab2a035a53"}"#
            ),
            Lu::Session("0199a213-81c0-7800-8aa1-bbab2a035a53".into())
        );
        assert_eq!(
            lire_ligne(
                r#"{"type":"item.completed","item":{"id":"item_3","type":"agent_message","text":"Le ping passe."}}"#
            ),
            Lu::Texte("Le ping passe.".into())
        );
        assert_eq!(
            lire_ligne(
                r#"{"type":"item.started","item":{"id":"item_1","type":"command_execution","command":"ls","status":"in_progress"}}"#
            ),
            Lu::HorsOutils("command_execution".into())
        );
        assert_eq!(
            lire_ligne(r#"{"type":"turn.failed","error":{"message":"401 Unauthorized"}}"#),
            Lu::Echec("401 Unauthorized".into())
        );
        // Relevé le 17 novembre 2026 avec Codex 0.169.0, réseau bloqué.
        assert_eq!(
            lire_ligne(r#"{"type":"error","message":"Reconnecting... 2/5 (stream disconnected)"}"#),
            Lu::Incident("Reconnecting... 2/5 (stream disconnected)".into())
        );
        assert_eq!(lire_ligne("pas du json"), Lu::Rien);
    }

    #[test]
    fn les_outils_de_codex_sont_coupes() {
        let reglages = reglages(Some(("/app/opn-display", "p1234jabc")));
        assert!(reglages.contains(&"features.shell_tool=false".to_string()));
        assert!(reglages.contains(&"mcp_servers.opn.command=/app/opn-display".to_string()));
        assert!(reglages.contains(&"mcp_servers.opn.env.OPN_OUTILS=p1234jabc".to_string()));
        assert_eq!(reglages.iter().filter(|r| *r == "-c").count(), 9);
    }
}
