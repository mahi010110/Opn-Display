//! Le petit serveur MCP que Codex lance pour avoir les outils de l'app.
//!
//! Codex lance l'app elle-même comme serveur MCP, avec la variable
//! `OPN_OUTILS`. Ce programme-là ne fait que relayer : chaque appel d'outil
//! part à l'app ouverte par 127.0.0.1, avec un jeton, et c'est elle qui
//! demande l'accord et exécute.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;

use serde_json::{Value, json};

use super::Appel;
use super::outils::definitions;

/// La variable qui met l'app en mode relais.
pub const VARIABLE: &str = "OPN_OUTILS";

/// Côté app ouverte : le relais qui reçoit les appels de Codex.
pub struct Relais {
    /// La valeur à donner à `OPN_OUTILS` : le port et le jeton.
    pub valeur: String,
    port: u16,
    arret: Arc<AtomicBool>,
}

fn hasard() -> u64 {
    // Chaque RandomState est tiré au hasard par la bibliothèque standard.
    RandomState::new().build_hasher().finish()
}

/// Écoute sur 127.0.0.1 les appels relayés, et les exécute avec `executer`.
pub fn ecouter(
    executer: impl Fn(Appel) -> String + Send + Sync + 'static,
) -> Result<Relais, String> {
    let ecoute = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Le relais des outils ne s'ouvre pas : {e}"))?;
    let port = ecoute.local_addr().map_err(|e| e.to_string())?.port();
    let jeton = format!("{:016x}{:016x}", hasard(), hasard());
    // Une lettre devant : sans elle, Codex pourrait lire la valeur comme
    // un nombre ou une date TOML, et la changer.
    let valeur = format!("p{port}j{jeton}");
    let arret = Arc::new(AtomicBool::new(false));
    let (arret_ici, executer) = (arret.clone(), Arc::new(executer));
    let compteur = Arc::new(AtomicU64::new(0));
    thread::spawn(move || {
        for flux in ecoute.incoming() {
            if arret_ici.load(Ordering::Relaxed) {
                break;
            }
            let Ok(flux) = flux else { continue };
            let (executer, jeton, compteur) = (executer.clone(), jeton.clone(), compteur.clone());
            thread::spawn(move || {
                let _ = repondre(flux, &jeton, &compteur, executer.as_ref());
            });
        }
    });
    Ok(Relais {
        valeur,
        port,
        arret,
    })
}

fn repondre(
    mut flux: TcpStream,
    jeton: &str,
    compteur: &AtomicU64,
    executer: &(dyn Fn(Appel) -> String + Send + Sync),
) -> std::io::Result<()> {
    let mut ligne = String::new();
    BufReader::new(flux.try_clone()?).read_line(&mut ligne)?;
    let demande: Value = serde_json::from_str(&ligne).unwrap_or(Value::Null);
    let reponse = if demande["jeton"].as_str() != Some(jeton) {
        json!({"erreur": "jeton refusé"})
    } else {
        let numero = compteur.fetch_add(1, Ordering::Relaxed);
        let texte = executer(Appel {
            id: format!("codex-{numero}"),
            nom: demande["nom"].as_str().unwrap_or_default().to_string(),
            arguments: demande["arguments"].clone(),
        });
        json!({"texte": texte})
    };
    writeln!(flux, "{reponse}")?;
    flux.shutdown(Shutdown::Both)
}

impl Drop for Relais {
    fn drop(&mut self) {
        self.arret.store(true, Ordering::Relaxed);
        // Débloque l'attente d'une connexion, pour que le fil s'arrête.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

/// Côté programme lancé par Codex : le serveur MCP, sur l'entrée et la
/// sortie standard, qui relaie à l'app.
pub fn servir(valeur: &str) -> Result<(), String> {
    let (port, jeton) = lire_valeur(valeur).ok_or("OPN_OUTILS est illisible.")?;
    let entree = std::io::stdin();
    let mut sortie = std::io::stdout();
    for ligne in entree.lock().lines().map_while(Result::ok) {
        let Ok(message) = serde_json::from_str::<Value>(&ligne) else {
            continue;
        };
        // Une notification n'a pas d'identifiant, et pas de réponse.
        let Some(id) = message.get("id").cloned() else {
            continue;
        };
        let resultat = traiter(&message, |nom, arguments| {
            relayer(port, &jeton, nom, arguments)
        });
        let reponse = match resultat {
            Ok(resultat) => json!({"jsonrpc": "2.0", "id": id, "result": resultat}),
            Err((code, texte)) => {
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": texte}})
            }
        };
        writeln!(sortie, "{reponse}").map_err(|e| e.to_string())?;
        sortie.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn lire_valeur(valeur: &str) -> Option<(u16, String)> {
    let reste = valeur.strip_prefix('p')?;
    let (port, jeton) = reste.split_once('j')?;
    Some((port.parse().ok()?, jeton.to_string()))
}

/// Une requête MCP et sa réponse. `relayer` exécute un outil.
pub(crate) fn traiter(
    message: &Value,
    relayer: impl Fn(&str, &Value) -> Result<String, String>,
) -> Result<Value, (i64, String)> {
    match message["method"].as_str().unwrap_or_default() {
        "initialize" => Ok(json!({
            "protocolVersion": message["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
            "capabilities": {"tools": {"listChanged": false}},
            "serverInfo": {"name": "opn-display", "version": env!("CARGO_PKG_VERSION")},
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": definitions().iter().map(|o| json!({
                "name": o.nom, "description": o.description, "inputSchema": o.parametres,
            })).collect::<Vec<_>>(),
        })),
        "tools/call" => {
            let nom = message["params"]["name"].as_str().unwrap_or_default();
            let arguments = &message["params"]["arguments"];
            let (texte, erreur) = match relayer(nom, arguments) {
                Ok(texte) => (texte, false),
                Err(texte) => (texte, true),
            };
            Ok(json!({"content": [{"type": "text", "text": texte}], "isError": erreur}))
        }
        autre => Err((-32601, format!("Méthode inconnue : {autre}"))),
    }
}

fn relayer(port: u16, jeton: &str, nom: &str, arguments: &Value) -> Result<String, String> {
    let mut flux = TcpStream::connect(("127.0.0.1", port))
        .map_err(|e| format!("L'app ne répond plus : {e}"))?;
    let demande = json!({"jeton": jeton, "nom": nom, "arguments": arguments});
    writeln!(flux, "{demande}").map_err(|e| e.to_string())?;
    let mut ligne = String::new();
    BufReader::new(flux)
        .read_line(&mut ligne)
        .map_err(|e| e.to_string())?;
    let reponse: Value = serde_json::from_str(&ligne).map_err(|e| e.to_string())?;
    match reponse["texte"].as_str() {
        Some(texte) => Ok(texte.to_string()),
        None => Err(reponse["erreur"]
            .as_str()
            .unwrap_or("réponse illisible")
            .to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repond_comme_un_serveur_mcp() {
        let rien = |_: &str, _: &Value| Ok(String::new());
        let init = traiter(
            &json!({"method": "initialize", "params": {"protocolVersion": "2025-03-26"}}),
            rien,
        )
        .unwrap();
        assert_eq!(init["protocolVersion"], "2025-03-26");
        let liste = traiter(&json!({"method": "tools/list"}), rien).unwrap();
        assert!(
            liste["tools"]
                .as_array()
                .unwrap()
                .iter()
                .any(|o| o["name"] == "lancer_commande")
        );
        assert_eq!(
            traiter(&json!({"method": "resources/list"}), rien)
                .unwrap_err()
                .0,
            -32601
        );
    }

    #[test]
    fn le_relais_execute_avec_le_bon_jeton_seulement() {
        let relais =
            ecouter(|appel| format!("{} {}", appel.nom, appel.arguments["machine"])).unwrap();
        let (port, jeton) = lire_valeur(&relais.valeur).unwrap();
        assert_eq!(
            relayer(port, &jeton, "lancer_commande", &json!({"machine": "pc1"})),
            Ok("lancer_commande \"pc1\"".into())
        );
        assert_eq!(
            relayer(port, "faux", "lancer_commande", &json!({})),
            Err("jeton refusé".into())
        );
        let resultat = traiter(
            &json!({"method": "tools/call", "params": {"name": "etat_du_lab", "arguments": {}}}),
            |nom, arguments| relayer(port, &jeton, nom, arguments),
        )
        .unwrap();
        assert_eq!(resultat["content"][0]["text"], "etat_du_lab null");
        assert_eq!(resultat["isError"], false);
    }
}
