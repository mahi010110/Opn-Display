import { useCallback, useState } from 'react'

import { retirerLinux, verifierPc } from '../pont'
import type { Rapport } from '../types'
import { Dialogue } from '../ui/Dialogue'
import { Fenetre } from '../ui/Fenetre'
import { Verifications } from '../ui/TonPc'

type Retrait = 'ferme' | 'question' | 'enCours' | { erreur: string }

/** Les réglages. Pour l'instant, Ton PC : ce qu'il sait faire. */
export function Reglages(props: { rapport: Rapport; onRapport: (rapport: Rapport) => void }) {
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
      <div className="lab-barre">
        <h1 className="lab-nom">Réglages</h1>
      </div>

      <main className="page page--seule">
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
      </main>

      <footer className="etat">
        <span>{rapport.systeme}</span>
      </footer>

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
