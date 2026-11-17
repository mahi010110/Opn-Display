import { reglerIa } from '../pont'
import type { EtatIa, Liberte, Niveau } from '../types'

const NIVEAUX: { niveau: Niveau; nom: string; dit: string }[] = [
  { niveau: 'prof', nom: 'Prof', dit: 'Elle explique tout, et donne la solution si tu la demandes.' },
  { niveau: 'coach', nom: 'Coach', dit: "Des questions et des indices, jamais la solution d'emblée." },
  { niveau: 'silencieuse', nom: 'Silencieuse', dit: 'Elle se tait tant que tu ne lui parles pas, sans indice.' },
]

/** Réglages, Régler l'IA : son niveau avec toi, et sa liberté dans le lab. */
export function ReglerIa(props: { etatIa: EtatIa; onEtatIa: (etat: EtatIa) => void }) {
  const { niveau, dansLeLab } = props.etatIa.reglages
  const regler = (nouveauNiveau: Niveau, liberte: Liberte) => {
    reglerIa(nouveauNiveau, liberte)
      .then(props.onEtatIa)
      .catch(() => undefined)
  }
  const choisi = NIVEAUX.find((n) => n.niveau === niveau)

  return (
    <section className="oc-window">
      <header className="oc-window__bar oc-window__bar--accent">
        <h2 className="oc-window__title">Régler l'IA</h2>
      </header>
      <div className="oc-window__body bloc">
        <fieldset className="oc-fieldset bloc">
          <legend>Avec toi</legend>
          <div className="segments" role="group" aria-label="Niveau de l'IA">
            {NIVEAUX.map((n) => (
              <button key={n.niveau} aria-pressed={niveau === n.niveau} onClick={() => regler(n.niveau, dansLeLab)}>
                {n.nom}
              </button>
            ))}
          </div>
          {choisi && <p>{choisi.dit}</p>}
        </fieldset>
        <fieldset className="oc-fieldset bloc">
          <legend>Dans le lab</legend>
          <div className="choix" role="radiogroup" aria-label="Dans le lab">
            <label className="choix-ligne">
              <span className="oc-radio">
                <input
                  type="radio"
                  name="lab"
                  checked={dansLeLab === 'avecAccord'}
                  onChange={() => regler(niveau, 'avecAccord')}
                />
              </span>
              Avec accord<span className="mono">elle demande avant d'agir</span>
            </label>
            <label className="choix-ligne">
              <span className="oc-radio">
                <input type="radio" name="lab" checked={dansLeLab === 'libre'} onChange={() => regler(niveau, 'libre')} />
              </span>
              Libre<span className="mono">elle agit sans demander</span>
            </label>
          </div>
        </fieldset>
      </div>
    </section>
  )
}
