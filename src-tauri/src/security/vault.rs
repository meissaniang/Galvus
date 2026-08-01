//! Accès au coffre natif de l'OS (Keychain sur macOS) via `keyring`.
//!
//! Aucun secret n'est stocké en clair sur disque : la clé de chiffrement de la
//! base SQLite vit uniquement dans le coffre natif (cf. CLAUDE.md).

use keyring::Entry;

use crate::errors::AppError;

const SERVICE: &str = "com.galvus.app";
const DB_KEY_ACCOUNT: &str = "db-encryption-key";

/// Récupère la clé de chiffrement de la base, en la générant au premier lancement.
///
/// La clé est une chaîne hexadécimale de 64 caractères (32 octets aléatoires),
/// utilisée comme passphrase SQLCipher.
pub fn get_or_create_db_key() -> Result<String, AppError> {
    let entry = Entry::new(SERVICE, DB_KEY_ACCOUNT)?;
    match entry.get_password() {
        Ok(key) => Ok(key),
        Err(keyring::Error::NoEntry) => {
            let key = generate_hex_key();
            entry.set_password(&key)?;
            Ok(key)
        }
        Err(e) => Err(AppError::Vault(e.to_string())),
    }
}

/// Génère 32 octets aléatoires encodés en hexadécimal.
fn generate_hex_key() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("générateur aléatoire indisponible");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
