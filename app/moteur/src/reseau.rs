//! Les adresses IPv4 des labs : les lire, et savoir qui est dans quel réseau.

use std::net::Ipv4Addr;

/// Une adresse avec son masque, « 10.0.1.10/24 ».
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adresse {
    pub ip: Ipv4Addr,
    pub masque: u8,
}

impl Adresse {
    pub fn lire(texte: &str) -> Option<Adresse> {
        let (ip, masque) = texte.trim().split_once('/')?;
        let masque: u8 = masque.parse().ok()?;
        (masque <= 32).then_some(Adresse {
            ip: ip.parse().ok()?,
            masque,
        })
    }

    fn bits(masque: u8) -> u32 {
        if masque == 0 {
            0
        } else {
            u32::MAX << (32 - masque)
        }
    }

    /// Le réseau, « 10.0.1.0/24 ».
    pub fn reseau(&self) -> Adresse {
        Adresse {
            ip: Ipv4Addr::from(u32::from(self.ip) & Self::bits(self.masque)),
            masque: self.masque,
        }
    }

    pub fn contient(&self, ip: Ipv4Addr) -> bool {
        let bits = Self::bits(self.masque);
        u32::from(ip) & bits == u32::from(self.ip) & bits
    }

    /// L'adresse est celle du réseau lui-même, ou celle de diffusion : une
    /// machine ne peut pas la prendre.
    pub fn reservee(&self) -> bool {
        if self.masque >= 31 {
            return false;
        }
        let hote = u32::from(self.ip) & !Self::bits(self.masque);
        hote == 0 || hote == !Self::bits(self.masque)
    }
}

impl std::fmt::Display for Adresse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.ip, self.masque)
    }
}

/// `ip` est-elle dans le réseau de `adresse` (« 10.0.2.1/24 ») ?
pub fn dans_le_reseau(ip: &str, adresse: &str) -> bool {
    match (ip.trim().parse::<Ipv4Addr>(), Adresse::lire(adresse)) {
        (Ok(ip), Some(adresse)) => adresse.contient(ip),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lit_une_adresse_et_son_reseau() {
        let a = Adresse::lire("10.0.1.10/24").unwrap();
        assert_eq!(a.reseau().to_string(), "10.0.1.0/24");
        assert!(a.contient("10.0.1.200".parse().unwrap()));
        assert!(!a.contient("10.0.2.1".parse().unwrap()));
        assert_eq!(Adresse::lire("10.0.1.10"), None);
        assert_eq!(Adresse::lire("10.0.1.10/33"), None);
        assert_eq!(Adresse::lire("pc1/24"), None);
    }

    #[test]
    fn reconnait_les_adresses_reservees() {
        assert!(Adresse::lire("10.0.1.0/24").unwrap().reservee());
        assert!(Adresse::lire("10.0.1.255/24").unwrap().reservee());
        assert!(!Adresse::lire("10.0.1.1/24").unwrap().reservee());
        assert!(!Adresse::lire("10.0.1.0/31").unwrap().reservee());
    }

    #[test]
    fn dit_si_une_passerelle_est_dans_le_reseau() {
        assert!(dans_le_reseau("10.0.2.2", "10.0.2.1/24"));
        assert!(!dans_le_reseau("10.0.3.2", "10.0.2.1/24"));
        assert!(!dans_le_reseau("pas une ip", "10.0.2.1/24"));
    }
}
