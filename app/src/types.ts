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

export interface MachineLab {
  nom: string
  sorte: Sorte
  x: number
  y: number
  interfaces: Interface[]
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
}
