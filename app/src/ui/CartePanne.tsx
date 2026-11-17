import { useState } from 'react'

import { abandonnerPanne, fermerPanne, indicePanne, verifierPanne } from '../pont'
import type { Niveau, VuePanne } from '../types'

function heure(secondes: number): string {
  return new Date(secondes * 1000).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })
}

function tests(nombre: number): string {
  return `${nombre} ${nombre > 1 ? 'tests' : 'test'}`
}

/**
 * La panne posée, épinglée en haut du panneau de l'IA. Les tests restent
 * cachés jusqu'à la vérification : leurs noms donneraient la réponse.
 */
export function CartePanne(props: {
  panne: VuePanne
  niveau: Niveau
  onPanne: (panne: VuePanne | null) => void
  onDire: (texte: string, sorte: 'ia' | 'note') => void
  onAutre: () => void
}) {
  const { panne } = props
  const [occupe, setOccupe] = useState<string | null>(null)
  const verification = panne.verification
  const reussis = verification?.tests.filter((t) => t.reussi).length ?? 0

  const indice = () => {
    indicePanne()
      .then((texte) => {
        props.onDire(texte, 'ia')
        props.onPanne({ ...panne, indicesDonnes: panne.indicesDonnes + 1, indicesLus: [...panne.indicesLus, texte] })
      })
      .catch((e) => props.onDire(String(e), 'note'))
  }

  const verifier = () => {
    setOccupe('Vérification')
    verifierPanne()
      .then((vue) => {
        props.onPanne(vue)
        if (vue.verification?.reparee) {
          if (vue.verification.explication) props.onDire(vue.verification.explication, 'ia')
          if (vue.inventee) props.onDire("L'IA peut se tromper. Les tests disent ce qu'elle a vérifié, rien de plus.", 'note')
        }
      })
      .catch((e) => props.onDire(String(e), 'note'))
      .finally(() => setOccupe(null))
  }

  const abandonner = () => {
    setOccupe('Retour en arrière')
    abandonnerPanne()
      .then(() => {
        props.onPanne(null)
        props.onDire('Le lab est revenu comme juste avant la panne.', 'note')
      })
      .catch((e) => props.onDire(String(e), 'note'))
      .finally(() => setOccupe(null))
  }

  const autre = () => {
    void fermerPanne()
    props.onPanne(null)
    props.onAutre()
  }

  let badge = <span className="oc-badge oc-badge--danger">Panne</span>
  let tete = `posée à ${heure(panne.posee)}`
  if (verification?.reparee) {
    badge = <span className="oc-badge oc-badge--shared">Réparée</span>
    tete = `${tests(verification.tests.length)} réussis sur ${verification.tests.length}`
  } else if (verification) {
    badge = <span className="oc-badge oc-badge--warn">Pas encore</span>
    tete = `${reussis} ${reussis > 1 ? 'tests réussis' : 'test réussi'} sur ${verification.tests.length}`
  }

  return (
    <div className="ia-epingle">
      <div className="carte">
        <div className="carte-tete">
          {badge}
          {tete}
        </div>
        <div className="carte-corps">
          <p className="carte-titre">{panne.titre}</p>
          {!verification && (
            <p className="note">{tests(panne.tests)} sont prêts. Tu les verras à la vérification.</p>
          )}
          {verification && !verification.reparee && <p className="note">Cherche encore, puis vérifie à nouveau.</p>}
          {verification?.reparee && (
            <ul className="tests">
              {verification.tests.map((t) => (
                <li key={t.nom}>
                  <i className="ic i-ok" aria-hidden="true"></i>
                  {t.nom}
                  <small>
                    {t.machine} : {t.commande}
                  </small>
                </li>
              ))}
            </ul>
          )}
        </div>
        <div className="carte-gestes">
          {verification?.reparee ? (
            <>
              <button className="oc-btn" disabled={occupe !== null} onClick={verifier}>
                <i className="ic i-zero" aria-hidden="true"></i>Relancer les tests
              </button>
              <button className="oc-btn oc-btn--primary" onClick={autre}>
                Une autre panne
              </button>
            </>
          ) : (
            <>
              <button className="oc-btn oc-btn--ghost oc-btn--sm a-gauche" disabled={occupe !== null} onClick={abandonner}>
                Abandonner
              </button>
              {props.niveau !== 'silencieuse' && panne.indicesDonnes < panne.indices && (
                <button className="oc-btn" disabled={occupe !== null} onClick={indice}>
                  <i className="ic i-indice" aria-hidden="true"></i>Un indice
                </button>
              )}
              <button className="oc-btn oc-btn--primary" disabled={occupe !== null} onClick={verifier}>
                {occupe ?? "C'est réparé"}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  )
}
