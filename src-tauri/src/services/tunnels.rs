//! Supervision des tunnels SSH : lance `ssh -N -L/-R/-D …` en processus enfant
//! et permet de les arrêter. Utilise le binaire `ssh` système (auth par clé /
//! agent ; les tunnels non interactifs ne gèrent pas de saisie de mot de passe).

use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::Mutex;

use crate::errors::AppError;
use crate::models::Tunnel;

#[derive(Default)]
pub struct TunnelManager {
    running: Mutex<HashMap<i64, Child>>,
}

impl TunnelManager {
    /// Construit les arguments `ssh` d'un tunnel.
    fn build_args(tunnel: &Tunnel) -> Result<Vec<String>, AppError> {
        let mut args = vec!["-N".to_string(), "-o".into(), "ExitOnForwardFailure=yes".into()];
        let host = tunnel.target_host.clone().unwrap_or_default();
        let port = tunnel.target_port.unwrap_or(0);
        match tunnel.kind.as_str() {
            "local" => args.extend([
                "-L".into(),
                format!("{}:{}:{}", tunnel.listen_port, host, port),
            ]),
            "remote" => args.extend([
                "-R".into(),
                format!("{}:{}:{}", tunnel.listen_port, host, port),
            ]),
            "dynamic" => args.extend(["-D".into(), tunnel.listen_port.to_string()]),
            other => return Err(AppError::Command(format!("type de tunnel inconnu : {other}"))),
        }
        args.push(tunnel.ssh_target.clone());
        Ok(args)
    }

    /// Démarre un tunnel. Sans effet s'il tourne déjà.
    pub fn start(&self, tunnel: &Tunnel) -> Result<(), AppError> {
        let mut running = self.running.lock().expect("tunnels mutex poisoned");
        if running.contains_key(&tunnel.id) {
            return Ok(());
        }
        let args = Self::build_args(tunnel)?;
        let child = Command::new("ssh")
            .args(&args)
            .spawn()
            .map_err(|e| AppError::Command(e.to_string()))?;
        log::info!("tunnel démarré #{} ({})", tunnel.id, tunnel.name);
        running.insert(tunnel.id, child);
        Ok(())
    }

    /// Arrête un tunnel.
    pub fn stop(&self, id: i64) -> Result<(), AppError> {
        let mut running = self.running.lock().expect("tunnels mutex poisoned");
        if let Some(mut child) = running.remove(&id) {
            let _ = child.kill();
            log::info!("tunnel arrêté #{id}");
        }
        Ok(())
    }

    /// Identifiants des tunnels effectivement actifs (nettoie les morts).
    pub fn running_ids(&self) -> Vec<i64> {
        let mut running = self.running.lock().expect("tunnels mutex poisoned");
        let mut alive = Vec::new();
        running.retain(|id, child| match child.try_wait() {
            Ok(Some(_)) => false, // processus terminé
            _ => {
                alive.push(*id);
                true
            }
        });
        alive
    }
}
