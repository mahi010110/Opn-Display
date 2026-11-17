//! La veille : Incus raconte ce qui arrive aux machines, par exemple une
//! machine qu'on éteint depuis son terminal. Sous Windows, elle garde aussi
//! Debian éveillé tant que l'app est ouverte : WSL arrête un Linux où plus
//! rien ne vient de Windows.

use std::io::{BufRead, BufReader};
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::systeme;

/// Ce qui vient d'arriver à une machine.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Changement {
    pub projet: String,
    pub machine: String,
    pub action: String,
}

pub struct Veille {
    arret: Arc<AtomicBool>,
    enfant: Arc<Mutex<Option<Child>>>,
}

/// Écoute Incus jusqu'à ce que la veille soit lâchée. Si Incus s'arrête,
/// elle recommence un peu plus tard.
pub fn veiller(mut changement: impl FnMut(Changement) + Send + 'static) -> Veille {
    let arret = Arc::new(AtomicBool::new(false));
    let enfant: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let (arret_ici, enfant_ici) = (arret.clone(), enfant.clone());
    thread::spawn(move || {
        while !arret_ici.load(Ordering::Relaxed) {
            let lance = systeme::lancer_long(
                "incus",
                &[
                    "monitor",
                    "--type=lifecycle",
                    "--format=json",
                    "--all-projects",
                ],
            );
            if let Ok(mut lance) = lance {
                let sortie = lance.stdout.take();
                if let Ok(mut place) = enfant_ici.lock() {
                    *place = Some(lance);
                }
                if let Some(sortie) = sortie {
                    for ligne in BufReader::new(sortie).lines().map_while(Result::ok) {
                        if let Some(c) = lire(&ligne) {
                            changement(c);
                        }
                    }
                }
                if let Ok(mut place) = enfant_ici.lock()
                    && let Some(mut fini) = place.take()
                {
                    let _ = fini.wait();
                }
            }
            thread::sleep(Duration::from_secs(3));
        }
    });
    Veille { arret, enfant }
}

impl Drop for Veille {
    fn drop(&mut self) {
        self.arret.store(true, Ordering::Relaxed);
        if let Ok(mut place) = self.enfant.lock()
            && let Some(enfant) = place.as_mut()
        {
            let _ = enfant.kill();
        }
    }
}

/// Un événement d'Incus, s'il dit qu'une machine a changé d'état.
fn lire(ligne: &str) -> Option<Changement> {
    let evenement: Value = serde_json::from_str(ligne.trim()).ok()?;
    let action = evenement["metadata"]["action"].as_str()?;
    let utile = matches!(
        action,
        "instance-created"
            | "instance-deleted"
            | "instance-started"
            | "instance-stopped"
            | "instance-shutdown"
            | "instance-restarted"
            | "instance-paused"
            | "instance-resumed"
    );
    if !utile {
        return None;
    }
    Some(Changement {
        projet: evenement["project"].as_str()?.to_string(),
        machine: evenement["metadata"]["name"].as_str()?.to_string(),
        action: action.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lit_les_evenements_d_incus() {
        // Deux lignes relevées le 17 novembre 2026 avec Incus 6.0.0.
        let arret = r#"{"location":"none","metadata":{"action":"instance-shutdown","name":"pc2","project":"opn-display","requestor":{"address":"@","protocol":"unix","username":"root"},"source":"/1.0/instances/pc2?project=opn-display"},"project":"opn-display","timestamp":"2026-11-17T08:57:35.090090068Z","type":"lifecycle"}"#;
        let commande = r#"{"location":"none","metadata":{"action":"instance-exec","context":{"command":["ip","-br","link","show","eth1"]},"name":"routeur","project":"opn-display","source":"/1.0/instances/routeur?project=opn-display"},"project":"opn-display","timestamp":"2026-11-17T08:57:35.13326343Z","type":"lifecycle"}"#;
        assert_eq!(
            lire(arret),
            Some(Changement {
                projet: "opn-display".into(),
                machine: "pc2".into(),
                action: "instance-shutdown".into()
            })
        );
        assert_eq!(lire(commande), None);
        assert_eq!(lire(""), None);
    }
}
