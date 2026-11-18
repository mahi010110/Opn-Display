import { Channel, invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import type {
  Branchement,
  Comportements,
  Demande,
  Difficulte,
  EntreeJournal,
  Etape,
  EtatIa,
  EtatLab,
  EvenementIa,
  Facon,
  Lab,
  ListeMemoire,
  Memoire,
  Niveau,
  PropositionLab,
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

/** Tous les labs et leurs machines, pour Mes labs. */
export function etatLabs(): Promise<EtatLab[]> {
  return invoke<EtatLab[]>('etat_labs')
}

/** Ouvre un lab à l'écran : l'IA et les pannes travaillent dans celui-là. */
export function ouvrirLab(id: string): Promise<EtatLab> {
  return invoke<EtatLab>('ouvrir_lab', { id })
}

export function etatLab(id: string): Promise<EtatLab> {
  return invoke<EtatLab>('etat_lab', { id })
}

export function lancerLab(id: string): Promise<EtatLab> {
  return invoke<EtatLab>('lancer_lab', { id })
}

export function arreterLab(id: string): Promise<EtatLab> {
  return invoke<EtatLab>('arreter_lab', { id })
}

export function demarrerMachine(id: string, nom: string): Promise<EtatLab> {
  return invoke<EtatLab>('demarrer_machine', { id, nom })
}

export function arreterMachine(id: string, nom: string): Promise<EtatLab> {
  return invoke<EtatLab>('arreter_machine', { id, nom })
}

export function remettreAZero(id: string): Promise<EtatLab> {
  return invoke<EtatLab>('remettre_a_zero', { id })
}

/** Le lab proposé, sans une machine. Rien n'est créé. */
export function retirerDuLab(lab: Lab, machine: string): Promise<Lab> {
  return invoke<Lab>('retirer_du_lab', { lab, machine })
}

/** Range le lab proposé parmi les labs. Ses machines se créent à son lancement. */
export function creerLab(lab: Lab): Promise<EtatLab> {
  return invoke<EtatLab>('creer_lab', { lab })
}

export function supprimerLab(id: string): Promise<void> {
  return invoke<void>('supprimer_lab', { id })
}

/** Le lab que propose l'IA, ou rien si l'utilisateur l'a arrêtée. `actuel`
 * est le lab proposé tel qu'il est à l'écran : l'IA repart de lui. */
export function proposerLab(description: string, actuel: Lab | null): Promise<PropositionLab | null> {
  return invoke<PropositionLab | null>('proposer_lab', { description, actuel })
}

export function oublierProposition(): Promise<void> {
  return invoke<void>('oublier_proposition')
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
  lab: string,
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
  const id = await invoke<number>('ouvrir_terminal', { lab, machine, colonnes, lignes, flux })
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

export function reglerIa(facon: Facon, niveau: Niveau, unParUn: Comportements): Promise<EtatIa> {
  return invoke<EtatIa>('regler_ia', { facon, niveau, unParUn })
}

export function memoireIa(): Promise<Memoire> {
  return invoke<Memoire>('memoire_ia')
}

export function activerMemoire(active: boolean): Promise<Memoire> {
  return invoke<Memoire>('activer_memoire', { active })
}

/** Une ligne corrigée. Vide, elle part. */
export function corrigerMemoire(liste: ListeMemoire, rang: number, texte: string): Promise<Memoire> {
  return invoke<Memoire>('corriger_memoire', { liste, rang, texte })
}

export function retirerDeLaMemoire(liste: ListeMemoire, rang: number): Promise<Memoire> {
  return invoke<Memoire>('retirer_de_la_memoire', { liste, rang })
}

export function effacerMemoire(): Promise<Memoire> {
  return invoke<Memoire>('effacer_memoire')
}

/** Le journal de l'IA, du plus récent au plus ancien. */
export function journalIa(): Promise<EntreeJournal[]> {
  return invoke<EntreeJournal[]>('journal_ia')
}

export function parlerIa(texte: string): Promise<void> {
  return invoke<void>('parler_ia', { texte })
}

/** L'utilisateur bloque sur une panne : l'IA propose son aide, si elle en a le droit. */
export function aiderIa(): Promise<void> {
  return invoke<void>('aider_ia')
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
