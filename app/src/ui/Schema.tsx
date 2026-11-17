import { useEffect, useRef, useState } from 'react'

import type { EtatMachine, Lab, MachineLab, Sorte } from '../types'

const ICONE: Record<Sorte, string> = {
  poste: 'i-poste',
  serveur: 'i-serveur',
  routeur: 'i-routeur',
  switch: 'i-switch',
  parFeu: 'i-parefeu',
  stockage: 'i-stockage',
}

/** Ce qu'une machine prend autour de son centre, en pixels : la moitié de sa
 * case, et dessous sa case, son nom et ses adresses. */
const DEMI_CASE = 24
const SOUS_LA_CASE = 62

type Point = { x: number; y: number }
type Etiquette = Point & { texte: string }
type Port = Etiquette & { cote: boolean }
/** Sur la ligne d'un switch, le réseau s'écrit sur la ligne s'il y a la
 * place entre deux descentes, sinon juste dessous. */
type Bus = Etiquette & { place: 'sur' | 'dessous' | 'cote' }

/** La largeur d'une étiquette de réseau, en pixels : la police mono fait
 * 6,6 pixels par caractère à 11 pixels, plus ses marges. */
const largeurReseau = (texte: string) => texte.length * 6.6 + 12

interface Trace {
  points: Point[]
  reseau: Etiquette | null
  ports: Port[]
}

const duReseau = (m: MachineLab) => m.sorte === 'routeur' || m.sorte === 'switch'

/** Le câble descend du switch `haut` vers une machine posée sous lui. */
function descend(haut: MachineLab, bas: MachineLab): boolean {
  return haut.sorte === 'switch' && !duReseau(bas) && bas.y > haut.y + 5
}

/**
 * Le tracé de chaque câble. Les machines d'un switch descendent d'une même
 * ligne, qui porte leur réseau une seule fois. Un câble qui passerait sur
 * une machine la contourne par le haut.
 */
export function tracer(lab: Lab, largeur: number, hauteur: number): { traces: Trace[]; bus: Bus[] } {
  const place = new Map(lab.machines.map((m) => [m.nom, m]))
  const enPixels = (p: Point) => ({ x: (p.x * largeur) / 100, y: (p.y * hauteur) / 100 })
  const enCent = (x: number, y: number): Point => ({ x: (x * 100) / largeur, y: (y * 100) / hauteur })

  /** Où écrire le nom d'une interface : sur le câble qui part de `a` vers
   * `b`, au-delà de la case et des étiquettes de la machine. */
  const pres = (a: Point, b: Point): Point & { cote: boolean } => {
    const debut = enPixels(a)
    const fin = enPixels(b)
    const dx = fin.x - debut.x
    const dy = fin.y - debut.y
    const longueur = Math.hypot(dx, dy) || 1
    const vertical = Math.abs(dy) > Math.abs(dx)
    let r = vertical ? (dy > 0 ? SOUS_LA_CASE + 12 : DEMI_CASE + 18) : Math.max(DEMI_CASE + 24, longueur * 0.2)
    r = Math.min(r, longueur * 0.45)
    return { ...enCent(debut.x + (dx * r) / longueur, debut.y + (dy * r) / longueur), cote: vertical }
  }

  const bouts = lab.cables.flatMap((cable) => {
    const de = place.get(cable.de.machine)
    const vers = place.get(cable.vers.machine)
    return de && vers ? [{ cable, de, vers }] : []
  })
  const descentes = new Map<string, { x: number; y: number; xs: number[]; reseau: string | null }>()
  for (const { cable, de, vers } of bouts) {
    const [sw, bas] = descend(de, vers) ? [de, vers] : descend(vers, de) ? [vers, de] : [null, null]
    if (!sw || !bas) continue
    const ligne = descentes.get(sw.nom) ?? { x: sw.x, y: (sw.y + bas.y) / 2, xs: [], reseau: null }
    ligne.xs.push(bas.x)
    ligne.reseau ??= cable.reseau
    descentes.set(sw.nom, ligne)
  }

  const traces = bouts.map(({ cable, de, vers }): Trace => {
    const [sw, bas] = descend(de, vers) ? [de, vers] : descend(vers, de) ? [vers, de] : [null, null]
    if (sw && bas) {
      const y = descentes.get(sw.nom)?.y ?? (sw.y + bas.y) / 2
      const interface_ = bas === de ? cable.de.interface : cable.vers.interface
      // Entre la ligne du switch et le haut de la case : le schéma du lab
      // est bas, et ce nom ne doit toucher ni l'une ni l'autre.
      const hautDeLaCase = bas.y - (DEMI_CASE * 100) / hauteur
      return {
        points: [sw, { x: sw.x, y }, { x: bas.x, y }, bas],
        reseau: null,
        ports: [{ x: bas.x, y: (y + hautDeLaCase) / 2, cote: true, texte: interface_ }],
      }
    }
    const entre =
      Math.abs(de.y - vers.y) < 1 &&
      lab.machines.some(
        (m) =>
          m !== de &&
          m !== vers &&
          Math.abs(m.y - de.y) < 1 &&
          m.x > Math.min(de.x, vers.x) + 0.5 &&
          m.x < Math.max(de.x, vers.x) - 0.5,
      )
    if (entre) {
      const y = Math.max(4, de.y - 16)
      const ecart = (vers.x - de.x) * 0.12
      return {
        points: [de, { x: de.x, y }, { x: vers.x, y }, vers],
        reseau: cable.reseau ? { x: (de.x + vers.x) / 2, y, texte: cable.reseau } : null,
        ports: [
          { x: de.x + ecart, y, cote: false, texte: cable.de.interface },
          { x: vers.x - ecart, y, cote: false, texte: cable.vers.interface },
        ],
      }
    }
    // Le réseau d'un switch qui a ses machines dessous est écrit sur sa ligne.
    const surSaLigne = [de, vers].some((m) => m.sorte === 'switch' && descentes.has(m.nom))
    return {
      points: [de, vers],
      reseau:
        cable.reseau && !surSaLigne ? { x: (de.x + vers.x) / 2, y: (de.y + vers.y) / 2, texte: cable.reseau } : null,
      ports: [
        { ...pres(de, vers), texte: cable.de.interface },
        { ...pres(vers, de), texte: cable.vers.interface },
      ],
    }
  })

  const bus = [...descentes.values()].flatMap((ligne): Bus[] => {
    if (!ligne.reseau) return []
    const texte = ligne.reseau
    const besoin = largeurReseau(texte) + 24
    const ecarts = (xs: number[]) => {
      const tries = [...new Set(xs)].sort((a, b) => a - b)
      return tries.slice(1).map((b, i) => ({ a: tries[i], b }))
    }
    const plusLarge = (liste: { a: number; b: number }[]) =>
      liste.reduce<{ a: number; b: number } | null>((m, e) => (!m || e.b - e.a > m.b - m.a ? e : m), null)
    // Sur la ligne, entre deux descentes, la jonction du switch comprise.
    const sur = plusLarge(ecarts([...ligne.xs, ligne.x]))
    if (sur && ((sur.b - sur.a) * largeur) / 100 >= besoin) {
      return [{ x: (sur.a + sur.b) / 2, y: ligne.y, texte, place: 'sur' }]
    }
    // Dessous, rien ne descend entre deux machines voisines.
    const dessous = plusLarge(ecarts(ligne.xs))
    if (dessous) return [{ x: (dessous.a + dessous.b) / 2, y: ligne.y, texte, place: 'dessous' }]
    const seule = ligne.xs[0]
    return Math.abs(seule - ligne.x) < 2
      ? [{ x: seule, y: ligne.y, texte, place: 'cote' }]
      : [{ x: (seule + ligne.x) / 2, y: ligne.y, texte, place: 'dessous' }]
  })
  return { traces, bus }
}

function adresses(machine: MachineLab): string {
  return machine.interfaces
    .flatMap((i) => (i.adresse ? [i.adresse.split('/')[0]] : []))
    .join(' · ')
}

/**
 * Le schéma du lab : les machines, leurs câbles, et le réseau de chacun. En
 * projet, il montre un lab proposé : rien n'y tourne, rien ne s'ouvre.
 */
export function Schema(props: {
  lab: Lab
  machines: EtatMachine[]
  choisie?: string | null
  onChoisir?: (machine: string) => void
  projet?: boolean
}) {
  const { lab } = props
  const plan = useRef<HTMLDivElement>(null)
  const [taille, setTaille] = useState({ largeur: 950, hauteur: 245 })

  useEffect(() => {
    const element = plan.current
    if (!element) return
    const observateur = new ResizeObserver(([entree]) => {
      const { width, height } = entree.contentRect
      if (width > 0 && height > 0) setTaille({ largeur: width, hauteur: height })
    })
    observateur.observe(element)
    return () => observateur.disconnect()
  }, [])

  const { traces, bus } = tracer(lab, taille.largeur, taille.hauteur)
  const ecrire = (e: Etiquette, classe: string, cle: string) => (
    <span key={cle} className={classe} style={{ left: `${e.x}%`, top: `${e.y}%` }}>
      {e.texte}
    </span>
  )

  return (
    <div className={props.projet ? 'plan plan--projet' : 'plan'} ref={plan}>
      <svg className="cables" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
        {traces.map((trace, i) => (
          <polyline key={i} points={trace.points.map((p) => `${p.x},${p.y}`).join(' ')} />
        ))}
      </svg>
      {traces.map((trace, i) => trace.reseau && ecrire(trace.reseau, 'reseau', `reseau-${i}`))}
      {bus.map((ligne, i) => ecrire(ligne, ligne.place === 'sur' ? 'reseau' : `reseau reseau--${ligne.place}`, `bus-${i}`))}
      {traces.flatMap((trace, i) =>
        trace.ports.map((port, j) => ecrire(port, port.cote ? 'port port--cote' : 'port', `port-${i}-${j}`)),
      )}
      {lab.machines.map((machine) => {
        if (props.projet) {
          return (
            <div key={machine.nom} className="machine" style={{ left: `${machine.x}%`, top: `${machine.y}%` }}>
              <span className="machine-case">
                <i className={'ic ic-24 ' + ICONE[machine.sorte]} aria-hidden="true"></i>
              </span>
              <span className="machine-nom">{machine.nom}</span>
              <span className="machine-ip">{adresses(machine)}</span>
            </div>
          )
        }
        const marche = props.machines.find((m) => m.nom === machine.nom)?.marche ?? 'absente'
        const allumee = marche === 'enMarche'
        const classes = [
          'machine',
          allumee ? '' : 'machine--eteinte',
          props.choisie === machine.nom ? 'machine--choisie' : '',
        ].join(' ')
        return (
          <button
            key={machine.nom}
            className={classes.trim()}
            style={{ left: `${machine.x}%`, top: `${machine.y}%` }}
            onClick={() => props.onChoisir?.(machine.nom)}
            aria-label={`${machine.nom}, ${allumee ? 'en marche' : 'éteinte'}. Ouvrir son terminal`}
          >
            <span className="machine-case">
              <i className={'ic ic-24 ' + ICONE[machine.sorte]} aria-hidden="true"></i>
            </span>
            <span className="machine-nom">{machine.nom}</span>
            <span className="machine-ip">{allumee ? adresses(machine) : 'éteinte'}</span>
          </button>
        )
      })}
    </div>
  )
}
