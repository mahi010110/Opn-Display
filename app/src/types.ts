// Les mêmes formes que dans le moteur (moteur/src/rapport.rs,
// moteur/src/installation.rs et moteur/src/lab.rs), telles que serde les
// envoie.

export type Etat = 'ok' | 'manque' | 'inconnu'

export interface Verification {
  nom: string
  etat: Etat
  detail: string
}

export type Blocage =
  | 'virtualisationCoupee'
  | 'sansVirtualisation'
  | 'wslAbsent'
  | 'incusAbsent'
  | 'incusMuet'
  | 'systemeNonPrisEnCharge'

export type LinuxApp = 'aInstaller' | 'installe' | 'systeme' | 'aucun'

export type Machines = 'aInstaller' | 'muettes' | 'aPreparer' | 'pretes'

export type Fabricant = 'intel' | 'amd' | 'autre'

export interface Rapport {
  plateforme: 'windows' | 'linux' | 'autre'
  systeme: string
  verifications: Verification[]
  blocage: Blocage | null
  processeur: { nom: string; fabricant: Fabricant }
  memoire: {
    totaleGo: number
    libreGo: number
    wslGo: number | null
    propositionGo: number | null
  }
  linuxApp: LinuxApp
  machines: Machines
  noyau: string | null
  kvm: boolean | null
  wifiSimule: boolean | null
}

export type EtatEtape = 'enCours' | 'fait' | 'saute' | 'erreur'

export type Pas = 'telecharger' | 'importer' | 'memoire' | 'demarrer' | 'incus' | 'image'

export interface Etape {
  pas: Pas
  etat: EtatEtape
  detail: string
  avance: number | null
}

export type Sorte = 'poste' | 'serveur' | 'routeur' | 'switch' | 'parFeu' | 'stockage'

export interface Interface {
  nom: string
  adresse: string | null
  passerelle: string | null
}

export interface Route {
  destination: string
  passerelle: string
}

export interface MachineLab {
  nom: string
  sorte: Sorte
  x: number
  y: number
  interfaces: Interface[]
  routes: Route[]
  transfere: boolean
  /** La mémoire qu'elle prendra, en octets, montrée avant le lancement. */
  memoirePrevue: number
}

export interface Bout {
  machine: string
  interface: string
}

export interface Cable {
  de: Bout
  vers: Bout
  reseau: string | null
}

export interface Lab {
  id: string
  nom: string
  machines: MachineLab[]
  cables: Cable[]
  numero: number
}

/** Le lab que propose l'IA. Sans lab, elle a posé une question. */
export interface PropositionLab {
  lab: Lab | null
  commentaire: string
}

export type Marche = 'absente' | 'arretee' | 'enMarche'

export interface EtatMachine {
  nom: string
  marche: Marche
  memoire: number | null
}

export interface EtatLab {
  lab: Lab
  machines: EtatMachine[]
  /** Sa dernière ouverture, en secondes depuis 1970. */
  ouvert: number | null
}

export type Branchement =
  | { sorte: 'openAi'; adresse: string; modele: string }
  | { sorte: 'anthropic'; modele: string }
  | { sorte: 'ollama'; adresse: string; modele: string }
  | { sorte: 'chatGpt'; modele: string }

export type Niveau = 'prof' | 'coach' | 'silencieuse'

export type Liberte = 'libre' | 'avecAccord'

/** Comment on règle l'IA, dans Régler l'IA. */
export type Facon = 'parNiveau' | 'unParUn' | 'sAdapte'

/** Ce qu'elle fait quand on lui demande la solution d'une panne. */
export type Solution = 'donnee' | 'indices' | 'rien'

/** Quand elle parle sans qu'on lui écrive. */
export type Initiative = 'quandTuBloques' | 'jamais'

export interface Comportements {
  dansLeLab: Liberte
  solution: Solution
  initiative: Initiative
}

export interface Adaptation {
  niveau: Niveau
  serie: number
  raison: string
}

export interface ReglagesIa {
  branchement: Branchement | null
  facon: Facon
  niveau: Niveau
  unParUn: Comportements
  adaptation: Adaptation
  essai: number | null
}

export interface EtatIa {
  reglages: ReglagesIa
  /** Ce qu'elle fait en ce moment, quelle que soit la façon de régler. */
  comportements: Comportements
  /** Le niveau qui décide en ce moment, sauf réglée un par un. */
  niveauDuMoment: Niveau | null
  cleRangee: boolean
  codexPresent: boolean
  ollama: string[] | null
}

export type ListeMemoire = 'toi' | 'notes'

export interface Memoire {
  active: boolean
  toi: string[]
  notes: string[]
}

export type Accord = 'donne' | 'refuse' | 'sansDemander'

export interface EntreeJournal {
  /** En secondes depuis 1970. */
  quand: number
  lieu: string
  quoi: string
  commande: string | null
  accord: Accord | null
}

export type EvenementIa =
  | { sorte: 'texte'; texte: string }
  | { sorte: 'outil'; id: string; titre: string; detail: string }
  | { sorte: 'resultat'; id: string; ok: boolean; apercu: string }
  | { sorte: 'refus'; id: string }

export interface Demande {
  id: string
  lieu: string
  titre: string
  detail: string
}

export type Difficulte = 'facile' | 'moyen' | 'difficile'

export interface ResumePanne {
  id: string
  titre: string
  famille: string
  niveau: Difficulte
}

export interface ResultatTest {
  nom: string
  machine: string
  commande: string
  reussi: boolean
  sortie: string
}

export interface VerificationPanne {
  reparee: boolean
  tests: ResultatTest[]
  explication: string | null
}

export interface VuePanne {
  /** Le lab où elle est posée, et son nom. */
  lab: string
  nomDuLab: string
  titre: string
  niveau: Difficulte
  inventee: boolean
  posee: number
  tests: number
  indices: number
  indicesDonnes: number
  /** Les indices déjà donnés, pour les remontrer quand l'app rouvre. */
  indicesLus: string[]
  verification: VerificationPanne | null
}
