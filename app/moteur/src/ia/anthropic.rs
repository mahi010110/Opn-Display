//! Le format de l'API d'Anthropic (`/v1/messages`).

use serde_json::{Value, json};

use super::outils::Outil;
use super::{Appel, Message, Reponse, http};

const ADRESSE: &str = "https://api.anthropic.com/v1";
const VERSION: &str = "2023-06-01";
const LONGUEUR_MAX: u32 = 4096;

pub(crate) fn corps(modele: &str, consigne: &str, messages: &[Message], outils: &[Outil]) -> Value {
    let mut liste: Vec<Value> = Vec::new();
    for message in messages {
        match message {
            Message::Utilisateur(texte) => {
                liste.push(json!({"role": "user", "content": [{"type": "text", "text": texte}]}));
            }
            Message::Ia { texte, appels } => {
                let mut contenu = Vec::new();
                if !texte.is_empty() {
                    contenu.push(json!({"type": "text", "text": texte}));
                }
                for appel in appels {
                    contenu.push(json!({
                        "type": "tool_use", "id": appel.id, "name": appel.nom, "input": appel.arguments,
                    }));
                }
                liste.push(json!({"role": "assistant", "content": contenu}));
            }
            Message::Resultats(resultats) => {
                let contenu: Vec<Value> = resultats
                    .iter()
                    .map(|(id, texte)| json!({"type": "tool_result", "tool_use_id": id, "content": texte}))
                    .collect();
                liste.push(json!({"role": "user", "content": contenu}));
            }
        }
    }
    let mut corps = json!({
        "model": modele,
        "max_tokens": LONGUEUR_MAX,
        "system": consigne,
        "messages": liste,
    });
    if !outils.is_empty() {
        corps["tools"] = outils
            .iter()
            .map(|o| json!({"name": o.nom, "description": o.description, "input_schema": o.parametres}))
            .collect();
    }
    corps
}

pub(crate) fn lire(reponse: &Value) -> Result<Reponse, String> {
    let blocs = reponse["content"]
        .as_array()
        .ok_or("Le modèle n'a rien répondu.")?;
    let mut lue = Reponse::default();
    for bloc in blocs {
        match bloc["type"].as_str() {
            Some("text") => lue
                .texte
                .push_str(bloc["text"].as_str().unwrap_or_default()),
            Some("tool_use") => lue.appels.push(Appel {
                id: bloc["id"].as_str().unwrap_or_default().to_string(),
                nom: bloc["name"].as_str().unwrap_or_default().to_string(),
                arguments: bloc["input"].clone(),
            }),
            _ => {}
        }
    }
    Ok(lue)
}

fn entetes(cle: &str) -> [(&'static str, &str); 2] {
    [("x-api-key", cle), ("anthropic-version", VERSION)]
}

pub(crate) fn appeler(
    cle: &str,
    modele: &str,
    consigne: &str,
    messages: &[Message],
    outils: &[Outil],
) -> Result<Reponse, String> {
    let reponse = http::poster(
        &format!("{ADRESSE}/messages"),
        &entetes(cle),
        &corps(modele, consigne, messages, outils),
    )?;
    lire(&reponse)
}

pub(crate) fn modeles(cle: &str) -> Result<Vec<String>, String> {
    let reponse = http::lire(&format!("{ADRESSE}/models?limit=100"), &entetes(cle))?;
    Ok(reponse["data"]
        .as_array()
        .map(|l| {
            l.iter()
                .filter_map(|m| m["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecrit_et_lit_le_format_d_anthropic() {
        let messages = vec![
            Message::Utilisateur("Vérifie pc1".into()),
            Message::Ia {
                texte: "Je regarde.".into(),
                appels: vec![Appel {
                    id: "toolu_1".into(),
                    nom: "etat_du_lab".into(),
                    arguments: json!({}),
                }],
            },
            Message::Resultats(vec![("toolu_1".into(), "pc1 en marche".into())]),
        ];
        let corps = corps("claude", "consigne", &messages, &[]);
        assert_eq!(corps["system"], "consigne");
        assert_eq!(corps["messages"][1]["content"][1]["type"], "tool_use");
        assert_eq!(corps["messages"][2]["content"][0]["tool_use_id"], "toolu_1");

        let reponse = json!({"content": [
            {"type": "text", "text": "Je lance un ping."},
            {"type": "tool_use", "id": "toolu_2", "name": "lancer_commande",
             "input": {"machine": "pc1", "commande": "ping -c 3 10.0.2.10"}}
        ], "stop_reason": "tool_use"});
        let lue = lire(&reponse).unwrap();
        assert_eq!(lue.texte, "Je lance un ping.");
        assert_eq!(lue.appels[0].arguments["commande"], "ping -c 3 10.0.2.10");
    }
}
