import { useCallback, useState } from 'react'

import type { EtatLab, Marche, Rapport } from '../types'
import { Console } from '../ui/Console'
import type { Gestes } from '../ui/Coque'
import { Dialogue } from '../ui/Dialogue'
import { Jauge, texteMemoire } from '../ui/Jauge'
import { Schema } from '../ui/Schema'
import { go } from '../ui/TonPc'

/** Le lab ouvert : le schéma en haut, les terminaux en bas. */
export function LabOuvert(props: {
  rapport: Rapport
  etat: EtatLab | null
  occupe: string | null
  erreur: string | null
  gestes: Gestes
  visible: boolean
  onRetour: () => void
}) {
  const [onglets, setOnglets] = useState<string[]>([])
  const [actif, setActif] = useState<string | null>(null)
  const [question, setQuestion] = useState(false)
  const fermerQuestion = useCallback(() => setQuestion(false), [])

  const machines = props.etat?.machines ?? []
  const lab = props.etat?.lab
  const marche = (nom: string): Marche => machines.find((m) => m.nom === nom)?.marche ?? 'absente'
  const enMarche = machines.filter((m) => m.marche === 'enMarche')
  const creees = machines.filter((m) => m.marche !== 'absente')
  const memoire = enMarche.reduce((total, m) => total + (m.memoire ?? 0), 0)
  const limiteGo = props.rapport.memoire.wslGo ?? props.rapport.memoire.totaleGo
  const occupe = props.occupe !== null

  const choisir = (nom: string) => {
    setOnglets((avant) => (avant.includes(nom) ? avant : [...avant, nom]))
    setActif(nom)
  }

  const fermerOnglet = (nom: string) => {
    const reste = onglets.filter((n) => n !== nom)
    setOnglets(reste)
    if (actif === nom) setActif(reste[reste.length - 1] ?? null)
  }

  const pastille = props.occupe
    ? props.occupe
    : enMarche.length > 0
      ? `${enMarche.length} ${enMarche.length > 1 ? 'machines' : 'machine'} en marche`
      : 'arrêté'

  return (
    <>
      <div className="lab-barre">
        <button className="oc-btn oc-btn--ghost oc-btn--sm" onClick={props.onRetour}>
          <i className="ic i-retour" aria-hidden="true"></i>Mes labs
        </button>
        <h1 className="lab-nom">{lab?.nom ?? 'Lab'}</h1>
        <span className={'oc-chip' + (enMarche.length > 0 && !occupe ? ' oc-chip--dot' : '')}>{pastille}</span>
        <div className="lab-gestes">
          <button className="oc-btn" disabled={occupe || creees.length === 0} onClick={() => setQuestion(true)}>
            <i className="ic i-zero" aria-hidden="true"></i>Remettre à zéro
          </button>
          {enMarche.length > 0 ? (
            <button className="oc-btn" disabled={occupe} onClick={() => void props.gestes.arreter()}>
              <i className="ic i-arreter" aria-hidden="true"></i>Arrêter le lab
            </button>
          ) : (
            <button className="oc-btn oc-btn--primary" disabled={occupe} onClick={() => void props.gestes.lancer()}>
              Lancer le lab
            </button>
          )}
        </div>
      </div>

      <main className="lab lab--seul">
        <div className="lab-gauche">
          <section className="oc-window">
            <header className="oc-window__bar oc-window__bar--inactive">
              <h2 className="oc-window__title">Schéma</h2>
            </header>
            <div className="oc-window__body oc-window__body--flush">
              {lab && <Schema lab={lab} machines={machines} choisie={actif} onChoisir={choisir} />}
            </div>
          </section>

          <section className="oc-window">
            <header className="oc-window__bar oc-window__bar--accent">
              <h2 className="oc-window__title">Terminaux</h2>
            </header>
            <div className="oc-window__body oc-window__body--flush">
              <div className="terminaux">
                <div className="terminal">
                  <div className="onglets" role="tablist" aria-label="Terminaux">
                    {onglets.map((nom) => (
                      <span key={nom} className={'onglet' + (nom === actif ? ' onglet--actif' : '')}>
                        <button role="tab" aria-selected={nom === actif} onClick={() => setActif(nom)}>
                          {nom}
                        </button>
                        <button
                          className="onglet-fermer"
                          aria-label={`Fermer le terminal de ${nom}`}
                          onClick={() => fermerOnglet(nom)}
                        >
                          <i className="ic i-fermer" aria-hidden="true"></i>
                        </button>
                      </span>
                    ))}
                  </div>
                  <div className="ecran ecran--terminaux">
                    {onglets.length === 0 && (
                      <p className="ecran-vide">
                        {enMarche.length > 0
                          ? 'Choisis une machine sur le schéma pour ouvrir son terminal.'
                          : 'Lance le lab, puis choisis une machine sur le schéma.'}
                      </p>
                    )}
                    {onglets.map((nom) => (
                      <Console
                        key={nom}
                        machine={nom}
                        marche={marche(nom)}
                        visible={props.visible && nom === actif}
                        occupe={occupe}
                        onDemarrer={() => void props.gestes.demarrer(nom)}
                      />
                    ))}
                  </div>
                </div>
              </div>
            </div>
          </section>
        </div>
      </main>

      <footer className="etat">
        <span>
          RAM du lab
          <Jauge part={memoire / (limiteGo * 1024 ** 3)} />
          {texteMemoire(memoire)} sur {go(Math.round(limiteGo))} Go
        </span>
        {props.erreur && <span className="etat-fin etat-erreur">{props.erreur}</span>}
      </footer>

      {question && (
        <Dialogue
          titre="Remettre à zéro"
          onFermer={fermerQuestion}
          pied={
            <button
              className="oc-btn oc-btn--danger"
              onClick={() => {
                setQuestion(false)
                void props.gestes.remettreAZero()
              }}
            >
              Remettre à zéro
            </button>
          }
        >
          <p className="lu">Les machines repartent comme au premier lancement. Ce que tu y as changé est perdu.</p>
        </Dialogue>
      )}
    </>
  )
}
