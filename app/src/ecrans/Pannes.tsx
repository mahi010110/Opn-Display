import { useEffect, useState } from 'react'

import { cataloguePannes } from '../pont'
import type { Difficulte, EtatIa, ResumePanne, VuePanne } from '../types'
import { Fenetre } from '../ui/Fenetre'

export const MOT_NIVEAU: Record<Difficulte, string> = { facile: 'facile', moyen: 'moyen', difficile: 'difficile' }

const NIVEAUX: { niveau: Difficulte; nom: string }[] = [
  { niveau: 'facile', nom: 'Facile' },
  { niveau: 'moyen', nom: 'Moyen' },
  { niveau: 'difficile', nom: 'Difficile' },
]

/** Pannes : le catalogue du lab, et une panne inventée par l'IA. */
export function Pannes(props: {
  etatIa: EtatIa | null
  panne: VuePanne | null
  nomDuLab: string
  occupe: string | null
  erreur: string | null
  invention: boolean
  onPoser: (id: string) => void
  onInventer: (niveau: Difficulte) => void
  onArreter: () => void
  onBrancher: () => void
  onOuvrirLab: () => void
}) {
  const [catalogue, setCatalogue] = useState<ResumePanne[]>([])
  const [niveau, setNiveau] = useState<Difficulte>('facile')
  const posee = props.panne !== null && !props.panne.verification?.reparee
  const bloque = posee || props.occupe !== null
  const branchee = props.etatIa?.reglages.branchement != null

  useEffect(() => {
    cataloguePannes()
      .then(setCatalogue)
      .catch(() => setCatalogue([]))
  }, [])

  return (
    <>
      <div className="lab-barre">
        <h1 className="lab-nom">Pannes</h1>
      </div>

      <main className="page page--deux">
        <div className="colonne">
          {posee && props.panne && (
            <div className="en-cours-panne">
              <span>
                Une panne est posée : <strong>{props.panne.titre}</strong>
              </span>
              <button className="oc-btn oc-btn--sm" onClick={props.onOuvrirLab}>
                Ouvrir le lab
              </button>
            </div>
          )}
          <Fenetre titre="Le catalogue" flush>
            <table className="table">
              <thead>
                <tr>
                  <th>Ce qui ne marche plus</th>
                  <th>Niveau</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {catalogue.map((p) => (
                  <tr key={p.id}>
                    <td className="nom">{p.titre}</td>
                    <td className="chiffre">{MOT_NIVEAU[p.niveau]}</td>
                    <td className="fin">
                      <button className="oc-btn oc-btn--sm" disabled={bloque} onClick={() => props.onPoser(p.id)}>
                        Poser
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </Fenetre>
          {props.erreur && <p className="note erreur">{props.erreur}</p>}
        </div>

        <Fenetre titre="Une panne inventée" active={branchee} bloc>
          {branchee ? (
            <>
              <p className="lu">L'IA casse quelque chose dans {props.nomDuLab}, sans te dire quoi.</p>
              <div className="segments" role="group" aria-label="Niveau de la panne">
                {NIVEAUX.map((n) => (
                  <button
                    key={n.niveau}
                    aria-pressed={niveau === n.niveau}
                    disabled={bloque}
                    onClick={() => setNiveau(n.niveau)}
                  >
                    {n.nom}
                  </button>
                ))}
              </div>
              <p className="note">L'app sauvegarde le lab avant. Tu peux toujours revenir en arrière.</p>
              {props.invention ? (
                <div className="ia-attente">
                  <span className="mono">L'IA travaille…</span>
                  <button className="oc-btn oc-btn--sm" onClick={props.onArreter}>
                    Arrêter
                  </button>
                </div>
              ) : (
                <div className="pied">
                  <button className="oc-btn oc-btn--primary" disabled={bloque} onClick={() => props.onInventer(niveau)}>
                    Poser une panne
                  </button>
                </div>
              )}
            </>
          ) : (
            <>
              <p className="lu">Branche l'IA pour qu'elle invente des pannes.</p>
              <div className="pied">
                <button className="oc-btn" onClick={props.onBrancher}>
                  Brancher l'IA
                </button>
              </div>
            </>
          )}
        </Fenetre>
      </main>

      <footer className="etat">
        <span>{props.occupe ?? `${catalogue.length} pannes dans le catalogue`}</span>
      </footer>
    </>
  )
}
