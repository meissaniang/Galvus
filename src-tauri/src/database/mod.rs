//! SQLite chiffrée au repos : connexion, migrations, accès bas niveau (repositories Rust).

mod connection;
pub mod servers_repository;

pub use connection::Database;
