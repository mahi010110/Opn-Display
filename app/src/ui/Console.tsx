import { FitAddon } from '@xterm/addon-fit'
import { Terminal, type ITheme } from '@xterm/xterm'
import '@xterm/xterm/css/xterm.css'
import { useEffect, useRef, useState } from 'react'

import { ouvrirTerminal, type Session } from '../pont'
import type { Marche } from '../types'

const POLICE = '"IBM Plex Mono", monospace'
const TAILLE = 13

/** Les couleurs du terminal viennent des jetons de la barre de titre. */
function couleurs(): ITheme {
  const style = getComputedStyle(document.documentElement)
  const jeton = (nom: string) => style.getPropertyValue(nom).trim()
  return {
    background: jeton('--titlebar-bg'),
    foreground: jeton('--titlebar-text'),
    cursor: jeton('--titlebar-text'),
    cursorAccent: jeton('--titlebar-bg'),
    selectionBackground: jeton('--titlebar-accent-bg'),
    // Le bleu d'origine ne se lit pas sur ce fond presque noir.
    blue: '#6EA8FF',
    brightBlue: '#9CC4FF',
  }
}

type Etat = 'attente' | 'ouverte' | 'finie'

/**
 * Le terminal d'une machine. Il s'ouvre quand la machine tourne, et garde ce
 * qu'il a affiché quand le shell s'arrête.
 */
export function Console(props: {
  lab: string
  machine: string
  marche: Marche
  visible: boolean
  occupe: boolean
  onDemarrer: () => void
}) {
  const boite = useRef<HTMLDivElement>(null)
  const terminal = useRef<Terminal | null>(null)
  const ajusteur = useRef<FitAddon | null>(null)
  const session = useRef<Session | null>(null)
  const generation = useRef(0)
  const visible = useRef(props.visible)
  visible.current = props.visible
  const [pret, setPret] = useState(false)
  const [etat, setEtat] = useState<Etat>('attente')
  const [voulue, setVoulue] = useState(true)

  useEffect(() => {
    const term = new Terminal({
      fontFamily: POLICE,
      fontSize: TAILLE,
      lineHeight: 1.15,
      cursorBlink: false,
      scrollback: 5000,
      theme: couleurs(),
    })
    const fit = new FitAddon()
    term.loadAddon(fit)
    terminal.current = term
    ajusteur.current = fit

    term.onData((donnees) => session.current?.ecrire(donnees))
    term.onResize(({ cols, rows }) => session.current?.redimensionner(cols, rows))
    // Ctrl+C reste à la machine : copier et coller prennent Maj en plus,
    // comme dans les terminaux de Linux.
    term.attachCustomKeyEventHandler((touche) => {
      if (touche.type !== 'keydown' || !touche.ctrlKey || !touche.shiftKey) return true
      if (touche.code === 'KeyC') {
        const choisi = term.getSelection()
        if (choisi) void navigator.clipboard.writeText(choisi)
        return false
      }
      if (touche.code === 'KeyV') {
        navigator.clipboard
          .readText()
          .then((texte) => term.paste(texte))
          .catch(() => undefined)
        return false
      }
      return true
    })

    const theme = new MutationObserver(() => {
      term.options.theme = couleurs()
    })
    theme.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] })

    let fini = false
    // La police doit être là avant la mesure des cases, sinon les colonnes
    // sont fausses.
    document.fonts
      .load(`${TAILLE}px ${POLICE}`)
      .catch(() => undefined)
      .then(() => {
        if (fini || !boite.current) return
        term.open(boite.current)
        fit.fit()
        setPret(true)
      })

    const taille = new ResizeObserver(() => {
      if (boite.current?.offsetParent) fit.fit()
    })
    if (boite.current) taille.observe(boite.current)

    return () => {
      fini = true
      generation.current += 1
      taille.disconnect()
      theme.disconnect()
      session.current?.fermer()
      session.current = null
      term.dispose()
    }
  }, [])

  const enMarche = props.marche === 'enMarche'

  useEffect(() => {
    const term = terminal.current
    if (!pret || !term || !enMarche || !voulue || etat === 'ouverte') return
    // Chaque ouverture a son numéro : ce qu'envoie une session dépassée, ou
    // arrivée après la fermeture de l'onglet, est ignoré.
    const ici = ++generation.current
    setEtat('ouverte')
    ouvrirTerminal(
      props.lab,
      props.machine,
      term.cols,
      term.rows,
      (octets) => {
        if (generation.current === ici) term.write(octets)
      },
      () => {
        if (generation.current !== ici) return
        session.current = null
        setEtat('finie')
        setVoulue(false)
      },
    )
      .then((nouvelle) => {
        if (generation.current !== ici) {
          nouvelle.fermer()
          return
        }
        session.current = nouvelle
        ajusteur.current?.fit()
        nouvelle.redimensionner(term.cols, term.rows)
        // Le bouton qui a démarré la machine a disparu avec elle : les
        // touches doivent aller au terminal.
        if (visible.current) term.focus()
      })
      .catch((erreur) => {
        if (generation.current !== ici) return
        term.write(`\r\n${String(erreur)}\r\n`)
        setEtat('finie')
        setVoulue(false)
      })
    // L'état n'entre pas ici : une session ouverte ne se rouvre pas.
  }, [pret, enMarche, voulue, props.lab, props.machine])

  useEffect(() => {
    if (!props.visible || !pret) return
    ajusteur.current?.fit()
    terminal.current?.focus()
  }, [props.visible, pret])

  return (
    <div className="console" hidden={!props.visible}>
      <div className="console-xterm" ref={boite}></div>
      {!enMarche && (
        <div className="console-etat">
          <p>{props.machine} est éteinte.</p>
          <button
            className="oc-btn"
            disabled={props.occupe}
            onClick={() => {
              setVoulue(true)
              props.onDemarrer()
            }}
          >
            Démarrer {props.machine}
          </button>
        </div>
      )}
      {enMarche && etat === 'finie' && (
        <div className="console-etat console-etat--bas">
          <p>Session terminée.</p>
          <button className="oc-btn" onClick={() => setVoulue(true)}>
            Rouvrir
          </button>
        </div>
      )}
    </div>
  )
}
