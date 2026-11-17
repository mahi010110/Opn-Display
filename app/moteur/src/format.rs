const OCTETS_PAR_GO: f64 = 1_073_741_824.0;

pub(crate) fn go(octets: u64) -> f64 {
    octets as f64 / OCTETS_PAR_GO
}

/// Un nombre de Go comme on l'écrit : « 32 », « 24,3 ».
pub(crate) fn texte_go(valeur: f64) -> String {
    let arrondi = (valeur * 10.0).round() / 10.0;
    if arrondi.fract() == 0.0 {
        format!("{arrondi:.0}")
    } else {
        format!("{arrondi:.1}").replace('.', ",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecrit_les_go_avec_une_virgule() {
        assert_eq!(texte_go(32.0), "32");
        assert_eq!(texte_go(24.31), "24,3");
        assert_eq!(texte_go(15.96), "16");
        assert_eq!(texte_go(0.4), "0,4");
    }

    #[test]
    fn convertit_les_octets() {
        assert_eq!(go(8 * 1_073_741_824), 8.0);
    }
}
