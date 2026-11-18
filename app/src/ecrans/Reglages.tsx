import { useCallback, useState, type MouseEvent } from 'react'

import { retirerLinux, verifierPc } from '../pont'
import type { EtatIa, Rapport } from '../types'
import { Dialogue } from '../ui/Dialogue'
import { Fenetre } from '../ui/Fenetre'
import { Verifications } from '../ui/TonPc'
import { BrancherIa } from './BrancherIa'
import { ReglerIa } from './ReglerIa'
import { SaMemoire } from './SaMemoire'
import { SonJournal } from './SonJournal'

type Retrait = 'ferme' | 'question' | 'enCours' | { erreur: string }

export type Section = 'brancher' | 'regler' | 'memoire' | 'journal' | 'pc'

const SECTIONS: { section: Section; nom: string; icone: string }[] = [
  { section: 'brancher', nom: "Brancher l'IA", icone: 'i-brancher' },
  { section: 'regler', nom: "Régler l'IA", icone: 'i-reglages' },
  { section: 'memoire', nom: 'Sa mémoire', icone: 'i-memoire' },
  { section: 'journal', nom: 'Son journal', icone: 'i-journal' },
  { section: 'pc', nom: 'Ton PC', icone: 'i-poste' },
]

/** Les réglages : la liste à gauche, la partie choisie à droite. */
export function Reglages(props: {
  rapport: Rapport
  onRapport: (rapport: Rapport) => void
  etatIa: EtatIa | null
  onEtatIa: (etat: EtatIa) => void
  section: Section
  onSection: (section: Section) => void
}) {
  return (
    <>
      <div className="lab-barre">
        <h1 className="lab-nom">Réglages</h1>
      </div>
      <main className="page reglages">
        <nav className="oc-window" aria-label="Réglages">
          <div className="oc-window__body oc-window__body--flush">
            <ul className="liste-nav">
              {SECTIONS.map(({ section, nom, icone }) => (
                <li key={section}>
                  <a
                    href="#"
                    aria-current={props.section === section ? 'page' : undefined}
                    onClick={(e: MouseEvent) => {
                      e.preventDefault()
                      props.onSection(section)
                    }}
                  >
                    <i className={'ic ' + icone} aria-hidden="true"></i>
                    {nom}
                  </a>
                </li>
              ))}
            </ul>
          </div>
        </nav>
        {props.section === 'brancher' && props.etatIa && (
          <BrancherIa
            etatIa={props.etatIa}
            onEtatIa={props.onEtatIa}
            sousWindows={props.rapport.plateforme === 'windows'}
          />
        )}
        {props.section === 'regler' && props.etatIa && <ReglerIa etatIa={props.etatIa} onEtatIa={props.onEtatIa} />}
        {props.section === 'memoire' && <SaMemoire />}
        {props.section === 'journal' && <SonJournal />}
        {props.section === 'pc' && <TonPc rapport={props.rapport} onRapport={props.onRapport} />}
      </main>
      <footer className="etat">
        <span>{props.rapport.systeme}</span>
      </footer>
    </>
  )
}

/** Ton PC : ce qu'il sait faire, et le Linux de l'app. */
function TonPc(props: { rapport: Rapport; onRapport: (rapport: Rapport) => void }) {
  const { rapport, onRapport } = props
  const [copie, setCopie] = useState(false)
  const [verification, setVerification] = useState(false)
  const [retrait, setRetrait] = useState<Retrait>('ferme')

  const verifier = () => {
    setVerification(true)
    verifierPc()
      .then(onRapport)
      .catch(() => undefined)
      .finally(() => setVerification(false))
  }

  const copier = () => {
    const lignes = rapport.verifications.map((v) => `${v.nom} : ${v.detail}`)
    navigator.clipboard
      .writeText([`Opn-Display, ${rapport.systeme}`, ...lignes].join('\n'))
      .then(() => setCopie(true))
      .catch(() => setCopie(false))
  }

  const fermer = useCallback(() => setRetrait((r) => (r === 'enCours' ? r : 'ferme')), [])

  const retirer = () => {
    setRetrait('enCours')
    retirerLinux()
      .then(onRapport)
      .catch((e) => setRetrait({ erreur: String(e) }))
  }

  return (
    <>
      <Fenetre titre="Ton PC" active bloc>
        <Verifications verifications={rapport.verifications} />
        <div className="pied">
          <button className="oc-btn" disabled={verification} onClick={verifier}>
            {verification ? 'Vérification en cours' : 'Vérifier à nouveau'}
          </button>
          <button className="oc-btn" onClick={copier}>
            {copie ? 'Copié' : 'Copier le rapport'}
          </button>
        </div>
        {rapport.linuxApp === 'installe' && (
          <div className="a-part">
            <span className="mono">Debian, le Linux de l'app, dans WSL2</span>
            <button className="oc-btn oc-btn--danger" onClick={() => setRetrait('question')}>
              Retirer Debian
            </button>
          </div>
        )}
      </Fenetre>

      {retrait !== 'ferme' && (
        <Dialogue
          titre="Retirer Debian"
          onFermer={fermer}
          pied={
            <button className="oc-btn oc-btn--danger" disabled={retrait === 'enCours'} onClick={retirer}>
              {retrait === 'enCours' ? 'Retrait en cours' : 'Retirer'}
            </button>
          }
        >
          <p className="lu">
            Debian est effacé de WSL2, avec Incus et les machines de tes labs. Tes autres Linux ne changent pas.
          </p>
          {typeof retrait === 'object' && <p className="note erreur">{retrait.erreur}</p>}
        </Dialogue>
      )}
    </>
  )
}
