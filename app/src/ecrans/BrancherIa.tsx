import { useState } from 'react'

import { brancherIa, essayerIa, etatIa as lireEtatIa, modelesIa } from '../pont'
import type { Branchement, EtatIa } from '../types'

type Choix = 'cle' | 'local' | 'chatgpt'

/** Les fournisseurs à clé API. Ceux au format d'OpenAI ont leur adresse. */
const FOURNISSEURS = [
  { id: 'openai', nom: 'OpenAI', adresse: 'https://api.openai.com/v1' },
  { id: 'anthropic', nom: 'Anthropic', adresse: '' },
  { id: 'mistral', nom: 'Mistral', adresse: 'https://api.mistral.ai/v1' },
  { id: 'openrouter', nom: 'OpenRouter', adresse: 'https://openrouter.ai/api/v1' },
  { id: 'autre', nom: 'Autre, au format OpenAI', adresse: '' },
]

const OLLAMA = 'http://127.0.0.1:11434/v1'

function choixDe(branchement: Branchement | null): Choix {
  if (branchement?.sorte === 'ollama') return 'local'
  if (branchement?.sorte === 'chatGpt') return 'chatgpt'
  return 'cle'
}

function fournisseurDe(branchement: Branchement | null): string {
  if (branchement?.sorte === 'anthropic') return 'anthropic'
  if (branchement?.sorte === 'openAi') {
    return FOURNISSEURS.find((f) => f.adresse === branchement.adresse)?.id ?? 'autre'
  }
  return 'openai'
}

function heure(secondes: number): string {
  return new Date(secondes * 1000).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })
}

type Retour = { ok: boolean; texte: string } | null

/** Réglages, Brancher l'IA : une clé API, un modèle local, ou l'abonnement ChatGPT. */
export function BrancherIa(props: { etatIa: EtatIa; onEtatIa: (etat: EtatIa) => void; sousWindows: boolean }) {
  const { etatIa } = props
  const actuel = etatIa.reglages.branchement
  const [choix, setChoix] = useState<Choix>(choixDe(actuel))
  const [fournisseur, setFournisseur] = useState(fournisseurDe(actuel))
  const [adresse, setAdresse] = useState(actuel?.sorte === 'openAi' ? actuel.adresse : '')
  const [modele, setModele] = useState(actuel ? actuel.modele : '')
  const [cle, setCle] = useState('')
  const [modeles, setModeles] = useState<string[]>([])
  const [occupe, setOccupe] = useState(false)
  const [retour, setRetour] = useState<Retour>(null)

  const branchement = (): Branchement => {
    if (choix === 'local') return { sorte: 'ollama', adresse: OLLAMA, modele }
    if (choix === 'chatgpt') return { sorte: 'chatGpt', modele }
    if (fournisseur === 'anthropic') return { sorte: 'anthropic', modele }
    const preset = FOURNISSEURS.find((f) => f.id === fournisseur)
    return { sorte: 'openAi', adresse: preset?.adresse || adresse, modele }
  }

  const choisir = (nouveau: Choix) => {
    setChoix(nouveau)
    setRetour(null)
    setModeles([])
    setModele(nouveau === choixDe(actuel) && actuel ? actuel.modele : '')
  }

  const voirModeles = () => {
    setRetour(null)
    modelesIa(branchement(), cle || null)
      .then((liste) => {
        setModeles(liste)
        if (liste.length === 0) setRetour({ ok: false, texte: 'Aucun modèle trouvé.' })
      })
      .catch((e) => setRetour({ ok: false, texte: String(e) }))
  }

  const essayer = () => {
    setOccupe(true)
    setRetour(null)
    brancherIa(branchement(), cle || null)
      .then((etat) => {
        props.onEtatIa(etat)
        setCle('')
        return essayerIa()
      })
      .then((reponse) => {
        setRetour({ ok: true, texte: `Elle répond : « ${reponse} »` })
        return lireEtatIa().then(props.onEtatIa)
      })
      .catch((e) => setRetour({ ok: false, texte: String(e) }))
      .finally(() => setOccupe(false))
  }

  const branche = (c: Choix) => actuel !== null && choixDe(actuel) === c
  const etatLigne = (c: Choix): string => {
    if (branche(c)) return etatIa.reglages.essai ? `branchée, essayée à ${heure(etatIa.reglages.essai)}` : 'branchée'
    if (c === 'local') {
      if (etatIa.ollama === null) return 'Ollama ne répond pas'
      const nombre = etatIa.ollama.length
      return `Ollama, ${nombre} ${nombre > 1 ? 'modèles' : 'modèle'}`
    }
    if (c === 'chatgpt') return etatIa.codexPresent ? 'Codex est installé' : "Codex n'est pas installé"
    return ''
  }

  const sansModele = modele.trim() === '' && choix !== 'chatgpt'
  const sansCle = choix === 'cle' && cle.trim() === '' && !(branche('cle') && etatIa.cleRangee)

  return (
    <section className="oc-window">
      <header className="oc-window__bar oc-window__bar--accent">
        <h2 className="oc-window__title">Brancher l'IA</h2>
      </header>
      <div className="oc-window__body bloc">
        <div className="choix" role="radiogroup" aria-label="Brancher l'IA">
          <label className="choix-ligne">
            <span className="oc-radio">
              <input type="radio" name="branchement" checked={choix === 'cle'} onChange={() => choisir('cle')} />
            </span>
            Une clé API<span className="mono">{etatLigne('cle')}</span>
          </label>
          {choix === 'cle' && (
            <div className="choix-detail">
              <div className="grille-2">
                <label className="oc-field">
                  <span className="oc-field__label">Fournisseur</span>
                  <select
                    className="oc-select"
                    value={fournisseur}
                    onChange={(e) => {
                      setFournisseur(e.target.value)
                      setModeles([])
                    }}
                  >
                    {FOURNISSEURS.map((f) => (
                      <option key={f.id} value={f.id}>
                        {f.nom}
                      </option>
                    ))}
                  </select>
                </label>
                <Modele modele={modele} modeles={modeles} onModele={setModele} onVoir={voirModeles} />
              </div>
              {fournisseur === 'autre' && (
                <label className="oc-field">
                  <span className="oc-field__label">Adresse</span>
                  <input
                    className="oc-input oc-input--mono"
                    value={adresse}
                    placeholder="https://…/v1"
                    onChange={(e) => setAdresse(e.target.value)}
                  />
                </label>
              )}
              <label className="oc-field">
                <span className="oc-field__label">Clé</span>
                <input
                  className="oc-input oc-input--mono"
                  type="password"
                  value={cle}
                  placeholder={branche('cle') && etatIa.cleRangee ? 'rangée, tape une autre clé pour la changer' : ''}
                  onChange={(e) => setCle(e.target.value)}
                  aria-label="Clé"
                />
              </label>
              <p className="note">
                La clé est rangée dans le trousseau {props.sousWindows ? 'de Windows' : 'du système'}. L'IA ne la voit
                jamais.
              </p>
            </div>
          )}

          <label className="choix-ligne">
            <span className="oc-radio">
              <input type="radio" name="branchement" checked={choix === 'local'} onChange={() => choisir('local')} />
            </span>
            Un modèle local<span className="mono">{etatLigne('local')}</span>
          </label>
          {choix === 'local' && (
            <div className="choix-detail">
              {etatIa.ollama === null ? (
                <p className="note">
                  Installe Ollama sur ce PC, puis un modèle qui sait utiliser des outils, par exemple qwen3.
                </p>
              ) : (
                <label className="oc-field">
                  <span className="oc-field__label">Modèle</span>
                  <select className="oc-select" value={modele} onChange={(e) => setModele(e.target.value)}>
                    <option value="">Choisis un modèle</option>
                    {etatIa.ollama.map((m) => (
                      <option key={m} value={m}>
                        {m}
                      </option>
                    ))}
                  </select>
                </label>
              )}
            </div>
          )}

          <label className="choix-ligne">
            <span className="oc-radio">
              <input
                type="radio"
                name="branchement"
                checked={choix === 'chatgpt'}
                onChange={() => choisir('chatgpt')}
              />
            </span>
            L'abonnement ChatGPT<span className="mono">{etatLigne('chatgpt')}</span>
          </label>
          {choix === 'chatgpt' && (
            <div className="choix-detail">
              {etatIa.codexPresent ? (
                <label className="oc-field">
                  <span className="oc-field__label">Modèle, si tu veux en choisir un</span>
                  <input className="oc-input oc-input--mono" value={modele} onChange={(e) => setModele(e.target.value)} />
                </label>
              ) : (
                <p className="note">Installe Codex, l'outil officiel d'OpenAI, puis connecte-toi avec ton compte ChatGPT.</p>
              )}
            </div>
          )}
        </div>

        <p className="note">Un modèle local est gratuit, et il prend de la RAM à tes labs.</p>
        {retour && <p className={'note' + (retour.ok ? ' reussi' : ' erreur')}>{retour.texte}</p>}
        <div className="pied">
          <button
            className="oc-btn oc-btn--primary"
            disabled={
              occupe ||
              sansModele ||
              sansCle ||
              (choix === 'local' && etatIa.ollama === null) ||
              (choix === 'chatgpt' && !etatIa.codexPresent)
            }
            onClick={essayer}
          >
            {occupe ? 'Essai en cours' : 'Brancher et essayer'}
          </button>
        </div>
      </div>
    </section>
  )
}

function Modele(props: { modele: string; modeles: string[]; onModele: (m: string) => void; onVoir: () => void }) {
  return (
    <label className="oc-field">
      <span className="oc-field__label">Modèle</span>
      <span className="rang">
        <input
          className="oc-input oc-input--mono"
          list="modeles-ia"
          value={props.modele}
          onChange={(e) => props.onModele(e.target.value)}
        />
        <button type="button" className="oc-btn oc-btn--sm" onClick={props.onVoir}>
          Voir la liste
        </button>
      </span>
      <datalist id="modeles-ia">
        {props.modeles.map((m) => (
          <option key={m} value={m} />
        ))}
      </datalist>
    </label>
  )
}
