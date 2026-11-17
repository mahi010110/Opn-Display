//! Le format de l'API d'OpenAI (`/chat/completions`), que parlent aussi
//! Ollama, Mistral, OpenRouter, Groq et bien d'autres.

use serde_json::{Value, json};

use super::outils::Outil;
use super::{Appel, Message, Reponse, http};

fn entetes(cle: Option<&str>) -> Vec<(&'static str, String)> {
    match cle {
        Some(cle) if !cle.is_empty() => vec![("Authorization", format!("Bearer {cle}"))],
        _ => Vec::new(),
    }
}

fn adresse_propre(adresse: &str) -> &str {
    adresse.trim().trim_end_matches('/')
}

pub(crate) fn corps(modele: &str, consigne: &str, messages: &[Message], outils: &[Outil]) -> Value {
    let mut liste = vec![json!({"role": "system", "content": consigne})];
    for message in messages {
        match message {
            Message::Utilisateur(texte) => liste.push(json!({"role": "user", "content": texte})),
            Message::Ia { texte, appels } => {
                let mut ia = json!({"role": "assistant", "content": texte});
                if !appels.is_empty() {
                    ia["tool_calls"] = appels
                        .iter()
                        .map(|a| {
                            json!({
                                "id": a.id,
                                "type": "function",
                                "function": {"name": a.nom, "arguments": a.arguments.to_string()},
                            })
                        })
                        .collect();
                }
                liste.push(ia);
            }
            Message::Resultats(resultats) => {
                for (id, contenu) in resultats {
                    liste.push(json!({"role": "tool", "tool_call_id": id, "content": contenu}));
                }
            }
        }
    }
    let mut corps = json!({"model": modele, "messages": liste});
    // Une liste d'outils vide est refusée par certains fournisseurs.
    if !outils.is_empty() {
        corps["tools"] = outils
            .iter()
            .map(|o| {
                json!({
                    "type": "function",
                    "function": {"name": o.nom, "description": o.description, "parameters": o.parametres},
                })
            })
            .collect();
    }
    corps
}

pub(crate) fn lire(reponse: &Value) -> Result<Reponse, String> {
    let message = &reponse["choices"][0]["message"];
    if message.is_null() {
        return Err("Le modèle n'a rien répondu.".into());
    }
    let appels = message["tool_calls"]
        .as_array()
        .map(|liste| {
            liste
                .iter()
                .enumerate()
                .map(|(rang, appel)| {
                    let brut = &appel["function"]["arguments"];
                    // Les arguments arrivent en texte JSON ; certains modèles
                    // locaux les donnent déjà en objet.
                    let arguments = match brut {
                        Value::String(texte) => {
                            serde_json::from_str(texte).unwrap_or(Value::Object(Default::default()))
                        }
                        Value::Object(_) => brut.clone(),
                        _ => Value::Object(Default::default()),
                    };
                    Appel {
                        id: appel["id"]
                            .as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("appel-{rang}")),
                        nom: appel["function"]["name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        arguments,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Reponse {
        texte: message["content"].as_str().unwrap_or_default().to_string(),
        appels,
    })
}

pub(crate) fn appeler(
    adresse: &str,
    cle: Option<&str>,
    modele: &str,
    consigne: &str,
    messages: &[Message],
    outils: &[Outil],
) -> Result<Reponse, String> {
    let entetes = entetes(cle);
    let entetes: Vec<(&str, &str)> = entetes.iter().map(|(n, v)| (*n, v.as_str())).collect();
    let reponse = http::poster(
        &format!("{}/chat/completions", adresse_propre(adresse)),
        &entetes,
        &corps(modele, consigne, messages, outils),
    )?;
    lire(&reponse)
}

pub(crate) fn modeles(adresse: &str, cle: Option<&str>) -> Result<Vec<String>, String> {
    let entetes = entetes(cle);
    let entetes: Vec<(&str, &str)> = entetes.iter().map(|(n, v)| (*n, v.as_str())).collect();
    let reponse = http::lire(&format!("{}/models", adresse_propre(adresse)), &entetes)?;
    let mut noms: Vec<String> = reponse["data"]
        .as_array()
        .map(|l| {
            l.iter()
                .filter_map(|m| m["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    noms.sort();
    Ok(noms)
}

/// Les modèles installés dans Ollama. Son API à lui est à côté de `/v1`.
pub(crate) fn modeles_ollama(adresse: &str) -> Result<Vec<String>, String> {
    let racine = adresse_propre(adresse).trim_end_matches("/v1");
    let reponse = http::lire(&format!("{racine}/api/tags"), &[])
        .map_err(|_| "Ollama ne répond pas sur ce PC.".to_string())?;
    Ok(reponse["models"]
        .as_array()
        .map(|l| {
            l.iter()
                .filter_map(|m| m["name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecrit_la_conversation_au_format_d_openai() {
        let messages = vec![
            Message::Utilisateur("Vérifie pc1".into()),
            Message::Ia {
                texte: String::new(),
                appels: vec![Appel {
                    id: "a1".into(),
                    nom: "lancer_commande".into(),
                    arguments: json!({"machine": "pc1", "commande": "ip -br a"}),
                }],
            },
            Message::Resultats(vec![("a1".into(), "code 0".into())]),
        ];
        let outils = vec![Outil {
            nom: "lancer_commande".into(),
            description: "d".into(),
            parametres: json!({"type": "object"}),
        }];
        let corps = corps("qwen3", "consigne", &messages, &outils);
        assert_eq!(corps["messages"][0]["role"], "system");
        assert_eq!(
            corps["messages"][2]["tool_calls"][0]["function"]["arguments"],
            "{\"commande\":\"ip -br a\",\"machine\":\"pc1\"}"
        );
        assert_eq!(corps["messages"][3]["role"], "tool");
        assert_eq!(corps["messages"][3]["tool_call_id"], "a1");
        assert_eq!(corps["tools"][0]["function"]["name"], "lancer_commande");
    }

    #[test]
    fn lit_une_reponse_d_ollama() {
        // Relevée le 17 novembre 2026, Ollama 0.40.16 et Qwen3 1.7B.
        let reponse = json!({"choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": "", "reasoning": "Okay...",
            "tool_calls": [{"id": "VPu3", "index": 0, "type": "function", "function": {
                "name": "lancer_commande", "arguments": "{\"machine\":\"pc1\",\"commande\":\"ping 10.0.2.10\"}"}}]}}]});
        let lue = lire(&reponse).unwrap();
        assert_eq!(lue.appels.len(), 1);
        assert_eq!(lue.appels[0].nom, "lancer_commande");
        assert_eq!(lue.appels[0].arguments["machine"], "pc1");
        assert!(lire(&json!({"choices": []})).is_err());
    }
}
