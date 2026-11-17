import type { Etat, Verification } from '../types'
import { Fenetre } from './Fenetre'

const ICONE: Record<Etat, string> = {
  ok: 'i-ok',
  manque: 'i-alerte manque',
  inconnu: 'i-horloge inconnu',
}

export function Verifications(props: { verifications: Verification[] }) {
  return (
    <ul className="tests tests--large">
      {props.verifications.map((v) => (
        <li key={v.nom}>
          <i className={'ic ' + ICONE[v.etat]} aria-hidden="true"></i>
          {v.nom}
          <small>{v.detail}</small>
        </li>
      ))}
    </ul>
  )
}

export function TonPc(props: { verifications: Verification[] }) {
  return (
    <Fenetre titre="Ton PC">
      <Verifications verifications={props.verifications} />
    </Fenetre>
  )
}

/** Un nombre de Go comme on l'écrit : « 32 », « 24,3 ». */
export function go(valeur: number): string {
  const arrondi = Math.round(valeur * 10) / 10
  return Number.isInteger(arrondi) ? String(arrondi) : arrondi.toFixed(1).replace('.', ',')
}
