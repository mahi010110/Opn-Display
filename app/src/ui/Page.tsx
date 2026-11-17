import type { ReactNode } from 'react'

/** Le cadre des écrans de la première ouverture : pas encore de menu. */
export function Page(props: { titre: string; etat: string; seule?: boolean; children: ReactNode }) {
  return (
    <div className="app">
      <header className="barre">
        <span className="marque">Opn-Display</span>
      </header>
      <div className="lab-barre">
        <h1 className="lab-nom">{props.titre}</h1>
      </div>
      <main className={'page ' + (props.seule ? 'page--seule' : 'page--paire')}>{props.children}</main>
      <footer className="etat">
        <span>{props.etat}</span>
      </footer>
    </div>
  )
}
