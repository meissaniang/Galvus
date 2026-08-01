//! Interfaces de synchronisation (Phase 2) — DÉFINITIONS SEULEMENT.
//!
//! Aucune implémentation réseau ici : la couche cloud (NestJS + JWT +
//! PostgreSQL + Redis) sera branchée plus tard en fournissant un type qui
//! implémente [`SyncProvider`], sans modifier le reste du backend.

use crate::errors::AppError;
use crate::models::Server;

/// Instantané synchronisable de l'état local.
#[derive(Debug, Clone)]
pub struct SyncSnapshot {
    pub servers: Vec<Server>,
    // snippets / préférences seront ajoutés avec la Phase 2.
}

/// Fournisseur de synchronisation (contrat Phase 2).
#[allow(dead_code)]
pub trait SyncProvider {
    fn is_enabled(&self) -> bool;
    fn pull(&self) -> Result<SyncSnapshot, AppError>;
    fn push(&self, snapshot: &SyncSnapshot) -> Result<(), AppError>;
}
