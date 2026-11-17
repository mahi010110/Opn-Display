//! Le terminal d'une machine du lab : un shell dans la machine, ouvert par
//! `incus exec` dans un pseudo-terminal. Sous Windows, wsl.exe tourne dans une
//! console ConPTY et passe la taille de l'écran à Debian.

use std::io::{Read, Write};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::lab::{Lab, projet};
use crate::systeme;

pub struct Terminal {
    maitre: Box<dyn MasterPty + Send>,
    /// Les touches partent dans l'ordre, par un fil à part : une écriture
    /// ne bloque jamais l'app, même quand le shell ne lit plus.
    touches: Sender<Vec<u8>>,
    tueur: Box<dyn ChildKiller + Send + Sync>,
}

fn taille(colonnes: u16, lignes: u16) -> PtySize {
    PtySize {
        rows: lignes.max(2),
        cols: colonnes.max(2),
        pixel_width: 0,
        pixel_height: 0,
    }
}

/// Ouvre un shell dans `machine`. `recevoir` reçoit ce que le terminal
/// affiche ; `fin` est appelée une fois, quand le shell s'est arrêté.
pub fn ouvrir(
    lab: &Lab,
    machine: &str,
    colonnes: u16,
    lignes: u16,
    mut recevoir: impl FnMut(Vec<u8>) + Send + 'static,
    fin: impl FnOnce() + Send + 'static,
) -> Result<Terminal, String> {
    if !lab.machines.iter().any(|m| m.nom == machine) {
        return Err(format!("{machine} n'est pas une machine de ce lab."));
    }
    let projet = projet(lab);
    let ligne = systeme::ligne(
        "incus",
        &[
            "exec",
            machine,
            "--project",
            &projet,
            "--env",
            "TERM=xterm-256color",
            "--",
            "bash",
            "-l",
        ],
    );
    let paire = native_pty_system()
        .openpty(taille(colonnes, lignes))
        .map_err(|e| format!("Le terminal ne s'ouvre pas : {e}"))?;
    let mut commande = CommandBuilder::new(&ligne[0]);
    commande.args(&ligne[1..]);
    commande.env("TERM", "xterm-256color");
    let mut enfant = paire
        .slave
        .spawn_command(commande)
        .map_err(|e| format!("Le terminal ne s'ouvre pas : {e}"))?;
    // Le shell garde son côté du terminal ; le nôtre doit se fermer pour que
    // la lecture s'arrête quand il quitte.
    drop(paire.slave);

    let mut lecteur = paire
        .master
        .try_clone_reader()
        .map_err(|e| format!("Le terminal ne se lit pas : {e}"))?;
    let mut ecrivain = paire
        .master
        .take_writer()
        .map_err(|e| format!("Le terminal ne s'écrit pas : {e}"))?;
    let tueur = enfant.clone_killer();
    let (touches, a_ecrire) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        for octets in a_ecrire {
            if ecrivain
                .write_all(&octets)
                .and_then(|_| ecrivain.flush())
                .is_err()
            {
                break;
            }
        }
    });

    let fin = Arc::new(Mutex::new(Some(fin)));
    let fin_lecture = fin.clone();
    thread::spawn(move || {
        let mut tampon = [0u8; 16 * 1024];
        loop {
            match lecteur.read(&mut tampon) {
                Ok(0) | Err(_) => break,
                Ok(lus) => recevoir(tampon[..lus].to_vec()),
            }
        }
        appeler_une_fois(&fin_lecture);
    });
    // Sous Windows, la lecture ne s'arrête qu'à la fermeture de la console :
    // c'est la fin du processus qui dit que le shell est parti.
    thread::spawn(move || {
        let _ = enfant.wait();
        appeler_une_fois(&fin);
    });

    Ok(Terminal {
        maitre: paire.master,
        touches,
        tueur,
    })
}

fn appeler_une_fois<F: FnOnce()>(fin: &Mutex<Option<F>>) {
    let fin = fin.lock().ok().and_then(|mut f| f.take());
    if let Some(fin) = fin {
        fin();
    }
}

impl Terminal {
    pub fn ecrire(&self, octets: &[u8]) -> Result<(), String> {
        self.touches
            .send(octets.to_vec())
            .map_err(|_| "Le terminal ne répond plus.".to_string())
    }

    pub fn redimensionner(&self, colonnes: u16, lignes: u16) -> Result<(), String> {
        self.maitre
            .resize(taille(colonnes, lignes))
            .map_err(|e| format!("Le terminal ne change pas de taille : {e}"))
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.tueur.kill();
    }
}
