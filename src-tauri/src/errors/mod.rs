//! Types d'erreurs centralisés et conversions (thiserror), sérialisables vers le frontend.

use serde::{Serialize, Serializer};

/// Erreur applicative renvoyée au frontend.
///
/// Les commandes Tauri retournent `Result<T, AppError>`. L'erreur est
/// sérialisée en chaîne lisible côté Vue.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Erreur d'entrée/sortie : {0}")]
    Io(String),

    #[error("Répertoire personnel introuvable")]
    HomeDirNotFound,

    #[error("Échec de la commande externe : {0}")]
    Command(String),
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Io(value.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
