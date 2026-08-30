//! SSH : résolution de config via 'ssh -G', scan des clés ~/.ssh, connexions.

pub mod agent;
pub mod config;
pub mod keys;

/// Options ajoutées à toute session interactive pour que la perte de la liaison
/// soit détectée.
///
/// Sans elles, `ssh` n'envoie rien sur un lien mort : le socket reste
/// « établi » côté client, le processus ne rend jamais la main, le PTY ne
/// ferme jamais — et le pane paraît gelé indéfiniment (frappes avalées sans
/// écho). C'est ce que voyait l'utilisateur au moindre coupure de réseau ou de
/// veille de la machine.
///
/// - `ServerAliveInterval` / `ServerAliveCountMax` : trois sondes applicatives
///   espacées de 15 s, donc coupure au bout de ~45 s avec le code 255. C'est le
///   seul mécanisme fiable : le keepalive TCP du système se compte en heures.
/// - `ConnectTimeout` : la phase de connexion elle-même ne peut plus attendre
///   sans fin un hôte injoignable.
///
/// Posées sur la ligne de commande, elles priment sur `~/.ssh/config` : le
/// diagnostic d'une session morte ne doit pas dépendre de la config de chacun.
const RESILIENCE_OPTIONS: [&str; 8] = [
    "-o",
    "ServerAliveInterval=15",
    "-o",
    "ServerAliveCountMax=3",
    "-o",
    "ConnectTimeout=15",
    "-o",
    "TCPKeepAlive=yes",
];

/// Préfixe les arguments d'une session par les options de résilience.
///
/// Elles passent en tête : `ssh` veut ses options avant la destination.
pub fn with_resilience(args: Vec<String>) -> Vec<String> {
    let mut full: Vec<String> = RESILIENCE_OPTIONS
        .iter()
        .map(|o| (*o).to_string())
        .collect();
    full.extend(args);
    full
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_precedent_la_destination() {
        let args = with_resilience(vec!["-p".into(), "2222".into(), "user@host".into()]);
        assert_eq!(args.last().unwrap(), "user@host");
        assert!(args
            .windows(2)
            .any(|w| w == ["-o", "ServerAliveInterval=15"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["-o", "ServerAliveCountMax=3"]));
    }

    #[test]
    fn un_alias_seul_reste_en_dernier() {
        let args = with_resilience(vec!["vps-1".into()]);
        assert_eq!(args.last().unwrap(), "vps-1");
    }
}
