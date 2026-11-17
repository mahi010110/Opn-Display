import { useCallback, useState } from 'react'

import { retirerLinux } from '../pont'
import type { Rapport } from '../types'
import { Dialogue } from '../ui/Dialogue'
import { Fenetre } from '../ui/Fenetre'
import { Page } from '../ui/Page'
import { Verifications } from '../ui/TonPc'

type Retrait = 'ferme' | 'question' | 'enCours' | { erreur: string }

/** Le Linux de l'app est en place : ce que le PC sait faire. */
export function Pret(props: { rapport: Rapport; onVerifier: () => void; onRetire: (rapport: Rapport) => void }) {
  const { rapport, onRetire } = props
  const [copie, setCopie] = useState(false)
  const [retrait, setRetrait] = useState<Retrait>('ferme')
  const etat =
    rapport.linuxApp === 'installe' ? 'Debian est installé dans WSL2.' : "Sous Linux, l'app utilise ton système."

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
      .then(onRetire)
      .catch((e) => setRetrait({ erreur: String(e) }))
  }

  return (
    <Page titre="Ton PC" etat={etat} seule>
      <Fenetre titre="Ce qu'il sait faire" active bloc>
        <Verifications verifications={rapport.verifications} />
        <div className="pied">
          <button className="oc-btn" onClick={props.onVerifier}>
            Vérifier à nouveau
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
          <p className="lu">Debian est effacé de WSL2, avec tout ce qu'il contient. Tes autres Linux ne changent pas.</p>
          {typeof retrait === 'object' && <p className="note erreur">{retrait.erreur}</p>}
        </Dialogue>
      )}
    </Page>
  )
}
