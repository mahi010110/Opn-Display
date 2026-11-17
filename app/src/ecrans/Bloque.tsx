import { useState, type ReactNode } from 'react'

import { installerWsl } from '../pont'
import type { Fabricant, Rapport } from '../types'
import { Fenetre } from '../ui/Fenetre'
import { Page } from '../ui/Page'
import { TonPc } from '../ui/TonPc'

/** Ce qui empêche de lancer un lab, et les gestes pour le régler. */
export function Bloque(props: { rapport: Rapport; onVerifier: () => void }) {
  const { rapport } = props
  let contenu: ReactNode
  switch (rapport.blocage) {
    case 'virtualisationCoupee':
      contenu = <VirtualisationCoupee fabricant={rapport.processeur.fabricant} processeur={rapport.processeur.nom} />
      break
    case 'wslAbsent':
      contenu = <WslAbsent />
      break
    case 'sansVirtualisation':
      contenu = (
        <p className="lu">
          Ton processeur ne propose pas la virtualisation. Les machines des labs ne peuvent pas tourner sur ce PC.
        </p>
      )
      break
    default:
      contenu = <p className="lu">Opn-Display ne tourne pas encore sur ce système.</p>
  }

  return (
    <Page titre="Avant ton premier lab" etat="Rien n'est installé pour l'instant.">
      <TonPc verifications={rapport.verifications} />
      <Fenetre titre="À faire une fois" active bloc>
        {contenu}
        <div className="pied">
          <button className="oc-btn oc-btn--primary" onClick={props.onVerifier}>
            Vérifier à nouveau
          </button>
        </div>
      </Fenetre>
    </Page>
  )
}

const REGLAGE: Record<Fabricant, string> = {
  amd: '« SVM Mode » ou « AMD-V »',
  intel: '« Intel VT-x » ou « Virtualization Technology »',
  autre: '« Virtualization Technology », « VT-x » ou « SVM »',
}

function VirtualisationCoupee(props: { fabricant: Fabricant; processeur: string }) {
  return (
    <>
      <p className="lu">
        Les labs ont besoin de la virtualisation. Elle s'active dans le BIOS, l'écran qui s'ouvre à l'allumage du PC.
      </p>
      <ol className="numeros">
        <li>
          <span>Redémarre le PC et ouvre le BIOS. La touche est souvent F2, F10 ou Suppr, juste à l'allumage.</span>
        </li>
        <li>
          <span>
            Active {REGLAGE[props.fabricant]}.{props.processeur !== '' && ` Ton processeur : ${props.processeur}.`}
          </span>
        </li>
        <li>
          <span>Enregistre, redémarre, puis rouvre l'app.</span>
        </li>
      </ol>
    </>
  )
}

type EtatWsl = 'avant' | 'enCours' | 'fait' | { erreur: string }

function WslAbsent() {
  const [etat, setEtat] = useState<EtatWsl>('avant')
  const lancer = () => {
    setEtat('enCours')
    installerWsl()
      .then(() => setEtat('fait'))
      .catch((e) => setEtat({ erreur: String(e) }))
  }

  return (
    <>
      <p className="lu">Les labs tournent dans WSL2, le Linux de Windows. Il n'est pas encore prêt sur ton PC.</p>
      <ol className="numeros">
        <li>
          <span>Installe WSL. Windows te demande ton accord.</span>
          {etat === 'avant' && (
            <button className="oc-btn" onClick={lancer}>
              Installer WSL
            </button>
          )}
          {etat === 'enCours' && <span className="mono">installation en cours</span>}
          {etat === 'fait' && <span className="mono">fait</span>}
          {typeof etat === 'object' && <span className="note erreur">{etat.erreur}</span>}
        </li>
        <li>
          <span>Redémarre le PC.</span>
        </li>
        <li>
          <span>Rouvre l'app.</span>
        </li>
      </ol>
    </>
  )
}
