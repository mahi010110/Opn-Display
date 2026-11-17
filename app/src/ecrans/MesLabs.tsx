import type { EtatLab, Rapport } from '../types'
import { Fenetre } from '../ui/Fenetre'
import { Jauge, texteMemoire } from '../ui/Jauge'
import { go } from '../ui/TonPc'
import type { Gestes } from '../ui/Coque'

/** La liste des labs, et ce que le PC sait faire. */
export function MesLabs(props: {
  rapport: Rapport
  etat: EtatLab | null
  occupe: string | null
  erreur: string | null
  gestes: Gestes
  /** Une panne posée attend dans le lab. */
  enPanne: boolean
  onOuvrir: () => void
}) {
  const { rapport, etat, occupe } = props
  const badge = props.enPanne && <span className="oc-badge oc-badge--danger">Panne</span>
  const { totaleGo, libreGo } = rapport.memoire
  const enMarche = etat?.machines.filter((m) => m.marche === 'enMarche') ?? []
  const memoire = enMarche.reduce((total, m) => total + (m.memoire ?? 0), 0)
  const nombre = etat?.lab.machines.length ?? 0

  const lancer = () => {
    props.onOuvrir()
    void props.gestes.lancer()
  }

  return (
    <>
      <div className="lab-barre">
        <h1 className="lab-nom">Mes labs</h1>
      </div>

      <main className="page page--deux">
        <div className="colonne">
          {etat && enMarche.length > 0 ? (
            <Fenetre titre="En marche" active>
              <div className="en-marche">
                <div className="en-marche-texte">
                  <p className="en-marche-nom">
                    {etat.lab.nom} {badge}
                  </p>
                  <p className="mono">
                    {occupe ?? `${enMarche.length} ${enMarche.length > 1 ? 'machines' : 'machine'} en marche sur ${nombre}, ${texteMemoire(memoire)} de RAM`}
                  </p>
                </div>
                <div className="lab-gestes">
                  <button className="oc-btn" disabled={occupe !== null} onClick={() => void props.gestes.arreter()}>
                    <i className="ic i-arreter" aria-hidden="true"></i>Arrêter
                  </button>
                  <button className="oc-btn oc-btn--primary" onClick={props.onOuvrir}>
                    Ouvrir
                  </button>
                </div>
              </div>
            </Fenetre>
          ) : (
            <Fenetre titre="Arrêtés" flush>
              <table className="table">
                <thead>
                  <tr>
                    <th>Lab</th>
                    <th>Machines</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {etat && (
                    <tr>
                      <td className="nom">
                        {etat.lab.nom} {badge}
                      </td>
                      <td className="chiffre">{nombre}</td>
                      <td className="fin">
                        <button className="oc-btn oc-btn--sm" disabled={occupe !== null} onClick={lancer}>
                          Lancer
                        </button>
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </Fenetre>
          )}
          {props.erreur && <p className="note erreur">{props.erreur}</p>}
        </div>

        <Fenetre titre="Ton PC">
          <dl className="fiche">
            <dt>Mémoire</dt>
            <dd>
              {go(Math.round(totaleGo))} Go, {go(libreGo)} libres
            </dd>
            <dt>Conteneurs</dt>
            <dd>oui</dd>
            <dt>Machines virtuelles</dt>
            <dd className={rapport.kvm ? undefined : 'manque'}>{rapport.kvm ? 'oui' : 'non'}</dd>
            <dt>WiFi simulé</dt>
            <dd className={rapport.wifiSimule ? undefined : 'manque'}>{rapport.wifiSimule ? 'oui' : 'non'}</dd>
          </dl>
        </Fenetre>
      </main>

      <footer className="etat">
        <span>
          RAM du PC
          <Jauge part={(totaleGo - libreGo) / totaleGo} />
          {go(libreGo)} Go libres sur {go(Math.round(totaleGo))}
        </span>
      </footer>
    </>
  )
}
