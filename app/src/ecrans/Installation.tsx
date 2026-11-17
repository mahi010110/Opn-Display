import { useEffect, useRef, useState } from 'react'

import { arreterInstallation, installer, suivreInstallation } from '../pont'
import type { Etape, EtatEtape, Pas, Rapport } from '../types'
import { Fenetre } from '../ui/Fenetre'
import { Page } from '../ui/Page'
import { TonPc } from '../ui/TonPc'

const CLASSE: Record<EtatEtape, string> = {
  enCours: 'en-cours',
  fait: 'fait',
  saute: 'saute',
  erreur: 'erreur',
}

const PAVES = 16

function nom(pas: Pas, memoireGo: number | null, rapport: Rapport): string {
  switch (pas) {
    case 'telecharger':
      return 'Télécharger Debian'
    case 'importer':
      return "L'installer dans WSL2"
    case 'memoire':
      return memoireGo !== null ? `Passer WSL2 à ${memoireGo} Go` : 'Garder la mémoire de WSL2'
    case 'demarrer':
      return 'Premier démarrage'
    case 'incus':
      return rapport.plateforme === 'windows' ? 'Installer Incus' : 'Préparer Incus'
    case 'image':
      return "Télécharger l'image des machines"
  }
}

export function Installation(props: {
  rapport: Rapport
  pas: Pas[]
  memoireGo: number | null
  onFini: (rapport: Rapport) => void
}) {
  const [etapes, setEtapes] = useState<Partial<Record<Pas, Etape>>>({})
  const [erreur, setErreur] = useState<string | null>(null)
  const [arret, setArret] = useState(false)
  const [essai, setEssai] = useState(0)
  const onFini = useRef(props.onFini)
  onFini.current = props.onFini

  useEffect(() => {
    let abandonne = false
    let arreterDEcouter: (() => void) | undefined
    setErreur(null)
    setArret(false)
    suivreInstallation((etape) => {
      setEtapes((avant) => ({ ...avant, [etape.pas]: etape }))
    })
      .then((arreter) => {
        arreterDEcouter = arreter
        return installer(props.memoireGo)
      })
      .then((rapport) => {
        if (!abandonne) onFini.current(rapport)
      })
      .catch((e) => {
        if (!abandonne) setErreur(String(e))
      })
    return () => {
      abandonne = true
      arreterDEcouter?.()
    }
  }, [essai, props.memoireGo])

  const enCours = props.pas.findIndex((pas) => etapes[pas]?.etat === 'enCours')
  const etat = erreur
    ? "L'installation s'est arrêtée."
    : `Installation en cours, étape ${Math.max(enCours, 0) + 1} sur ${props.pas.length}`

  return (
    <Page titre="Avant ton premier lab" etat={etat}>
      <TonPc verifications={props.rapport.verifications} />

      <Fenetre titre="Installation" active bloc>
        <ul className="etapes">
          {props.pas.map((pas) => {
            const etape = etapes[pas]
            return (
              <li key={pas} className={etape ? CLASSE[etape.etat] : ''}>
                {nom(pas, props.memoireGo, props.rapport)}
                {etape?.etat === 'enCours' && etape.avance !== null && <Avance part={etape.avance} />}
                {etape && etape.detail !== '' && <small>{etape.detail}</small>}
              </li>
            )
          })}
        </ul>
        {erreur === null ? (
          <>
            <p className="note">Tu peux faire autre chose pendant ce temps.</p>
            <div className="pied">
              <button
                className="oc-btn"
                disabled={arret}
                onClick={() => {
                  setArret(true)
                  arreterInstallation().catch(() => setArret(false))
                }}
              >
                {arret ? 'Arrêt en cours' : "Arrêter l'installation"}
              </button>
            </div>
          </>
        ) : (
          <div className="pied">
            <button className="oc-btn oc-btn--primary" onClick={() => setEssai((n) => n + 1)}>
              Réessayer
            </button>
          </div>
        )}
      </Fenetre>
    </Page>
  )
}

function Avance(props: { part: number }) {
  const pleins = Math.round(Math.min(Math.max(props.part, 0), 1) * PAVES)
  return (
    <span className="pas pas--grand" role="progressbar" aria-valuenow={Math.round(props.part * 100)} aria-valuemin={0} aria-valuemax={100}>
      {Array.from({ length: PAVES }, (_, i) => (
        <i key={i} className={i < pleins ? 'fait' : undefined}></i>
      ))}
    </span>
  )
}
