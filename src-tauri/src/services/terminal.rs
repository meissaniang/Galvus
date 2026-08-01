//! Gestion des sessions de terminal (PTY).
//!
//! Chaque session ouvre un pseudo-terminal (via `portable-pty`) dans lequel on
//! exécute le binaire `ssh` du système. `portable-pty` ne gère QUE le PTY : le
//! protocole SSH reste entièrement délégué à OpenSSH (cf. CLAUDE.md).
//!
//! Cycle de vie : `open` (spawn + thread de lecture qui émet la sortie vers le
//! frontend), `write` (frappes clavier), `resize` (redimensionnement), `close`
//! (fin de session).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::errors::AppError;

/// Événement : fragment de sortie du PTY (octets bruts, pour préserver l'UTF-8
/// multi-octets à cheval sur deux lectures).
#[derive(Clone, Serialize)]
struct OutputPayload {
    #[serde(rename = "sessionId")]
    session_id: String,
    data: Vec<u8>,
}

/// Événement : la session s'est terminée (processus `ssh` fermé).
#[derive(Clone, Serialize)]
struct ExitPayload {
    #[serde(rename = "sessionId")]
    session_id: String,
}

/// Ressources d'une session vivante.
struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
}

/// Registre des sessions actives, partagé comme état Tauri.
#[derive(Default)]
pub struct TerminalManager {
    sessions: Mutex<HashMap<String, Session>>,
}

impl TerminalManager {
    /// Ouvre une session : PTY + `program args…`, et démarre le thread de lecture.
    pub fn open(
        &self,
        app: AppHandle,
        session_id: String,
        program: String,
        args: Vec<String>,
        cols: u16,
        rows: u16,
    ) -> Result<(), AppError> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::Command(e.to_string()))?;

        let mut cmd = CommandBuilder::new(program);
        cmd.args(args);
        cmd.env("TERM", "xterm-256color");

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::Command(e.to_string()))?;
        // Le côté esclave n'est plus nécessaire dans le parent une fois `ssh` lancé.
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| AppError::Command(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| AppError::Command(e.to_string()))?;

        // Thread de lecture : relaie chaque fragment vers le frontend.
        let reader_app = app.clone();
        let reader_id = session_id.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let _ = reader_app.emit(
                            "terminal://output",
                            OutputPayload {
                                session_id: reader_id.clone(),
                                data: buf[..n].to_vec(),
                            },
                        );
                    }
                }
            }
            let _ = reader_app.emit(
                "terminal://exit",
                ExitPayload {
                    session_id: reader_id,
                },
            );
        });

        let session = Session {
            master: pair.master,
            writer,
            child,
        };
        self.sessions
            .lock()
            .expect("sessions mutex poisoned")
            .insert(session_id, session);
        Ok(())
    }

    /// Transmet des octets (frappes clavier) à la session.
    pub fn write(&self, session_id: &str, data: &[u8]) -> Result<(), AppError> {
        let mut sessions = self.sessions.lock().expect("sessions mutex poisoned");
        if let Some(session) = sessions.get_mut(session_id) {
            session
                .writer
                .write_all(data)
                .map_err(|e| AppError::Io(e.to_string()))?;
            session
                .writer
                .flush()
                .map_err(|e| AppError::Io(e.to_string()))?;
        }
        Ok(())
    }

    /// Redimensionne le PTY (nombre de colonnes/lignes).
    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), AppError> {
        let sessions = self.sessions.lock().expect("sessions mutex poisoned");
        if let Some(session) = sessions.get(session_id) {
            session
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| AppError::Command(e.to_string()))?;
        }
        Ok(())
    }

    /// Termine la session et tue le processus enfant.
    pub fn close(&self, session_id: &str) -> Result<(), AppError> {
        let mut sessions = self.sessions.lock().expect("sessions mutex poisoned");
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.child.kill();
        }
        Ok(())
    }
}
