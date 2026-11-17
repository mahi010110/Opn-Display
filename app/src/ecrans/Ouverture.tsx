import { useState } from 'react'

import type { Rapport } from '../types'
import { Fenetre } from '../ui/Fenetre'
import { Page } from '../ui/Page'
import { go, TonPc } from '../ui/TonPc'

export function Ouverture(props: { rapport: Rapport; onInstaller: (memoireGo: number | null) => void }) {
  const { totaleGo, wslGo, propositionGo } = props.rapport.memoire
  const [plusDeMemoire, setPlusDeMemoire] = useState(false)

  return (
    <Page titre="Avant ton premier lab" etat="Rien n'est installé pour l'instant.">
      <TonPc verifications={props.rapport.verifications} />

      <Fenetre titre="Ce que l'app installe" active bloc>
        <p className="lu">Un Linux à elle, Debian, rangé dans WSL2. Les machines de tes labs tournent dedans.</p>
        {propositionGo !== null && wslGo !== null && (
          <fieldset className="oc-fieldset">
            <legend>Un réglage de Windows</legend>
            <p>
              WSL2 ne prend que {go(Math.round(wslGo))} Go sur tes {go(Math.round(totaleGo))}. Avec {propositionGo} Go,
              tu lances de plus gros labs.
            </p>
            <p className="note">
              Ce réglage vaut pour tout WSL2 sur ce PC. L'app ajoute cette ligne au fichier .wslconfig, puis redémarre
              WSL2 : tes autres Linux s'arrêtent un instant.
            </p>
            <pre className="ligne">memory={propositionGo}GB</pre>
            <label className="oc-check">
              <input
                type="checkbox"
                checked={plusDeMemoire}
                onChange={(e) => setPlusDeMemoire(e.target.checked)}
              />
              Passer à {propositionGo} Go
            </label>
          </fieldset>
        )}
        <div className="pied">
          <button
            className="oc-btn oc-btn--primary"
            onClick={() => props.onInstaller(plusDeMemoire ? propositionGo : null)}
          >
            Installer
          </button>
        </div>
      </Fenetre>
    </Page>
  )
}
