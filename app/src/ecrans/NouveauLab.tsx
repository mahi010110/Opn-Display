import { useEffect, useState, type FormEvent, type KeyboardEvent } from 'react'

import { arreterIa, oublierProposition, proposerLab, retirerDuLab } from '../pont'
import type { EtatIa, Lab, MachineLab, Rapport, Sorte } from '../types'
import { Fenetre } from '../ui/Fenetre'
import { Jauge, texteMemoire } from '../ui/Jauge'
import { Schema } from '../ui/Schema'
import { go } from '../ui/TonPc'

/** Ce que l'app sait faire tourner à cette étape : des labs Debian. */
const IDEES = ['Deux réseaux et un routeur', 'Un switch et trois postes', 'Trois routeurs en chaîne']

/** Comme MACHINES_MAX dans moteur/src/proposition.rs. */
const MACHINES_MAX = 12

const AJOUTS: { sorte: Sorte; nom: string; icone: string; demande: string }[] = [
  { sorte: 'poste', nom: 'Poste', icone: 'i-poste', demande: 'Ajoute un poste.' },
  { sorte: 'serveur', nom: 'Serveur', icone: 'i-serveur', demande: 'Ajoute un serveur.' },
  { sorte: 'routeur', nom: 'Routeur', icone: 'i-routeur', demande: 'Ajoute un routeur.' },
  { sorte: 'switch', nom: 'Switch', icone: 'i-switch', demande: 'Ajoute un switch.' },
]

function systeme(machine: MachineLab): string {
  return machine.sorte === 'switch' ? 'Switch' : 'Debian 13'
}

/** Nouveau lab : on le décrit, l'IA le propose, on le lance. */
export function NouveauLab(props: {
  rapport: Rapport
  etatIa: EtatIa | null
  erreur: string | null
  onRetour: () => void
  onBrancher: () => void
  onCreer: (lab: Lab) => void
}) {
  const [description, setDescription] = useState('')
  const [lab, setLab] = useState<Lab | null>(null)
  const [commentaire, setCommentaire] = useState('')
  const [occupee, setOccupee] = useState(false)
  const [erreur, setErreur] = useState<string | null>(null)
  const [lancement, setLancement] = useState(false)
  const [ajout, setAjout] = useState(false)
  const branchee = props.etatIa?.reglages.branchement != null

  // Un nouveau lab repart d'une conversation vide avec l'IA.
  useEffect(() => {
    void oublierProposition()
  }, [])

  const proposer = (texte: string) => {
    if (texte.trim() === '' || occupee) return
    setDescription(texte)
    setErreur(null)
    setOccupee(true)
    proposerLab(texte, lab)
      .then((proposition) => {
        if (proposition === null) return
        if (proposition.lab) setLab(proposition.lab)
        setCommentaire(proposition.commentaire)
      })
      .catch((e) => setErreur(String(e)))
      .finally(() => setOccupee(false))
  }

  const envoyer = (evenement: FormEvent) => {
    evenement.preventDefault()
    proposer(description)
  }

  // Entrée propose, Maj+Entrée va à la ligne.
  const touche = (evenement: KeyboardEvent<HTMLTextAreaElement>) => {
    if (evenement.key === 'Enter' && !evenement.shiftKey && !evenement.nativeEvent.isComposing) {
      evenement.preventDefault()
      proposer(description)
    }
  }

  const retirer = (machine: string) => {
    if (!lab) return
    retirerDuLab(lab, machine)
      .then(setLab)
      .catch((e) => setErreur(String(e)))
  }

  // L'IA place la machine et la câble : elle repart du lab à l'écran.
  const ajouter = (demande: string) => {
    setAjout(false)
    proposer(demande)
  }

  const lancer = () => {
    if (!lab) return
    setLancement(true)
    props.onCreer(lab)
  }

  const prevue = lab?.machines.reduce((total, m) => total + m.memoirePrevue, 0) ?? 0
  const { libreGo, totaleGo } = props.rapport.memoire
  const libre = libreGo * 1024 ** 3
  const trop = prevue > libre
  const plein = (lab?.machines.length ?? 0) >= MACHINES_MAX
  const probleme = erreur ?? (lancement ? props.erreur : null)

  return (
    <>
      <div className="lab-barre">
        <button className="oc-btn oc-btn--ghost oc-btn--sm" onClick={props.onRetour}>
          <i className="ic i-retour" aria-hidden="true"></i>Mes labs
        </button>
        <h1 className="lab-nom">Nouveau lab</h1>
      </div>

      <main className="page page--plein">
        <Fenetre titre="Décris ton lab" active>
          {branchee ? (
            <>
              <form className="decrire" onSubmit={envoyer}>
                <textarea
                  className="oc-input"
                  rows={2}
                  aria-label="Décris ton lab"
                  placeholder="Par exemple : deux réseaux, un routeur entre les deux, et deux postes dans chacun."
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  onKeyDown={touche}
                />
                {occupee ? (
                  <button type="button" className="oc-btn" onClick={() => void arreterIa()}>
                    Arrêter
                  </button>
                ) : (
                  <button className="oc-btn" disabled={description.trim() === ''}>
                    Proposer
                  </button>
                )}
              </form>
              {!lab && !occupee && (
                <div className="propositions">
                  {IDEES.map((idee) => (
                    <button key={idee} className="pointille" onClick={() => proposer(idee)}>
                      {idee}
                    </button>
                  ))}
                </div>
              )}
            </>
          ) : (
            <>
              <p className="lu">Branche l'IA : elle construit le lab que tu décris.</p>
              <div className="pied">
                <button className="oc-btn oc-btn--primary" onClick={props.onBrancher}>
                  Brancher l'IA
                </button>
              </div>
            </>
          )}
        </Fenetre>

        <Fenetre titre="Le lab proposé" flush>
          <div className="propose">
            {lab ? (
              <Schema lab={lab} machines={[]} projet />
            ) : (
              <div className="plan plan--projet">
                <p className="ecran-vide">{occupee ? "L'IA prépare le lab…" : "Le lab que propose l'IA apparaît ici."}</p>
              </div>
            )}
            <div className="propose-liste">
              {lab && <p className="carte-titre">{lab.nom}</p>}
              {occupee && lab && <p className="mono">L'IA reprend le lab…</p>}
              {commentaire !== '' && <p className="mot mot--ia">{commentaire}</p>}
              {lab && (
                <>
                  <div className="propose-defile">
                    <table className="table table--serree">
                      <thead>
                        <tr>
                          <th>Machine</th>
                          <th>Système</th>
                          <th>RAM</th>
                          <th></th>
                        </tr>
                      </thead>
                      <tbody>
                        {lab.machines.map((machine) => (
                          <tr key={machine.nom}>
                            <td className="nom">{machine.nom}</td>
                            <td>{systeme(machine)}</td>
                            <td className="chiffre">{texteMemoire(machine.memoirePrevue)}</td>
                            <td className="fin">
                              <button
                                className="oc-iconbtn oc-iconbtn--ghost oc-iconbtn--sm"
                                aria-label={`Retirer ${machine.nom}`}
                                title={`Retirer ${machine.nom}`}
                                disabled={occupee || lancement || lab.machines.length === 1}
                                onClick={() => retirer(machine.nom)}
                              >
                                <i className="ic i-fermer" aria-hidden="true"></i>
                              </button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                  <div className="ajout">
                    {ajout ? (
                      <>
                        {AJOUTS.map((a) => (
                          <button
                            key={a.sorte}
                            className="pointille"
                            disabled={occupee || lancement}
                            onClick={() => ajouter(a.demande)}
                          >
                            <i className={'ic ' + a.icone} aria-hidden="true"></i>
                            {a.nom}
                          </button>
                        ))}
                        <button
                          className="oc-iconbtn oc-iconbtn--ghost oc-iconbtn--sm"
                          aria-label="Ne rien ajouter"
                          title="Ne rien ajouter"
                          onClick={() => setAjout(false)}
                        >
                          <i className="ic i-fermer" aria-hidden="true"></i>
                        </button>
                      </>
                    ) : (
                      <button
                        className="pointille"
                        disabled={occupee || lancement || plein}
                        title={plein ? `${MACHINES_MAX} machines au plus` : undefined}
                        onClick={() => setAjout(true)}
                      >
                        <i className="ic i-plus" aria-hidden="true"></i>Ajouter une machine
                      </button>
                    )}
                  </div>
                </>
              )}
              {probleme && <p className="note erreur">{probleme}</p>}
              <div className="propose-pied">
                <span className={trop ? 'mono trop' : 'mono'}>
                  {lab && (
                    <>
                      {trop
                        ? `${texteMemoire(prevue)} pour ${go(libreGo)} Go libres`
                        : `${texteMemoire(prevue)} sur ${go(libreGo)} Go libres`}
                      <Jauge part={libre > 0 ? prevue / libre : 1} />
                    </>
                  )}
                </span>
                <button
                  className="oc-btn oc-btn--primary"
                  disabled={!lab || trop || occupee || lancement}
                  onClick={lancer}
                >
                  Lancer le lab
                </button>
              </div>
            </div>
          </div>
        </Fenetre>
      </main>

      <footer className="etat">
        <span>
          RAM du PC
          <Jauge part={(totaleGo - libreGo) / totaleGo} />
          {go(libreGo)} Go libres sur {go(Math.round(totaleGo))}
        </span>
      </footer>
    </>
  )
}
