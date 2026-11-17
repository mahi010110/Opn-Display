import { useEffect, useRef, type ReactNode } from 'react'

/**
 * Une fenêtre posée sur l'écran, le voile en trame derrière. La croix et la
 * touche Échap la ferment. Le focus va sur la croix : sur une question qui
 * détruit quelque chose, la touche Entrée ne doit pas dire oui toute seule.
 */
export function Dialogue(props: { titre: string; onFermer: () => void; pied: ReactNode; children: ReactNode }) {
  const croix = useRef<HTMLButtonElement>(null)
  const { onFermer } = props

  useEffect(() => {
    croix.current?.focus()
    const touche = (evenement: KeyboardEvent) => {
      if (evenement.key === 'Escape') onFermer()
    }
    window.addEventListener('keydown', touche)
    return () => window.removeEventListener('keydown', touche)
  }, [onFermer])

  return (
    <div className="oc-overlay">
      <section className="oc-window dialogue" role="dialog" aria-modal="true" aria-label={props.titre}>
        <header className="oc-window__bar oc-window__bar--accent">
          <h2 className="oc-window__title">{props.titre}</h2>
          <button ref={croix} className="oc-window__btn" aria-label="Fermer" onClick={props.onFermer}>
            <i className="ic i-fermer" aria-hidden="true"></i>
          </button>
        </header>
        <div className="oc-window__body">
          {props.children}
          <div className="oc-dialog__foot">{props.pied}</div>
        </div>
      </section>
    </div>
  )
}
