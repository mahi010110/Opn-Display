import { useCallback, useEffect, useState, type MouseEvent } from 'react'

import { LabOuvert } from '../ecrans/LabOuvert'
import { MesLabs } from '../ecrans/MesLabs'
import { Reglages } from '../ecrans/Reglages'
import {
  arreterLab,
  arreterMachine,
  demarrerMachine,
  etatLab,
  lancerLab,
  remettreAZero,
  suivreLab,
  suivreMachines,
  veiller,
} from '../pont'
import type { EtatLab, Rapport } from '../types'

type Ecran = 'labs' | 'lab' | 'reglages'

/** Ce que les écrans peuvent demander au lab. */
export interface Gestes {
  lancer: () => Promise<void>
  arreter: () => Promise<void>
  demarrer: (machine: string) => Promise<void>
  eteindre: (machine: string) => Promise<void>
  remettreAZero: () => Promise<void>
}

/** L'app installée : le menu, et l'écran choisi dessous. */
export function Coque(props: { rapport: Rapport; onRapport: (rapport: Rapport) => void }) {
  const [ecran, setEcran] = useState<Ecran>('labs')
  const [labOuvert, setLabOuvert] = useState(false)
  const [etat, setEtat] = useState<EtatLab | null>(null)
  const [occupe, setOccupe] = useState<string | null>(null)
  const [erreur, setErreur] = useState<string | null>(null)

  const rafraichir = useCallback(() => {
    etatLab()
      .then(setEtat)
      .catch((e) => setErreur(String(e)))
  }, [])

  useEffect(() => {
    veiller().catch(() => undefined)
    rafraichir()
    const arrets: Array<() => void> = []
    suivreMachines(rafraichir).then((f) => arrets.push(f))
    suivreLab((texte) => setOccupe(texte)).then((f) => arrets.push(f))
    // La veille d'Incus dit ce qui change ; ce relevé rattrape ce qu'elle
    // aurait manqué.
    const minuterie = window.setInterval(rafraichir, 5000)
    return () => {
      arrets.forEach((f) => f())
      window.clearInterval(minuterie)
    }
  }, [rafraichir])

  const agir = useCallback(
    async (debut: string, action: () => Promise<EtatLab>) => {
      setErreur(null)
      setOccupe(debut)
      try {
        setEtat(await action())
      } catch (e) {
        setErreur(String(e))
        rafraichir()
      } finally {
        setOccupe(null)
      }
    },
    [rafraichir],
  )

  const gestes: Gestes = {
    lancer: () => agir('Démarrage', lancerLab),
    arreter: () => agir('Arrêt', arreterLab),
    demarrer: (machine) => agir(`Démarrage de ${machine}`, () => demarrerMachine(machine)),
    eteindre: (machine) => agir(`Arrêt de ${machine}`, () => arreterMachine(machine)),
    remettreAZero: () => agir('Remise à zéro', remettreAZero),
  }

  const aller = (vers: Ecran) => (evenement: MouseEvent) => {
    evenement.preventDefault()
    setEcran(vers)
  }

  const ouvrirLeLab = () => {
    setLabOuvert(true)
    setEcran('lab')
  }

  return (
    <div className="app">
      <header className="barre">
        <span className="marque">Opn-Display</span>
        <nav className="menu" aria-label="Menu">
          <a href="#" aria-current={ecran !== 'reglages' ? 'page' : undefined} onClick={aller('labs')}>
            <i className="ic i-labs" aria-hidden="true"></i>Labs
          </a>
        </nav>
        <div className="barre-fin">
          <a href="#" aria-current={ecran === 'reglages' ? 'page' : undefined} onClick={aller('reglages')}>
            <i className="ic i-reglages" aria-hidden="true"></i>Réglages
          </a>
        </div>
      </header>

      {ecran === 'labs' && (
        <MesLabs rapport={props.rapport} etat={etat} occupe={occupe} erreur={erreur} gestes={gestes} onOuvrir={ouvrirLeLab} />
      )}
      {/* Le lab reste monté quand on le quitte : ses terminaux continuent. */}
      {labOuvert && (
        <div className="ecran-garde" hidden={ecran !== 'lab'}>
          <LabOuvert
            rapport={props.rapport}
            etat={etat}
            occupe={occupe}
            erreur={erreur}
            gestes={gestes}
            visible={ecran === 'lab'}
            onRetour={() => setEcran('labs')}
          />
        </div>
      )}
      {ecran === 'reglages' && <Reglages rapport={props.rapport} onRapport={props.onRapport} />}
    </div>
  )
}
