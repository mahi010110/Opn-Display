import { useState } from 'react'

import type { Rapport } from '../types'
import { Fenetre } from '../ui/Fenetre'
import { Page } from '../ui/Page'
import { Verifications } from '../ui/TonPc'

/** Le Linux de l'app est en place : ce que le PC sait faire. */
export function Pret(props: { rapport: Rapport; onVerifier: () => void }) {
  const { rapport } = props
  const [copie, setCopie] = useState(false)
  const etat =
    rapport.linuxApp === 'installe' ? 'Debian est installé dans WSL2.' : "Sous Linux, l'app utilise ton système."

  const copier = () => {
    const lignes = rapport.verifications.map((v) => `${v.nom} : ${v.detail}`)
    navigator.clipboard
      .writeText([`Opn-Display, ${rapport.systeme}`, ...lignes].join('\n'))
      .then(() => setCopie(true))
      .catch(() => setCopie(false))
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
      </Fenetre>
    </Page>
  )
}
