// Les mêmes formes que dans le moteur (moteur/src/rapport.rs et
// moteur/src/installation.rs), telles que serde les envoie.

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
  | 'systemeNonPrisEnCharge'

export type LinuxApp = 'aInstaller' | 'installe' | 'systeme' | 'aucun'

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
  noyau: string | null
  kvm: boolean | null
  wifiSimule: boolean | null
}

export type EtatEtape = 'enCours' | 'fait' | 'saute' | 'erreur'

export interface Etape {
  numero: number
  etat: EtatEtape
  detail: string
  avance: number | null
}
