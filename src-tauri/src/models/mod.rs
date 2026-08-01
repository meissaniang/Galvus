//! Structures de données du domaine (Server, SshKey, Tag, Settings…).

mod host;
mod key;
mod server;

pub use host::Host;
pub use key::SshKey;
pub use server::{Server, ServerInput};
