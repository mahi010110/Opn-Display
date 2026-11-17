import { Fenetre } from '../ui/Fenetre'
import { Page } from '../ui/Page'

/** Le temps de regarder le PC, ou ce qui a empêché de le faire. */
export function Verification(props: { erreur: string | null; onReessayer: () => void }) {
  return (
    <Page titre="Avant ton premier lab" etat="Rien n'est installé pour l'instant." seule>
      <Fenetre titre="Ton PC" active={props.erreur !== null} bloc>
        {props.erreur === null ? (
          <p className="mono">L'app regarde ce que ton PC sait faire.</p>
        ) : (
          <>
            <p className="note erreur">La vérification n'a pas abouti : {props.erreur}</p>
            <div className="pied">
              <button className="oc-btn oc-btn--primary" onClick={props.onReessayer}>
                Réessayer
              </button>
            </div>
          </>
        )}
      </Fenetre>
    </Page>
  )
}
