import { reglerIa } from '../pont'
import type { Comportements, EtatIa, Facon, Initiative, Liberte, Niveau, Solution } from '../types'
import { NIVEAUX, NOM_NIVEAU } from '../ui/reglageIa'

const FACONS: { facon: Facon; nom: string }[] = [
  { facon: 'parNiveau', nom: 'Par niveau' },
  { facon: 'unParUn', nom: 'Un par un' },
  { facon: 'sAdapte', nom: "Elle s'adapte" },
]

interface Option<T> {
  valeur: T
  nom: string
  dit: string
}

const DANS_LE_LAB: Option<Liberte>[] = [
  { valeur: 'avecAccord', nom: 'Avec accord', dit: "elle demande avant d'agir" },
  { valeur: 'libre', nom: 'Libre', dit: 'elle agit sans demander' },
]

const SOLUTION: Option<Solution>[] = [
  { valeur: 'donnee', nom: 'Elle la donne', dit: "et l'explique" },
  { valeur: 'indices', nom: 'Des indices', dit: 'jamais la cause' },
  { valeur: 'rien', nom: 'Rien', dit: "comme à l'examen" },
]

const INITIATIVE: Option<Initiative>[] = [
  { valeur: 'quandTuBloques', nom: 'Quand tu bloques', dit: '10 minutes sur une panne sans rien lui demander' },
  { valeur: 'jamais', nom: 'Jamais', dit: 'seulement quand tu lui écris' },
]

const DIT_LAB: Record<Liberte, string> = {
  avecAccord: 'elle agit avec ton accord',
  libre: 'elle agit sans demander',
}

const DIT_SOLUTION: Record<Solution, string> = {
  donnee: "elle la donne et l'explique",
  indices: 'des indices, jamais la cause',
  rien: "rien, comme à l'examen",
}

const DIT_INITIATIVE: Record<Initiative, string> = {
  quandTuBloques: 'quand tu bloques sur une panne',
  jamais: 'jamais, seulement quand tu lui écris',
}

function Fiche(props: { comportements: Comportements }) {
  const c = props.comportements
  return (
    <dl className="fiche">
      <dt>Dans le lab</dt>
      <dd>{DIT_LAB[c.dansLeLab]}</dd>
      <dt>Quand tu demandes la solution</dt>
      <dd>{DIT_SOLUTION[c.solution]}</dd>
      <dt>Elle te parle d'elle-même</dt>
      <dd>{DIT_INITIATIVE[c.initiative]}</dd>
    </dl>
  )
}

function Choix<T extends string>(props: {
  nom: string
  valeur: T
  options: Option<T>[]
  onChoix: (valeur: T) => void
}) {
  return (
    <div>
      <p className="reglage-nom">{props.nom}</p>
      <div className="choix" role="radiogroup" aria-label={props.nom}>
        {props.options.map((option) => (
          <label key={option.valeur} className="choix-ligne">
            <span className="oc-radio">
              <input
                type="radio"
                name={props.nom}
                checked={props.valeur === option.valeur}
                onChange={() => props.onChoix(option.valeur)}
              />
            </span>
            {option.nom}
            <span className="mono">{option.dit}</span>
          </label>
        ))}
      </div>
    </div>
  )
}

/**
 * Réglages, Régler l'IA : par niveau, un comportement à la fois, ou une IA
 * dont l'app change le niveau d'après tes pannes. Sa liberté sur le PC
 * viendra avec ses outils sur le PC.
 */
export function ReglerIa(props: { etatIa: EtatIa; onEtatIa: (etat: EtatIa) => void }) {
  const { reglages, comportements } = props.etatIa
  const regler = (facon: Facon, niveau: Niveau, unParUn: Comportements) => {
    reglerIa(facon, niveau, unParUn)
      .then(props.onEtatIa)
      .catch(() => undefined)
  }

  const changerDeFacon = (facon: Facon) => {
    if (facon === reglages.facon) return
    // Un par un part de ce qu'elle fait en ce moment ; par niveau, du niveau
    // où elle en est.
    const niveau = props.etatIa.niveauDuMoment ?? reglages.niveau
    regler(facon, niveau, facon === 'unParUn' ? comportements : reglages.unParUn)
  }

  const changer = (changement: Partial<Comportements>) =>
    regler('unParUn', reglages.niveau, { ...reglages.unParUn, ...changement })

  return (
    <section className="oc-window">
      <header className="oc-window__bar oc-window__bar--accent">
        <h2 className="oc-window__title">Régler l'IA</h2>
      </header>
      <div className="oc-window__body bloc">
        <fieldset className="oc-fieldset bloc">
          <legend>Avec toi</legend>
          <div className="rang">
            <div className="segments" role="group" aria-label="Façon de régler">
              {FACONS.map(({ facon, nom }) => (
                <button key={facon} aria-pressed={reglages.facon === facon} onClick={() => changerDeFacon(facon)}>
                  {nom}
                </button>
              ))}
            </div>
            {reglages.facon === 'parNiveau' && (
              <div className="segments" role="group" aria-label="Niveau">
                {NIVEAUX.map(({ niveau, nom }) => (
                  <button
                    key={niveau}
                    aria-pressed={reglages.niveau === niveau}
                    onClick={() => regler('parNiveau', niveau, reglages.unParUn)}
                  >
                    {nom}
                  </button>
                ))}
              </div>
            )}
          </div>

          {reglages.facon === 'parNiveau' && <Fiche comportements={comportements} />}

          {reglages.facon === 'unParUn' && (
            <>
              <Choix
                nom="Dans le lab"
                valeur={reglages.unParUn.dansLeLab}
                options={DANS_LE_LAB}
                onChoix={(dansLeLab) => changer({ dansLeLab })}
              />
              <Choix
                nom="Quand tu demandes la solution"
                valeur={reglages.unParUn.solution}
                options={SOLUTION}
                onChoix={(solution) => changer({ solution })}
              />
              <Choix
                nom="Elle te parle d'elle-même"
                valeur={reglages.unParUn.initiative}
                options={INITIATIVE}
                onChoix={(initiative) => changer({ initiative })}
              />
            </>
          )}

          {reglages.facon === 'sAdapte' && (
            <>
              <p className="lu">
                Son niveau du moment : <strong>{NOM_NIVEAU[props.etatIa.niveauDuMoment ?? 'coach']}</strong>
              </p>
              {reglages.adaptation.raison !== '' && <p className="note">{reglages.adaptation.raison}</p>}
              <Fiche comportements={comportements} />
              <p className="note">
                Deux pannes réparées sans indice : elle aide moins. Un abandon, ou deux indices sur une panne : elle
                aide plus.
              </p>
            </>
          )}
        </fieldset>
      </div>
    </section>
  )
}
