import type { ReactNode } from 'react'

/**
 * La fenêtre du kit d'OpenContact. Barre marine pour celle où l'on agit,
 * barre en trame pour les autres.
 */
export function Fenetre(props: {
  titre: string
  active?: boolean
  bloc?: boolean
  flush?: boolean
  children: ReactNode
}) {
  const barre = props.active ? 'oc-window__bar--accent' : 'oc-window__bar--inactive'
  const corps = 'oc-window__body' + (props.bloc ? ' bloc' : '') + (props.flush ? ' oc-window__body--flush' : '')
  return (
    <section className="oc-window">
      <header className={'oc-window__bar ' + barre}>
        <h2 className="oc-window__title">{props.titre}</h2>
      </header>
      <div className={corps}>{props.children}</div>
    </section>
  )
}
