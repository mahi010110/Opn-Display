//! Les appels HTTP aux fournisseurs d'IA.

use std::time::Duration;

use serde_json::Value;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

fn agent() -> ureq::Agent {
    #[cfg(windows)]
    let fournisseur = TlsProvider::NativeTls;
    #[cfg(not(windows))]
    let fournisseur = TlsProvider::Rustls;
    ureq::Agent::config_builder()
        // Un modèle local, sur le processeur, met parfois des minutes.
        .timeout_global(Some(Duration::from_secs(300)))
        .http_status_as_error(false)
        // Les certificats du système, comme le navigateur : ils suivent
        // ceux qu'ajoute un réseau d'entreprise ou d'école.
        .tls_config(
            TlsConfig::builder()
                .provider(fournisseur)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .into()
}

pub(crate) fn poster(url: &str, entetes: &[(&str, &str)], corps: &Value) -> Result<Value, String> {
    let mut requete = agent().post(url);
    for (nom, valeur) in entetes {
        requete = requete.header(*nom, *valeur);
    }
    let reponse = requete
        .send_json(corps)
        .map_err(|e| format!("Le fournisseur ne répond pas : {e}"))?;
    lire_reponse(reponse)
}

pub(crate) fn lire(url: &str, entetes: &[(&str, &str)]) -> Result<Value, String> {
    let mut requete = agent().get(url);
    for (nom, valeur) in entetes {
        requete = requete.header(*nom, *valeur);
    }
    let reponse = requete
        .call()
        .map_err(|e| format!("Le fournisseur ne répond pas : {e}"))?;
    lire_reponse(reponse)
}

fn lire_reponse(reponse: ureq::http::Response<ureq::Body>) -> Result<Value, String> {
    let statut = reponse.status().as_u16();
    let texte = reponse
        .into_body()
        .read_to_string()
        .map_err(|e| format!("La réponse du fournisseur s'est coupée : {e}"))?;
    let valeur: Value = serde_json::from_str(&texte).unwrap_or(Value::Null);
    if (200..300).contains(&statut) {
        if valeur.is_null() {
            return Err("Le fournisseur a répondu autre chose que du JSON.".into());
        }
        Ok(valeur)
    } else {
        Err(erreur(statut, &valeur, &texte))
    }
}

/// Une erreur du fournisseur, dite en clair, avec son propre message.
pub(crate) fn erreur(statut: u16, valeur: &Value, texte: &str) -> String {
    let message = valeur["error"]["message"]
        .as_str()
        .or_else(|| valeur["error"].as_str())
        .or_else(|| valeur["message"].as_str())
        .unwrap_or(texte)
        .trim();
    let debut = match statut {
        401 | 403 => "La clé est refusée.".to_string(),
        404 => "Introuvable : vérifie l'adresse et le modèle.".to_string(),
        429 => "Trop de demandes, ou le crédit est épuisé.".to_string(),
        500..=599 => format!("Le fournisseur a une panne (erreur {statut})."),
        _ => format!("Le fournisseur refuse la demande (erreur {statut})."),
    };
    if message.is_empty() {
        debut
    } else {
        let message: String = message.chars().take(300).collect();
        format!("{debut} Il dit : {message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dit_l_erreur_en_clair() {
        let corps = json!({"error": {"message": "Incorrect API key provided"}});
        assert_eq!(
            erreur(401, &corps, ""),
            "La clé est refusée. Il dit : Incorrect API key provided"
        );
        assert_eq!(
            erreur(404, &json!({"error": "model 'x' not found"}), ""),
            "Introuvable : vérifie l'adresse et le modèle. Il dit : model 'x' not found"
        );
        assert_eq!(
            erreur(503, &Value::Null, ""),
            "Le fournisseur a une panne (erreur 503)."
        );
    }
}
