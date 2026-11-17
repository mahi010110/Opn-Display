import { useCallback, useEffect, useState } from 'react'

import { Bloque } from './ecrans/Bloque'
import { Installation } from './ecrans/Installation'
import { Ouverture } from './ecrans/Ouverture'
import { Verification } from './ecrans/Verification'
import { verifierPc } from './pont'
import type { Pas, Rapport } from './types'
import { Coque } from './ui/Coque'

/** Les étapes qui restent à faire, dans l'ordre. */
function etapes(rapport: Rapport): Pas[] {
  if (rapport.plateforme === 'windows' && rapport.linuxApp === 'aInstaller') {
    return ['telecharger', 'importer', 'memoire', 'demarrer', 'incus', 'image']
  }
  return ['incus', 'image']
}

export function App() {
  const [rapport, setRapport] = useState<Rapport | null>(null)
  const [erreur, setErreur] = useState<string | null>(null)
  const [installation, setInstallation] = useState<{ memoireGo: number | null; pas: Pas[] } | null>(null)

  const verifier = useCallback(() => {
    setRapport(null)
    setErreur(null)
    verifierPc()
      .then(setRapport)
      .catch((e) => setErreur(String(e)))
  }, [])

  useEffect(verifier, [verifier])

  if (!rapport) return <Verification erreur={erreur} onReessayer={verifier} />

  if (installation) {
    return (
      <Installation
        rapport={rapport}
        pas={installation.pas}
        memoireGo={installation.memoireGo}
        onFini={(nouveau) => {
          setInstallation(null)
          setRapport(nouveau)
        }}
      />
    )
  }

  if (rapport.blocage) return <Bloque rapport={rapport} onVerifier={verifier} />

  if (rapport.machines !== 'pretes') {
    return (
      <Ouverture
        rapport={rapport}
        onInstaller={(memoireGo) => setInstallation({ memoireGo, pas: etapes(rapport) })}
      />
    )
  }

  return <Coque rapport={rapport} onRapport={setRapport} />
}
