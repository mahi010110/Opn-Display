//! Comment l'utilisateur règle l'IA : par niveau, un comportement à la fois,
//! ou une IA dont l'app change le niveau d'après ses pannes. Les trois
//! façons portent sur le lab, la solution et l'initiative
//! (`docs/projet.md`, « Les réglages »).

use serde::{Deserialize, Serialize};

use super::Niveau;
use super::outils::Liberte;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Facon {
    #[default]
    ParNiveau,
    UnParUn,
    SAdapte,
}

/// Ce qu'elle fait quand on lui demande la solution d'une panne.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Solution {
    /// Elle la donne, et l'explique.
    Donnee,
    /// Des indices, jamais la cause.
    #[default]
    Indices,
    /// Rien sur la panne, comme à un examen.
    Rien,
}

/// Quand elle parle sans qu'on lui écrive.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Initiative {
    /// Quand l'utilisateur bloque sur une panne.
    #[default]
    QuandTuBloques,
    Jamais,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Comportements {
    pub dans_le_lab: Liberte,
    pub solution: Solution,
    pub initiative: Initiative,
}

impl Niveau {
    /// Dans le lab, chaque niveau demande l'accord : agir sans demander se
    /// choisit seulement « un par un ».
    pub fn comportements(self) -> Comportements {
        let (solution, initiative) = match self {
            Niveau::Prof => (Solution::Donnee, Initiative::QuandTuBloques),
            Niveau::Coach => (Solution::Indices, Initiative::QuandTuBloques),
            Niveau::Silencieuse => (Solution::Rien, Initiative::Jamais),
        };
        Comportements {
            dans_le_lab: Liberte::AvecAccord,
            solution,
            initiative,
        }
    }

    pub fn nom(self) -> &'static str {
        match self {
            Niveau::Prof => "Prof",
            Niveau::Coach => "Coach",
            Niveau::Silencieuse => "Silencieuse",
        }
    }

    fn moins_d_aide(self) -> Niveau {
        match self {
            Niveau::Prof => Niveau::Coach,
            Niveau::Coach | Niveau::Silencieuse => Niveau::Silencieuse,
        }
    }

    fn plus_d_aide(self) -> Niveau {
        match self {
            Niveau::Silencieuse => Niveau::Coach,
            Niveau::Coach | Niveau::Prof => Niveau::Prof,
        }
    }
}

/// Le niveau le plus proche de comportements choisis un par un.
fn proche(comportements: Comportements) -> Niveau {
    match comportements.solution {
        Solution::Donnee => Niveau::Prof,
        Solution::Indices => Niveau::Coach,
        Solution::Rien => Niveau::Silencieuse,
    }
}

/// Où en est l'IA qui s'adapte, et pourquoi, en une phrase.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Adaptation {
    pub niveau: Niveau,
    /// Les pannes réparées sans indice depuis son dernier changement.
    pub serie: u32,
    pub raison: String,
}

/// Comment une panne s'est finie.
pub enum Issue<'a> {
    Reparee { titre: &'a str, indices: usize },
    Abandonnee { titre: &'a str },
}

/// Deux pannes de suite réparées sans indice, et elle aide moins.
const SERIE: u32 = 2;

impl Adaptation {
    pub fn depart(niveau: Niveau) -> Adaptation {
        Adaptation {
            niveau,
            serie: 0,
            raison: format!("Elle part de {}, le niveau que tu avais.", niveau.nom()),
        }
    }

    pub fn apres(&self, issue: &Issue) -> Adaptation {
        let niveau = self.niveau;
        // Le titre d'une panne est une phrase : son point ne finit pas celle-ci.
        let issue = match *issue {
            Issue::Reparee { titre, indices } => Issue::Reparee {
                titre: titre.trim().trim_end_matches('.'),
                indices,
            },
            Issue::Abandonnee { titre } => Issue::Abandonnee {
                titre: titre.trim().trim_end_matches('.'),
            },
        };
        match issue {
            Issue::Reparee { titre, indices: 0 } => {
                if niveau == Niveau::Silencieuse {
                    return Adaptation {
                        niveau,
                        serie: 0,
                        raison: format!(
                            "« {titre} » réparée sans indice : elle reste en Silencieuse."
                        ),
                    };
                }
                let serie = self.serie + 1;
                let suivant = niveau.moins_d_aide();
                if serie >= SERIE {
                    Adaptation {
                        niveau: suivant,
                        serie: 0,
                        raison: format!(
                            "{SERIE} pannes réparées sans indice : elle passe en {}.",
                            suivant.nom()
                        ),
                    }
                } else {
                    Adaptation {
                        niveau,
                        serie,
                        raison: format!(
                            "« {titre} » réparée sans indice. Encore une, et elle passe en {}.",
                            suivant.nom()
                        ),
                    }
                }
            }
            Issue::Reparee { titre, indices: 1 } => Adaptation {
                niveau,
                serie: 0,
                raison: format!(
                    "« {titre} » réparée avec un indice : elle reste en {}.",
                    niveau.nom()
                ),
            },
            Issue::Reparee { titre, indices } => {
                let nouveau = niveau.plus_d_aide();
                Adaptation {
                    niveau: nouveau,
                    serie: 0,
                    raison: format!(
                        "« {titre} » réparée avec {indices} indices : elle {} en {}.",
                        if nouveau == niveau {
                            "reste"
                        } else {
                            "repasse"
                        },
                        nouveau.nom()
                    ),
                }
            }
            Issue::Abandonnee { titre } => {
                let nouveau = niveau.plus_d_aide();
                Adaptation {
                    niveau: nouveau,
                    serie: 0,
                    raison: format!(
                        "« {titre} » abandonnée : elle {} en {}.",
                        if nouveau == niveau {
                            "reste"
                        } else {
                            "repasse"
                        },
                        nouveau.nom()
                    ),
                }
            }
        }
    }
}

/// Le réglage de l'IA avec l'utilisateur, tel qu'il est gardé.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Reglage {
    #[serde(default)]
    pub facon: Facon,
    #[serde(default)]
    pub niveau: Niveau,
    #[serde(default)]
    pub un_par_un: Comportements,
    #[serde(default)]
    pub adaptation: Adaptation,
}

impl Reglage {
    /// Le niveau qui décide en ce moment, sauf réglé un par un.
    pub fn niveau_du_moment(&self) -> Option<Niveau> {
        match self.facon {
            Facon::ParNiveau => Some(self.niveau),
            Facon::SAdapte => Some(self.adaptation.niveau),
            Facon::UnParUn => None,
        }
    }

    pub fn comportements(&self) -> Comportements {
        self.niveau_du_moment()
            .map(Niveau::comportements)
            .unwrap_or(self.un_par_un)
    }

    /// Change le réglage. L'IA qui s'adapte part du niveau qu'avait
    /// l'utilisateur.
    pub fn changer(&self, facon: Facon, niveau: Niveau, un_par_un: Comportements) -> Reglage {
        let adaptation = if facon == Facon::SAdapte && self.facon != Facon::SAdapte {
            Adaptation::depart(
                self.niveau_du_moment()
                    .unwrap_or_else(|| proche(self.un_par_un)),
            )
        } else {
            self.adaptation.clone()
        };
        Reglage {
            facon,
            niveau,
            un_par_un,
            adaptation,
        }
    }

    /// Une panne finie : l'IA qui s'adapte change peut-être de niveau.
    pub fn apres(&self, issue: &Issue) -> Reglage {
        let mut reglage = self.clone();
        if self.facon == Facon::SAdapte {
            reglage.adaptation = self.adaptation.apres(issue);
        }
        reglage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'IA qui s'adapte, partie de `niveau`.
    fn adapte(niveau: Niveau) -> Reglage {
        Reglage::default()
            .changer(Facon::ParNiveau, niveau, Comportements::default())
            .changer(Facon::SAdapte, niveau, Comportements::default())
    }

    fn reparee(indices: usize) -> Issue<'static> {
        Issue::Reparee {
            titre: "pc1 ne joint plus pc2",
            indices,
        }
    }

    #[test]
    fn chaque_niveau_demande_l_accord_dans_le_lab() {
        for niveau in [Niveau::Prof, Niveau::Coach, Niveau::Silencieuse] {
            assert_eq!(niveau.comportements().dans_le_lab, Liberte::AvecAccord);
        }
        assert_eq!(Niveau::Prof.comportements().solution, Solution::Donnee);
        assert_eq!(Niveau::Silencieuse.comportements().solution, Solution::Rien);
        assert_eq!(
            Niveau::Silencieuse.comportements().initiative,
            Initiative::Jamais
        );
    }

    #[test]
    fn un_par_un_garde_ce_qui_est_choisi() {
        let libre = Comportements {
            dans_le_lab: Liberte::Libre,
            solution: Solution::Rien,
            initiative: Initiative::QuandTuBloques,
        };
        let reglage = Reglage::default().changer(Facon::UnParUn, Niveau::Coach, libre);
        assert_eq!(reglage.niveau_du_moment(), None);
        assert_eq!(reglage.comportements(), libre);
    }

    #[test]
    fn s_adapter_part_du_niveau_d_avant() {
        let prof =
            Reglage::default().changer(Facon::ParNiveau, Niveau::Prof, Comportements::default());
        let adapte = prof.changer(Facon::SAdapte, Niveau::Prof, Comportements::default());
        assert_eq!(adapte.niveau_du_moment(), Some(Niveau::Prof));
        assert_eq!(
            adapte.adaptation.raison,
            "Elle part de Prof, le niveau que tu avais."
        );
        // Rester dans « Elle s'adapte » ne la fait pas repartir.
        let encore = adapte.apres(&reparee(0)).changer(
            Facon::SAdapte,
            Niveau::Coach,
            Comportements::default(),
        );
        assert_eq!(encore.adaptation.serie, 1);
    }

    #[test]
    fn deux_pannes_reparees_sans_indice_et_elle_aide_moins() {
        let reglage = adapte(Niveau::Prof).apres(&reparee(0));
        assert_eq!(reglage.niveau_du_moment(), Some(Niveau::Prof));
        assert_eq!(
            reglage.adaptation.raison,
            "« pc1 ne joint plus pc2 » réparée sans indice. Encore une, et elle passe en Coach."
        );
        let reglage = reglage.apres(&reparee(0));
        assert_eq!(reglage.niveau_du_moment(), Some(Niveau::Coach));
        assert_eq!(reglage.adaptation.serie, 0);
        let reglage = reglage.apres(&reparee(0)).apres(&reparee(0));
        assert_eq!(reglage.niveau_du_moment(), Some(Niveau::Silencieuse));
        let reglage = reglage.apres(&reparee(0));
        assert_eq!(reglage.niveau_du_moment(), Some(Niveau::Silencieuse));
    }

    #[test]
    fn un_indice_arrete_la_serie_et_deux_font_aider_plus() {
        let reglage = adapte(Niveau::Coach).apres(&reparee(0)).apres(&reparee(1));
        assert_eq!(reglage.niveau_du_moment(), Some(Niveau::Coach));
        assert_eq!(reglage.adaptation.serie, 0);
        let reglage = reglage.apres(&reparee(2));
        assert_eq!(reglage.niveau_du_moment(), Some(Niveau::Prof));
        assert_eq!(
            reglage.adaptation.raison,
            "« pc1 ne joint plus pc2 » réparée avec 2 indices : elle repasse en Prof."
        );
        let reglage = reglage.apres(&reparee(3));
        assert!(reglage.adaptation.raison.ends_with("elle reste en Prof."));
    }

    #[test]
    fn le_point_du_titre_ne_coupe_pas_la_phrase() {
        let reglage = adapte(Niveau::Coach).apres(&Issue::Reparee {
            titre: "pc2 ne joint plus rien hors de son réseau.",
            indices: 1,
        });
        assert_eq!(
            reglage.adaptation.raison,
            "« pc2 ne joint plus rien hors de son réseau » réparée avec un indice : elle reste en Coach."
        );
    }

    #[test]
    fn un_abandon_la_fait_aider_plus() {
        let reglage = adapte(Niveau::Silencieuse).apres(&Issue::Abandonnee {
            titre: "srv1 ne répond plus",
        });
        assert_eq!(reglage.niveau_du_moment(), Some(Niveau::Coach));
        assert_eq!(
            reglage.adaptation.raison,
            "« srv1 ne répond plus » abandonnée : elle repasse en Coach."
        );
    }

    #[test]
    fn par_niveau_une_panne_ne_change_rien() {
        let reglage = Reglage::default();
        assert_eq!(reglage.apres(&Issue::Abandonnee { titre: "x" }), reglage);
    }
}
