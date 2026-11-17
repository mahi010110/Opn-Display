import { go } from './TonPc'

const CASES = 10

/** Dix cases, pleines selon la part prise. Une part non nulle en remplit une. */
export function Jauge(props: { part: number }) {
  const part = Math.min(Math.max(props.part, 0), 1)
  const pleines = part > 0 ? Math.max(1, Math.round(part * CASES)) : 0
  return (
    <span className="jauge" aria-hidden="true">
      {Array.from({ length: CASES }, (_, i) => (
        <i key={i} className={i < pleines ? 'plein' : undefined}></i>
      ))}
    </span>
  )
}

const MO = 1024 * 1024

/** Une mémoire en octets, comme on la dit : « 42 Mo », « 1,3 Go ». */
export function texteMemoire(octets: number): string {
  if (octets < 1024 * MO) return `${Math.round(octets / MO)} Mo`
  return `${go(octets / (1024 * MO))} Go`
}
