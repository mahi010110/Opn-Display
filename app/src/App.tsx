import { useCallback, useEffect, useState } from 'react'

import { Bloque } from './ecrans/Bloque'
import { Installation } from './ecrans/Installation'
import { Ouverture } from './ecrans/Ouverture'
import { Pret } from './ecrans/Pret'
import { Verification } from './ecrans/Verification'
import { verifierPc } from './pont'
import type { Rapport } from './types'

export function App() {
  const [rapport, setRapport] = useState<Rapport | null>(null)
  const [erreur, setErreur] = useState<string | null>(null)
  const [installation, setInstallation] = useState<{ memoireGo: number | null } | null>(null)

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
        memoireGo={installation.memoireGo}
        onFini={(nouveau) => {
          setInstallation(null)
          setRapport(nouveau)
        }}
      />
    )
  }

  if (rapport.blocage) return <Bloque rapport={rapport} onVerifier={verifier} />

  if (rapport.linuxApp === 'aInstaller') {
    return <Ouverture rapport={rapport} onInstaller={(memoireGo) => setInstallation({ memoireGo })} />
  }

  return <Pret rapport={rapport} onVerifier={verifier} onRetire={setRapport} />
}
