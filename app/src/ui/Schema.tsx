import type { EtatMachine, Lab, MachineLab, Sorte } from '../types'

const ICONE: Record<Sorte, string> = {
  poste: 'i-poste',
  serveur: 'i-serveur',
  routeur: 'i-routeur',
  switch: 'i-switch',
  parFeu: 'i-parefeu',
  stockage: 'i-stockage',
}

/** Où se pose le nom d'une interface, sur le câble, près de sa machine. */
const PRES_DE_LA_MACHINE = 0.2

function adresses(machine: MachineLab): string {
  return machine.interfaces
    .flatMap((i) => (i.adresse ? [i.adresse.split('/')[0]] : []))
    .join(' · ')
}

/** Le schéma du lab : les machines, leurs câbles, et le réseau de chacun. */
export function Schema(props: {
  lab: Lab
  machines: EtatMachine[]
  choisie: string | null
  onChoisir: (machine: string) => void
}) {
  const { lab } = props
  const place = (nom: string) => lab.machines.find((m) => m.nom === nom)
  const traits = lab.cables.flatMap((cable) => {
    const de = place(cable.de.machine)
    const vers = place(cable.vers.machine)
    return de && vers ? [{ cable, de, vers }] : []
  })

  return (
    <div className="plan">
      <svg className="cables" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
        {traits.map(({ de, vers }, i) => (
          <polyline key={i} points={`${de.x},${de.y} ${vers.x},${vers.y}`} />
        ))}
      </svg>
      {traits.map(({ cable, de, vers }, i) => (
        <span key={`reseau-${i}`}>
          {cable.reseau && (
            <span className="reseau" style={{ left: `${(de.x + vers.x) / 2}%`, top: `${(de.y + vers.y) / 2}%` }}>
              {cable.reseau}
            </span>
          )}
          <span
            className="port"
            style={{
              left: `${de.x + (vers.x - de.x) * PRES_DE_LA_MACHINE}%`,
              top: `${de.y + (vers.y - de.y) * PRES_DE_LA_MACHINE}%`,
            }}
          >
            {cable.de.interface}
          </span>
          <span
            className="port"
            style={{
              left: `${vers.x + (de.x - vers.x) * PRES_DE_LA_MACHINE}%`,
              top: `${vers.y + (de.y - vers.y) * PRES_DE_LA_MACHINE}%`,
            }}
          >
            {cable.vers.interface}
          </span>
        </span>
      ))}
      {lab.machines.map((machine) => {
        const marche = props.machines.find((m) => m.nom === machine.nom)?.marche ?? 'absente'
        const allumee = marche === 'enMarche'
        const classes = [
          'machine',
          allumee ? '' : 'machine--eteinte',
          props.choisie === machine.nom ? 'machine--choisie' : '',
        ].join(' ')
        return (
          <button
            key={machine.nom}
            className={classes.trim()}
            style={{ left: `${machine.x}%`, top: `${machine.y}%` }}
            onClick={() => props.onChoisir(machine.nom)}
            aria-label={`${machine.nom}, ${allumee ? 'en marche' : 'éteinte'}. Ouvrir son terminal`}
          >
            <span className="machine-case">
              <i className={'ic ic-24 ' + ICONE[machine.sorte]} aria-hidden="true"></i>
            </span>
            <span className="machine-nom">{machine.nom}</span>
            <span className="machine-ip">{allumee ? adresses(machine) : 'éteinte'}</span>
          </button>
        )
      })}
    </div>
  )
}
