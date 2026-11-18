import type { EtatIa, Niveau } from '../types'

export const NIVEAUX: { niveau: Niveau; nom: string }[] = [
  { niveau: 'prof', nom: 'Prof' },
  { niveau: 'coach', nom: 'Coach' },
  { niveau: 'silencieuse', nom: 'Silencieuse' },
]

export const NOM_NIVEAU: Record<Niveau, string> = { prof: 'Prof', coach: 'Coach', silencieuse: 'Silencieuse' }

/** La façon de régler, en quelques mots, quand ce n'est pas un niveau choisi. */
export function faconIa(etat: EtatIa): string | null {
  switch (etat.reglages.facon) {
    case 'parNiveau':
      return null
    case 'unParUn':
      return 'Réglée un par un'
    case 'sAdapte':
      return `Elle s'adapte : ${NOM_NIVEAU[etat.niveauDuMoment ?? 'coach']}`
  }
}

/** Pour la barre d'état du lab : « IA : Coach, avec accord dans le lab ». */
export function resumeIa(etat: EtatIa): string {
  const facon =
    etat.reglages.facon === 'unParUn'
      ? 'réglée un par un'
      : etat.reglages.facon === 'sAdapte'
        ? `s'adapte, en ${NOM_NIVEAU[etat.niveauDuMoment ?? 'coach']}`
        : NOM_NIVEAU[etat.reglages.niveau]
  const lab = etat.comportements.dansLeLab === 'avecAccord' ? 'avec accord dans le lab' : 'libre dans le lab'
  return `IA : ${facon}, ${lab}`
}
