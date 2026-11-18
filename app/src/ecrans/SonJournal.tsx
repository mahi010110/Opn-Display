import { useEffect, useState } from 'react'

import { journalIa } from '../pont'
import type { Accord, EntreeJournal } from '../types'

const MOT_ACCORD: Record<Accord, string> = {
  donne: 'autorisé',
  refuse: 'refusé',
  sansDemander: 'sans demander',
}

const HEURE = new Intl.DateTimeFormat('fr-FR', { hour: '2-digit', minute: '2-digit' })
const JOUR = new Intl.DateTimeFormat('fr-FR', { day: 'numeric', month: 'short' })

/** « 14:02 » aujourd'hui, « hier 14:02 », sinon « 12 nov. 14:02 ». */
function quand(secondes: number): string {
  const date = new Date(secondes * 1000)
  const maintenant = new Date()
  const minuit = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()
  const jours = Math.round((minuit(maintenant) - minuit(date)) / 86_400_000)
  const heure = HEURE.format(date)
  if (jours <= 0) return heure
  if (jours === 1) return `hier ${heure}`
  return `${JOUR.format(date)} ${heure}`
}

/** Réglages, Son journal : ce que l'IA a fait, le plus récent en haut. */
export function SonJournal() {
  const [entrees, setEntrees] = useState<EntreeJournal[] | null>(null)
  const [erreur, setErreur] = useState<string | null>(null)

  useEffect(() => {
    journalIa()
      .then(setEntrees)
      .catch((e) => setErreur(String(e)))
  }, [])

  return (
    <section className="oc-window">
      <header className="oc-window__bar oc-window__bar--accent">
        <h2 className="oc-window__title">Son journal</h2>
      </header>
      <div className="oc-window__body oc-window__body--flush journal">
        {erreur && <p className="note erreur">{erreur}</p>}
        {entrees?.length === 0 && (
          <p className="ecran-vide">Rien encore. Ce que l'IA fait dans tes labs s'écrit ici.</p>
        )}
        {entrees && entrees.length > 0 && (
          <table className="table">
            <thead>
              <tr>
                <th>Heure</th>
                <th>Où</th>
                <th>Ce qu'elle a fait</th>
                <th>Accord</th>
              </tr>
            </thead>
            <tbody>
              {entrees.map((entree, i) => (
                <tr key={i}>
                  <td className="chiffre">{quand(entree.quand)}</td>
                  <td className="chiffre">{entree.lieu}</td>
                  <td>
                    {entree.quoi}
                    {entree.commande && (
                      <>
                        {' : '}
                        <code>{entree.commande}</code>
                      </>
                    )}
                  </td>
                  <td className={entree.accord === 'refuse' ? 'chiffre refus' : 'chiffre'}>
                    {entree.accord ? MOT_ACCORD[entree.accord] : ''}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </section>
  )
}
