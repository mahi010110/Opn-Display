import { useState } from 'react'

import type { Rapport } from '../types'
import { Fenetre } from '../ui/Fenetre'
import { Page } from '../ui/Page'
import { go, TonPc } from '../ui/TonPc'

/** La première ouverture : ce que le PC sait faire, ce que l'app installe. */
export function Ouverture(props: { rapport: Rapport; onInstaller: (memoireGo: number | null) => void }) {
  const { rapport } = props
  const { totaleGo, wslGo, propositionGo } = rapport.memoire
  const [plusDeMemoire, setPlusDeMemoire] = useState(false)
  const toutAInstaller = rapport.plateforme === 'windows' && rapport.linuxApp === 'aInstaller'
  const sousLinux = rapport.plateforme === 'linux'

  const titre = sousLinux ? "Ce que l'app prépare" : toutAInstaller ? "Ce que l'app installe" : "Ce qu'il reste à installer"
  const etat = toutAInstaller
    ? "Rien n'est installé pour l'instant."
    : sousLinux
      ? "Incus est là, l'app doit encore s'y préparer."
      : 'Debian est installé, il reste Incus.'

  return (
    <Page titre="Avant ton premier lab" etat={etat}>
      <TonPc verifications={rapport.verifications} />

      <Fenetre titre={titre} active bloc>
        {toutAInstaller && (
          <p className="lu">
            Un Linux à elle, Debian, rangé dans WSL2, avec Incus, qui y fait tourner les machines de tes labs.
          </p>
        )}
        {!toutAInstaller && !sousLinux && (
          <p className="lu">Incus, qui fait tourner les machines de tes labs dans Debian, et l'image de ces machines.</p>
        )}
        {sousLinux && (
          <p className="lu">Une place pour ses machines dans l'Incus de ton Linux, et l'image Debian de ces machines.</p>
        )}
        {toutAInstaller && propositionGo !== null && wslGo !== null && (
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
            onClick={() => props.onInstaller(toutAInstaller && plusDeMemoire ? propositionGo : null)}
          >
            {sousLinux ? 'Préparer' : 'Installer'}
          </button>
        </div>
      </Fenetre>
    </Page>
  )
}
