import { useCallback, useEffect, useState, type FormEvent, type KeyboardEvent } from 'react'

import { activerMemoire, corrigerMemoire, effacerMemoire, memoireIa, retirerDeLaMemoire } from '../pont'
import type { ListeMemoire, Memoire } from '../types'
import { Dialogue } from '../ui/Dialogue'

/** Comme LIGNES_MAX et LIGNE_MAX dans moteur/src/ia/memoire.rs. */
const LIGNES_MAX = 30
const LIGNE_MAX = 200

interface Edition {
  liste: ListeMemoire
  rang: number
  texte: string
}

/** Réglages, Sa mémoire : ce que l'IA retient d'une session à l'autre. */
export function SaMemoire() {
  const [memoire, setMemoire] = useState<Memoire | null>(null)
  const [edition, setEdition] = useState<Edition | null>(null)
  const [question, setQuestion] = useState(false)
  const [erreur, setErreur] = useState<string | null>(null)
  const fermer = useCallback(() => setQuestion(false), [])

  useEffect(() => {
    memoireIa()
      .then(setMemoire)
      .catch((e) => setErreur(String(e)))
  }, [])

  const garder = (changement: Promise<Memoire>) => {
    setErreur(null)
    changement.then(setMemoire).catch((e) => setErreur(String(e)))
  }

  const enregistrer = (evenement: FormEvent) => {
    evenement.preventDefault()
    if (!edition) return
    garder(corrigerMemoire(edition.liste, edition.rang, edition.texte))
    setEdition(null)
  }

  const touche = (evenement: KeyboardEvent<HTMLInputElement>) => {
    if (evenement.key === 'Escape') setEdition(null)
  }

  const lignes = (liste: ListeMemoire, textes: string[], vide: string) =>
    textes.length === 0 ? (
      <p className="note">{vide}</p>
    ) : (
      <ul className="lignes">
        {textes.map((texte, rang) =>
          edition?.liste === liste && edition.rang === rang ? (
            <li key={rang}>
              <form className="ligne-edition" onSubmit={enregistrer}>
                <input
                  className="oc-input"
                  autoFocus
                  aria-label="Corriger la ligne"
                  maxLength={LIGNE_MAX}
                  value={edition.texte}
                  onChange={(e) => setEdition({ ...edition, texte: e.target.value })}
                  onKeyDown={touche}
                />
                <button className="oc-btn oc-btn--sm">Enregistrer</button>
              </form>
            </li>
          ) : (
            <li key={rang}>
              <span>{texte}</span>
              <button
                className="oc-iconbtn oc-iconbtn--ghost oc-iconbtn--sm"
                aria-label="Corriger"
                title="Corriger"
                onClick={() => setEdition({ liste, rang, texte })}
              >
                <i className="ic i-crayon" aria-hidden="true"></i>
              </button>
              <button
                className="oc-iconbtn oc-iconbtn--ghost oc-iconbtn--sm"
                aria-label="Retirer"
                title="Retirer"
                onClick={() => garder(retirerDeLaMemoire(liste, rang))}
              >
                <i className="ic i-fermer" aria-hidden="true"></i>
              </button>
            </li>
          ),
        )}
      </ul>
    )

  const nombre = memoire ? memoire.toi.length + memoire.notes.length : 0

  return (
    <section className="oc-window">
      <header className="oc-window__bar oc-window__bar--accent">
        <h2 className="oc-window__title">Sa mémoire</h2>
      </header>
      <div className="oc-window__body bloc">
        {memoire && (
          <>
            <label className="oc-switch">
              <input
                type="checkbox"
                checked={memoire.active}
                onChange={(e) => garder(activerMemoire(e.target.checked))}
              />
              Elle se souvient d'une session à l'autre
            </label>
            <fieldset className="oc-fieldset">
              <legend>Ce qu'elle sait de toi</legend>
              {lignes('toi', memoire.toi, 'Rien encore. Dis-lui ce que tu prépares : elle le retiendra.')}
            </fieldset>
            <fieldset className="oc-fieldset">
              <legend>Ses notes</legend>
              {lignes('notes', memoire.notes, 'Rien encore.')}
            </fieldset>
          </>
        )}
        {erreur && <p className="note erreur">{erreur}</p>}
        {memoire && (
          <div className="a-part">
            <span className="mono">
              {nombre} {nombre > 1 ? 'lignes' : 'ligne'} sur {LIGNES_MAX}
            </span>
            <button className="oc-btn oc-btn--danger" disabled={nombre === 0} onClick={() => setQuestion(true)}>
              <i className="ic i-fermer" aria-hidden="true"></i>Effacer sa mémoire
            </button>
          </div>
        )}
      </div>

      {question && (
        <Dialogue
          titre="Effacer sa mémoire"
          onFermer={fermer}
          pied={
            <button
              className="oc-btn oc-btn--danger"
              onClick={() => {
                garder(effacerMemoire())
                setQuestion(false)
              }}
            >
              Effacer
            </button>
          }
        >
          <p className="lu">Elle oublie ce qu'elle sait de toi et ses notes. Ça ne se rattrape pas.</p>
        </Dialogue>
      )}
    </section>
  )
}
