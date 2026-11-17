import { useCallback, useState } from 'react'

import type { EtatLab, Lab, Rapport } from '../types'
import { Dialogue } from '../ui/Dialogue'
import { Fenetre } from '../ui/Fenetre'
import { Jauge, texteMemoire } from '../ui/Jauge'
import { go } from '../ui/TonPc'

const PREMIER_LAB = 'premier-lab'

const enMarche = (etat: EtatLab) => etat.machines.filter((m) => m.marche === 'enMarche')

/** La liste des labs, et ce que le PC sait faire. */
export function MesLabs(props: {
  rapport: Rapport
  labs: EtatLab[]
  occupe: string | null
  erreur: string | null
  /** Le lab où une panne attend. */
  enPanne: string | null
  onOuvrir: (id: string) => void
  onLancer: (id: string) => void
  onArreter: (id: string) => void
  onSupprimer: (id: string) => void
  onNouveau: () => void
}) {
  const { rapport, occupe } = props
  const { totaleGo, libreGo } = rapport.memoire
  const [aSupprimer, setASupprimer] = useState<Lab | null>(null)
  const fermer = useCallback(() => setASupprimer(null), [])
  const tournent = props.labs.filter((l) => enMarche(l).length > 0)
  const arretes = props.labs.filter((l) => enMarche(l).length === 0)
  const badge = (id: string) =>
    props.enPanne === id && <span className="oc-badge oc-badge--danger">Panne</span>

  return (
    <>
      <div className="lab-barre">
        <h1 className="lab-nom">Mes labs</h1>
        <div className="lab-gestes">
          <button className="oc-btn" onClick={props.onNouveau}>
            <i className="ic i-plus" aria-hidden="true"></i>Nouveau lab
          </button>
        </div>
      </div>

      <main className="page page--deux">
        <div className="colonne">
          {tournent.length > 0 && (
            <Fenetre titre="En marche" active>
              {tournent.map((etat) => {
                const machines = enMarche(etat)
                const memoire = machines.reduce((total, m) => total + (m.memoire ?? 0), 0)
                return (
                  <div className="en-marche" key={etat.lab.id}>
                    <div className="en-marche-texte">
                      <p className="en-marche-nom">
                        {etat.lab.nom} {badge(etat.lab.id)}
                      </p>
                      <p className="mono">
                        {`${machines.length} ${machines.length > 1 ? 'machines' : 'machine'} en marche sur ${etat.lab.machines.length}, ${texteMemoire(memoire)} de RAM`}
                      </p>
                    </div>
                    <div className="lab-gestes">
                      <button
                        className="oc-btn"
                        disabled={occupe !== null}
                        onClick={() => props.onArreter(etat.lab.id)}
                      >
                        <i className="ic i-arreter" aria-hidden="true"></i>Arrêter
                      </button>
                      <button className="oc-btn oc-btn--primary" onClick={() => props.onOuvrir(etat.lab.id)}>
                        Ouvrir
                      </button>
                    </div>
                  </div>
                )
              })}
            </Fenetre>
          )}
          {arretes.length > 0 && (
            <Fenetre titre="Arrêtés" flush>
              <table className="table">
                <thead>
                  <tr>
                    <th>Lab</th>
                    <th>Machines</th>
                    <th>RAM</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {arretes.map((etat) => (
                    <tr key={etat.lab.id}>
                      <td className="nom">
                        {etat.lab.nom} {badge(etat.lab.id)}
                      </td>
                      <td className="chiffre">{etat.lab.machines.length}</td>
                      <td className="chiffre">
                        {texteMemoire(etat.lab.machines.reduce((total, m) => total + m.memoirePrevue, 0))}
                      </td>
                      <td className="fin">
                        {etat.lab.id !== PREMIER_LAB && (
                          <button
                            className="oc-btn oc-btn--ghost oc-btn--sm"
                            disabled={occupe !== null}
                            onClick={() => setASupprimer(etat.lab)}
                          >
                            Supprimer
                          </button>
                        )}
                        <button
                          className="oc-btn oc-btn--sm"
                          disabled={occupe !== null}
                          onClick={() => props.onLancer(etat.lab.id)}
                        >
                          Lancer
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </Fenetre>
          )}
          {props.erreur && <p className="note erreur">{props.erreur}</p>}
        </div>

        <Fenetre titre="Ton PC">
          <dl className="fiche">
            <dt>Mémoire</dt>
            <dd>
              {go(Math.round(totaleGo))} Go, {go(libreGo)} libres
            </dd>
            <dt>Conteneurs</dt>
            <dd>oui</dd>
            <dt>Machines virtuelles</dt>
            <dd className={rapport.kvm ? undefined : 'manque'}>{rapport.kvm ? 'oui' : 'non'}</dd>
            <dt>WiFi simulé</dt>
            <dd className={rapport.wifiSimule ? undefined : 'manque'}>{rapport.wifiSimule ? 'oui' : 'non'}</dd>
          </dl>
        </Fenetre>
      </main>

      <footer className="etat">
        <span>
          {occupe ?? (
            <>
              RAM du PC
              <Jauge part={(totaleGo - libreGo) / totaleGo} />
              {go(libreGo)} Go libres sur {go(Math.round(totaleGo))}
            </>
          )}
        </span>
      </footer>

      {aSupprimer && (
        <Dialogue
          titre="Supprimer le lab"
          onFermer={fermer}
          pied={
            <button
              className="oc-btn oc-btn--danger"
              onClick={() => {
                props.onSupprimer(aSupprimer.id)
                setASupprimer(null)
              }}
            >
              Supprimer
            </button>
          }
        >
          <p className="lu">
            « {aSupprimer.nom} » et ses machines disparaissent. Ce que tu y as changé est perdu.
          </p>
        </Dialogue>
      )}
    </>
  )
}
