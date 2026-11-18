import { useEffect, useRef, useState, type FormEvent } from 'react'

import { aiderIa, arreterIa, oublierConversation, parlerIa, repondreAccord, suivreIa } from '../pont'
import type { Demande, EtatIa, Niveau, VuePanne } from '../types'
import { CartePanne } from './CartePanne'
import { NIVEAUX, faconIa } from './reglageIa'

type EtatOutil = 'enCours' | 'fait' | 'echec' | 'refuse'

type Entree =
  | { sorte: 'toi'; texte: string }
  | { sorte: 'ia'; texte: string }
  | { sorte: 'outil'; id: string; titre: string; detail: string; etat: EtatOutil; apercu: string }
  | { sorte: 'erreur'; texte: string }
  | { sorte: 'note'; texte: string }

/** Sur une panne, sans rien demander depuis ce temps, l'utilisateur bloque. */
const BLOCAGE = 10 * 60 * 1000

const MOT_ETAT: Record<EtatOutil, string> = {
  enCours: 'en cours',
  fait: 'fait',
  echec: 'échec',
  refuse: 'refusé',
}

/** Le panneau de droite du lab : la discussion avec l'IA et ses demandes d'accord. */
export function PanneauIa(props: {
  etatIa: EtatIa | null
  onNiveau: (niveau: Niveau) => void
  onBrancher: () => void
  panne: VuePanne | null
  onPanne: (panne: VuePanne | null) => void
  onAutrePanne: () => void
}) {
  // Les indices déjà donnés avant que l'app ne se ferme reviennent dans le fil.
  const [entrees, setEntrees] = useState<Entree[]>(() =>
    (props.panne?.indicesLus ?? []).map((texte) => ({ sorte: 'ia', texte })),
  )
  const [demandes, setDemandes] = useState<Demande[]>([])
  const [texte, setTexte] = useState('')
  const [occupee, setOccupee] = useState(false)
  const fil = useRef<HTMLDivElement>(null)
  const posee = props.panne?.posee
  const dernierePosee = useRef(posee)
  // Elle propose son aide une fois par blocage : ce que fait l'utilisateur
  // remet l'attente à zéro.
  const derniereActivite = useRef(Date.now())
  const aidee = useRef(false)
  const activite = () => {
    derniereActivite.current = Date.now()
    aidee.current = false
  }

  useEffect(() => {
    let arreter: (() => void) | undefined
    let fini = false
    suivreIa(
      (evenement) => {
        setEntrees((avant) => {
          switch (evenement.sorte) {
            case 'texte':
              return [...avant, { sorte: 'ia', texte: evenement.texte }]
            case 'outil':
              return [...avant, { ...evenement, sorte: 'outil', etat: 'enCours', apercu: '' }]
            case 'resultat':
              return avant.map((e) =>
                e.sorte === 'outil' && e.id === evenement.id
                  ? { ...e, etat: evenement.ok ? 'fait' : 'echec', apercu: evenement.apercu }
                  : e,
              )
            case 'refus':
              return avant.map((e) => (e.sorte === 'outil' && e.id === evenement.id ? { ...e, etat: 'refuse' } : e))
          }
        })
        if (evenement.sorte !== 'texte') {
          setDemandes((avant) => avant.filter((d) => d.id !== evenement.id))
        }
      },
      (demande) => setDemandes((avant) => [...avant, demande]),
    ).then((f) => {
      if (fini) f()
      else arreter = f
    })
    return () => {
      fini = true
      arreter?.()
    }
  }, [])

  useEffect(() => {
    fil.current?.scrollTo({ top: fil.current.scrollHeight })
  }, [entrees, demandes, occupee])

  // Une nouvelle panne repart d'une discussion vide : l'IA ne mélange pas
  // l'ancienne panne et la nouvelle.
  useEffect(() => {
    if (posee !== undefined && posee !== dernierePosee.current && !occupee) {
      void oublierConversation()
      setEntrees([])
    }
    dernierePosee.current = posee
  }, [posee, occupee])

  const branchee = props.etatIa?.reglages.branchement != null
  const peutAider = props.etatIa?.comportements.initiative === 'quandTuBloques'
  const enPanne = props.panne !== null && !props.panne.verification?.reparee

  useEffect(activite, [posee, props.panne?.indicesDonnes, props.panne?.verification])

  useEffect(() => {
    if (!branchee || !peutAider || !enPanne || occupee) return
    const minuterie = window.setInterval(() => {
      if (aidee.current || document.visibilityState !== 'visible') return
      if (Date.now() - derniereActivite.current < BLOCAGE) return
      aidee.current = true
      setOccupee(true)
      aiderIa()
        .catch((e) => setEntrees((avant) => [...avant, { sorte: 'erreur', texte: String(e) }]))
        .finally(() => setOccupee(false))
    }, 30_000)
    return () => window.clearInterval(minuterie)
  }, [branchee, peutAider, enPanne, occupee])

  const envoyer = (evenement: FormEvent) => {
    evenement.preventDefault()
    const message = texte.trim()
    if (message === '' || occupee) return
    activite()
    setTexte('')
    setEntrees((avant) => [...avant, { sorte: 'toi', texte: message }])
    setOccupee(true)
    parlerIa(message)
      .catch((e) => setEntrees((avant) => [...avant, { sorte: 'erreur', texte: String(e) }]))
      .finally(() => {
        setOccupee(false)
        setDemandes([])
      })
  }

  const repondre = (demande: Demande, oui: boolean) => {
    void repondreAccord(demande.id, oui)
    setDemandes((avant) => avant.filter((d) => d.id !== demande.id))
  }

  const dire = (texte: string, sorte: 'ia' | 'note') => {
    setEntrees((avant) => [...avant, { sorte, texte }])
  }

  const recommencer = () => {
    void oublierConversation()
    setEntrees([])
  }

  return (
    <section className="oc-window">
      <header className="oc-window__bar oc-window__bar--inactive">
        <h2 className="oc-window__title">IA</h2>
      </header>
      <div className="oc-window__body oc-window__body--flush">
        <div className="ia">
          {branchee && props.etatIa && (
            <div className="ia-niveau">
              {props.etatIa.reglages.facon === 'parNiveau' ? (
                <div className="segments" role="group" aria-label="Niveau de l'IA">
                  {NIVEAUX.map(({ niveau, nom }) => (
                    <button
                      key={niveau}
                      aria-pressed={props.etatIa?.reglages.niveau === niveau}
                      onClick={() => props.onNiveau(niveau)}
                    >
                      {nom}
                    </button>
                  ))}
                </div>
              ) : (
                <span className="mono">{faconIa(props.etatIa)}</span>
              )}
              {entrees.length > 0 && !occupee && (
                <button className="oc-btn oc-btn--ghost oc-btn--sm ia-recommencer" onClick={recommencer}>
                  Nouvelle discussion
                </button>
              )}
            </div>
          )}

          {props.panne && (
            <CartePanne
              panne={props.panne}
              indices={props.etatIa?.comportements.solution !== 'rien'}
              onPanne={props.onPanne}
              onDire={dire}
              onAutre={props.onAutrePanne}
            />
          )}

          <div className="ia-fil" ref={fil}>
            {!branchee && (
              <div className="ia-vide">
                <p>L'IA n'est pas branchée.</p>
                <button className="oc-btn oc-btn--primary" onClick={props.onBrancher}>
                  Brancher l'IA
                </button>
              </div>
            )}
            {entrees.map((entree, i) => {
              switch (entree.sorte) {
                case 'toi':
                  return (
                    <p key={i} className="mot mot--toi">
                      {entree.texte}
                    </p>
                  )
                case 'ia':
                  return (
                    <p key={i} className="mot mot--ia">
                      {entree.texte}
                    </p>
                  )
                case 'erreur':
                  return (
                    <p key={i} className="mot mot--erreur">
                      {entree.texte}
                    </p>
                  )
                case 'note':
                  return (
                    <p key={i} className="note">
                      {entree.texte}
                    </p>
                  )
                case 'outil':
                  return (
                    <details key={i} className={'trace trace--' + entree.etat}>
                      <summary>
                        <span className="trace-titre">{entree.titre}</span>
                        {entree.detail !== '' && <span className="mono trace-detail">{entree.detail}</span>}
                        <span className="trace-etat">{MOT_ETAT[entree.etat]}</span>
                      </summary>
                      {entree.apercu !== '' && <pre>{entree.apercu}</pre>}
                    </details>
                  )
              }
            })}
            {demandes.map((demande) => (
              <div key={demande.id} className="carte">
                <div className="carte-tete">
                  <span className="oc-badge oc-badge--warn">Accord demandé</span>
                  {demande.lieu}
                </div>
                <div className="carte-corps">
                  <p className="carte-titre">{demande.titre}</p>
                  {demande.detail !== '' && <pre className="carte-detail">{demande.detail}</pre>}
                </div>
                <div className="carte-gestes">
                  <button className="oc-btn" onClick={() => repondre(demande, false)}>
                    Refuser
                  </button>
                  <button className="oc-btn oc-btn--primary" onClick={() => repondre(demande, true)}>
                    Autoriser
                  </button>
                </div>
              </div>
            ))}
            {occupee && demandes.length === 0 && (
              <div className="ia-attente">
                <span className="mono">L'IA travaille…</span>
                <button className="oc-btn oc-btn--sm" onClick={() => void arreterIa()}>
                  Arrêter
                </button>
              </div>
            )}
          </div>

          {branchee && (
            <form className="ia-saisie" onSubmit={envoyer}>
              <input
                className="oc-input"
                type="text"
                placeholder="Écris à l'IA"
                aria-label="Écrire à l'IA"
                value={texte}
                onChange={(e) => setTexte(e.target.value)}
              />
              <button className="oc-iconbtn" aria-label="Envoyer" disabled={occupee || texte.trim() === ''}>
                <i className="ic i-envoyer" aria-hidden="true"></i>
              </button>
            </form>
          )}
        </div>
      </div>
    </section>
  )
}
