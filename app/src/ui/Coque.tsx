import { useCallback, useEffect, useState, type MouseEvent } from 'react'

import { LabOuvert } from '../ecrans/LabOuvert'
import { MesLabs } from '../ecrans/MesLabs'
import { NouveauLab } from '../ecrans/NouveauLab'
import { Pannes } from '../ecrans/Pannes'
import { Reglages, type Section } from '../ecrans/Reglages'
import {
  arreterIa,
  etatIa as lireEtatIa,
  inventerPanne,
  panneEnCours,
  poserPanne,
  reglerIa,
  arreterLab,
  arreterMachine,
  creerLab,
  demarrerMachine,
  etatLabs,
  lancerLab,
  ouvrirLab,
  remettreAZero,
  suivreLab,
  suivreMachines,
  supprimerLab,
  veiller,
} from '../pont'
import type { Difficulte, EtatIa, EtatLab, Lab, Niveau, Rapport, VuePanne } from '../types'

type Ecran = 'labs' | 'lab' | 'nouveau' | 'pannes' | 'reglages'

const PREMIER_LAB = 'premier-lab'

/** Ce que les écrans peuvent demander au lab ouvert. */
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
  const [labMonte, setLabMonte] = useState(false)
  const [ouvert, setOuvert] = useState(PREMIER_LAB)
  const [labs, setLabs] = useState<EtatLab[]>([])
  const [occupe, setOccupe] = useState<string | null>(null)
  const [erreur, setErreur] = useState<string | null>(null)
  const [etatIa, setEtatIa] = useState<EtatIa | null>(null)
  const [section, setSection] = useState<Section>('brancher')
  const [panne, setPanne] = useState<VuePanne | null>(null)
  const [invention, setInvention] = useState(false)

  const etat = labs.find((l) => l.lab.id === ouvert) ?? null

  const rafraichir = useCallback(() => {
    etatLabs()
      .then(setLabs)
      .catch((e) => setErreur(String(e)))
  }, [])

  const retenir = useCallback((nouvel: EtatLab) => {
    setLabs((avant) =>
      avant.some((l) => l.lab.id === nouvel.lab.id)
        ? avant.map((l) => (l.lab.id === nouvel.lab.id ? nouvel : l))
        : [...avant, nouvel],
    )
  }, [])

  useEffect(() => {
    veiller().catch(() => undefined)
    lireEtatIa()
      .then(setEtatIa)
      .catch(() => undefined)
    panneEnCours()
      .then(setPanne)
      .catch(() => undefined)
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
        retenir(await action())
      } catch (e) {
        setErreur(String(e))
        rafraichir()
      } finally {
        setOccupe(null)
      }
    },
    [rafraichir, retenir],
  )

  const gestes: Gestes = {
    lancer: () => agir('Démarrage', () => lancerLab(ouvert)),
    arreter: () => agir('Arrêt', () => arreterLab(ouvert)),
    demarrer: (machine) => agir(`Démarrage de ${machine}`, () => demarrerMachine(ouvert, machine)),
    eteindre: (machine) => agir(`Arrêt de ${machine}`, () => arreterMachine(ouvert, machine)),
    // La remise à zéro emporte aussi la panne posée.
    remettreAZero: () =>
      agir('Remise à zéro', () => remettreAZero(ouvert)).then(() =>
        panneEnCours()
          .then(setPanne)
          .catch(() => undefined),
      ),
  }

  const aller = (vers: Ecran) => (evenement: MouseEvent) => {
    evenement.preventDefault()
    setEcran(vers)
  }

  /** Ouvre un lab à l'écran. L'IA et les pannes travaillent dans celui-là. */
  const ouvrir = useCallback(
    async (id: string) => {
      setErreur(null)
      try {
        retenir(await ouvrirLab(id))
        setOuvert(id)
        setLabMonte(true)
        setEcran('lab')
        return true
      } catch (e) {
        setErreur(String(e))
        return false
      }
    },
    [retenir],
  )

  const lancerLe = async (id: string) => {
    if (await ouvrir(id)) await agir('Démarrage', () => lancerLab(id))
  }

  const arreterLe = (id: string) => agir('Arrêt', () => arreterLab(id))

  const supprimer = async (id: string) => {
    setErreur(null)
    setOccupe('Suppression du lab')
    try {
      await supprimerLab(id)
      if (ouvert === id) {
        setOuvert(PREMIER_LAB)
        setLabMonte(false)
      }
      setLabs((avant) => avant.filter((l) => l.lab.id !== id))
      setPanne(await panneEnCours())
    } catch (e) {
      setErreur(String(e))
    } finally {
      setOccupe(null)
      rafraichir()
    }
  }

  /** Range le lab proposé, l'ouvre et le lance. */
  const creer = async (lab: Lab) => {
    setErreur(null)
    try {
      const nouveau = await creerLab(lab)
      retenir(nouveau)
      await lancerLe(nouveau.lab.id)
    } catch (e) {
      setErreur(String(e))
    }
  }

  const changerNiveau = (niveau: Niveau) => {
    if (!etatIa) return
    reglerIa('parNiveau', niveau, etatIa.reglages.unParUn)
      .then(setEtatIa)
      .catch(() => undefined)
  }

  const changerPanne = (vue: VuePanne | null) => {
    setPanne(vue)
    // Une panne finie peut changer le niveau de l'IA qui s'adapte.
    if (vue === null || vue.verification?.reparee) {
      lireEtatIa()
        .then(setEtatIa)
        .catch(() => undefined)
    }
  }

  const brancherLIa = () => {
    setSection('brancher')
    setEcran('reglages')
  }

  const poser = (pose: () => Promise<VuePanne | null>, debut: string) => {
    setErreur(null)
    setOccupe(debut)
    pose()
      .then((vue) => {
        if (vue === null) return
        setPanne(vue)
        rafraichir()
        setLabMonte(true)
        setEcran('lab')
      })
      .catch((e) => setErreur(String(e)))
      .finally(() => {
        setOccupe(null)
        setInvention(false)
      })
  }

  const inventer = (niveau: Difficulte) => {
    setInvention(true)
    poser(() => inventerPanne(niveau), "L'IA invente une panne")
  }

  const enPanne = panne !== null && !panne.verification?.reparee ? panne.lab : null
  const panneIci = panne !== null && panne.lab === ouvert ? panne : null

  return (
    <div className="app">
      <header className="barre">
        <span className="marque">Opn-Display</span>
        <nav className="menu" aria-label="Menu">
          <a
            href="#"
            aria-current={ecran === 'labs' || ecran === 'lab' || ecran === 'nouveau' ? 'page' : undefined}
            onClick={aller('labs')}
          >
            <i className="ic i-labs" aria-hidden="true"></i>Labs
          </a>
          <a href="#" aria-current={ecran === 'pannes' ? 'page' : undefined} onClick={aller('pannes')}>
            <i className="ic i-pannes" aria-hidden="true"></i>Pannes
          </a>
        </nav>
        <div className="barre-fin">
          <a href="#" aria-current={ecran === 'reglages' ? 'page' : undefined} onClick={aller('reglages')}>
            <i className="ic i-reglages" aria-hidden="true"></i>Réglages
          </a>
        </div>
      </header>

      {ecran === 'labs' && (
        <MesLabs
          rapport={props.rapport}
          labs={labs}
          occupe={occupe}
          erreur={erreur}
          enPanne={enPanne}
          onOuvrir={(id) => void ouvrir(id)}
          onLancer={(id) => void lancerLe(id)}
          onArreter={(id) => void arreterLe(id)}
          onSupprimer={(id) => void supprimer(id)}
          onNouveau={() => setEcran('nouveau')}
        />
      )}
      {ecran === 'nouveau' && (
        <NouveauLab
          rapport={props.rapport}
          etatIa={etatIa}
          erreur={erreur}
          onRetour={() => setEcran('labs')}
          onBrancher={brancherLIa}
          onCreer={(lab) => void creer(lab)}
        />
      )}
      {/* Le lab reste monté quand on le quitte : ses terminaux continuent. */}
      {labMonte && (
        <div className="ecran-garde" hidden={ecran !== 'lab'}>
          <LabOuvert
            key={ouvert}
            rapport={props.rapport}
            etat={etat}
            occupe={occupe}
            erreur={erreur}
            gestes={gestes}
            visible={ecran === 'lab'}
            onRetour={() => setEcran('labs')}
            etatIa={etatIa}
            onNiveau={changerNiveau}
            onBrancher={brancherLIa}
            panne={panneIci}
            onPanne={changerPanne}
            onAutrePanne={() => setEcran('pannes')}
          />
        </div>
      )}
      {ecran === 'pannes' && (
        <Pannes
          key={ouvert}
          etatIa={etatIa}
          panne={panne}
          nomDuLab={etat?.lab.nom ?? ''}
          occupe={occupe}
          erreur={erreur}
          invention={invention}
          onPoser={(id) => poser(() => poserPanne(id), 'Pose de la panne')}
          onInventer={inventer}
          onArreter={() => void arreterIa()}
          onBrancher={brancherLIa}
          onOuvrirLab={() => void ouvrir(panne?.lab ?? ouvert)}
        />
      )}
      {ecran === 'reglages' && (
        <Reglages
          rapport={props.rapport}
          onRapport={props.onRapport}
          etatIa={etatIa}
          onEtatIa={setEtatIa}
          section={section}
          onSection={setSection}
        />
      )}
    </div>
  )
}
