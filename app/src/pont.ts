import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import type { Etape, Rapport } from './types'

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
