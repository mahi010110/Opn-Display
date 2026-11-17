import { Channel, invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import type {
  Branchement,
  Demande,
  Difficulte,
  Etape,
  EtatIa,
  EtatLab,
  EvenementIa,
  Liberte,
  Niveau,
  Rapport,
  ResumePanne,
  VuePanne,
} from './types'

export function verifierPc(): Promise<Rapport> {
  return invoke<Rapport>('verifier_pc')
}

export function installer(memoireGo: number | null): Promise<Rapport> {
  return invoke<Rapport>('installer', { memoireGo })
}

export function arreterInstallation(): Promise<void> {
  return invoke<void>('arreter_installation')
}

export function retirerLinux(): Promise<Rapport> {
  return invoke<Rapport>('retirer_linux')
}

export function installerWsl(): Promise<void> {
  return invoke<void>('installer_wsl')
}

export function suivreInstallation(quandEtape: (etape: Etape) => void): Promise<UnlistenFn> {
  return listen<Etape>('installation', (evenement) => quandEtape(evenement.payload))
}

export function etatLab(): Promise<EtatLab> {
  return invoke<EtatLab>('etat_lab')
}

export function lancerLab(): Promise<EtatLab> {
  return invoke<EtatLab>('lancer_lab')
}

export function arreterLab(): Promise<EtatLab> {
  return invoke<EtatLab>('arreter_lab')
}

export function demarrerMachine(nom: string): Promise<EtatLab> {
  return invoke<EtatLab>('demarrer_machine', { nom })
}

export function arreterMachine(nom: string): Promise<EtatLab> {
  return invoke<EtatLab>('arreter_machine', { nom })
}

export function remettreAZero(): Promise<EtatLab> {
  return invoke<EtatLab>('remettre_a_zero')
}

/** Ce que le moteur raconte pendant qu'il crée ou démarre des machines. */
export function suivreLab(quandAvance: (texte: string) => void): Promise<UnlistenFn> {
  return listen<string>('lab-avance', (evenement) => quandAvance(evenement.payload))
}

/** Une machine a changé d'état, par exemple éteinte depuis son terminal. */
export function suivreMachines(quandChange: () => void): Promise<UnlistenFn> {
  return listen('lab', () => quandChange())
}

export function veiller(): Promise<void> {
  return invoke<void>('veiller')
}

export interface Session {
  id: number
  ecrire: (texte: string) => void
  redimensionner: (colonnes: number, lignes: number) => void
  fermer: () => void
}

/**
 * Ouvre un shell dans une machine. Les octets arrivent par `recevoir`, dans
 * l'ordre ; `fin` est appelée quand le shell s'est arrêté.
 */
export async function ouvrirTerminal(
  machine: string,
  colonnes: number,
  lignes: number,
  recevoir: (octets: Uint8Array) => void,
  fin: () => void,
): Promise<Session> {
  const flux = new Channel<ArrayBuffer | number[] | string>()
  flux.onmessage = (message) => {
    if (message === 'fin') fin()
    else if (message instanceof ArrayBuffer) recevoir(new Uint8Array(message))
    else if (Array.isArray(message)) recevoir(Uint8Array.from(message))
  }
  const id = await invoke<number>('ouvrir_terminal', { machine, colonnes, lignes, flux })
  return {
    id,
    ecrire: (donnees) => void invoke('ecrire_terminal', { id, donnees }).catch(() => undefined),
    redimensionner: (c, l) =>
      void invoke('redimensionner_terminal', { id, colonnes: c, lignes: l }).catch(() => undefined),
    fermer: () => void invoke('fermer_terminal', { id }).catch(() => undefined),
  }
}

export function etatIa(): Promise<EtatIa> {
  return invoke<EtatIa>('etat_ia')
}

export function brancherIa(branchement: Branchement, cle: string | null): Promise<EtatIa> {
  return invoke<EtatIa>('brancher_ia', { branchement, cle })
}

export function essayerIa(): Promise<string> {
  return invoke<string>('essayer_ia')
}

export function modelesIa(branchement: Branchement, cle: string | null): Promise<string[]> {
  return invoke<string[]>('modeles_ia', { branchement, cle })
}

export function reglerIa(niveau: Niveau, dansLeLab: Liberte): Promise<EtatIa> {
  return invoke<EtatIa>('regler_ia', { niveau, dansLeLab })
}

export function parlerIa(texte: string): Promise<void> {
  return invoke<void>('parler_ia', { texte })
}

export function repondreAccord(id: string, oui: boolean): Promise<void> {
  return invoke<void>('repondre_accord', { id, oui })
}

export function arreterIa(): Promise<void> {
  return invoke<void>('arreter_ia')
}

export function oublierConversation(): Promise<void> {
  return invoke<void>('oublier_conversation')
}

export function suivreIa(
  quandEvenement: (evenement: EvenementIa) => void,
  quandAccord: (demande: Demande) => void,
): Promise<UnlistenFn> {
  return Promise.all([
    listen<EvenementIa>('ia', (e) => quandEvenement(e.payload)),
    listen<Demande>('ia-accord', (e) => quandAccord(e.payload)),
  ]).then((arrets) => () => arrets.forEach((arreter) => arreter()))
}

export function cataloguePannes(): Promise<ResumePanne[]> {
  return invoke<ResumePanne[]>('catalogue_pannes')
}

export function poserPanne(id: string): Promise<VuePanne> {
  return invoke<VuePanne>('poser_panne', { id })
}

/** La panne inventée, ou rien si l'utilisateur a arrêté l'IA. */
export function inventerPanne(niveau: Difficulte): Promise<VuePanne | null> {
  return invoke<VuePanne | null>('inventer_panne', { niveau })
}

export function panneEnCours(): Promise<VuePanne | null> {
  return invoke<VuePanne | null>('panne_en_cours')
}

export function indicePanne(): Promise<string> {
  return invoke<string>('indice_panne')
}

export function verifierPanne(): Promise<VuePanne> {
  return invoke<VuePanne>('verifier_panne')
}

export function abandonnerPanne(): Promise<void> {
  return invoke<void>('abandonner_panne')
}

export function fermerPanne(): Promise<void> {
  return invoke<void>('fermer_panne')
}
