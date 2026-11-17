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
    recevoir: impl FnMut(Vec<u8>) + Send + 'static,
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
    lancer(&ligne, colonnes, lignes, recevoir, fin)
}

/// Lance `ligne` dans un pseudo-terminal : ConPTY sous Windows, un pty
/// sous Linux.
fn lancer(
    ligne: &[String],
    colonnes: u16,
    lignes: u16,
    mut recevoir: impl FnMut(Vec<u8>) + Send + 'static,
    fin: impl FnOnce() + Send + 'static,
) -> Result<Terminal, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    /// Le même chemin que les terminaux du lab, avec un programme qui
    /// existe partout. Sous Windows, il passe par ConPTY, comme wsl.exe.
    #[test]
    fn un_programme_tourne_dans_le_pseudo_terminal() {
        #[cfg(windows)]
        let ligne = ["cmd.exe", "/c", "echo bonjour"];
        #[cfg(not(windows))]
        let ligne = ["sh", "-c", "echo bonjour; stty size"];
        let ligne: Vec<String> = ligne.iter().map(|m| m.to_string()).collect();

        let (octets, recus) = mpsc::channel::<Vec<u8>>();
        let (fini, fin) = mpsc::channel();
        let terminal = lancer(
            &ligne,
            100,
            30,
            move |o| {
                let _ = octets.send(o);
            },
            move || {
                let _ = fini.send(());
            },
        )
        .unwrap();
        let limite = Instant::now() + Duration::from_secs(20);
        let mut recu = Vec::new();
        let mut repondu = false;
        while fin.try_recv().is_err() {
            assert!(
                Instant::now() < limite,
                "le programme ne s'est pas arrêté, reçu : {:?}",
                String::from_utf8_lossy(&recu)
            );
            if let Ok(o) = recus.recv_timeout(Duration::from_millis(100)) {
                recu.extend(o);
            }
            // ConPTY demande où est le curseur (ESC [6n) et attend la
            // réponse avant de lancer le programme. Dans l'app, xterm.js
            // répond de lui-même.
            if !repondu && recu.windows(4).any(|w| w == b"\x1b[6n") {
                terminal.ecrire(b"\x1b[1;1R").unwrap();
                repondu = true;
            }
        }
        drop(terminal);
        std::thread::sleep(Duration::from_millis(300));
        recu.extend(recus.try_iter().flatten());
        let texte = String::from_utf8_lossy(&recu).into_owned();
        assert!(texte.contains("bonjour"), "reçu : {texte:?}");
        #[cfg(not(windows))]
        assert!(texte.contains("30 100"), "reçu : {texte:?}");
    }
}
