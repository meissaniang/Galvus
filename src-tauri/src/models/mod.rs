//! Structures de données du domaine (Server, SshKey, Tag, Settings…).

mod host;
mod key;
mod server;
mod tunnel;

pub use host::{ConfigHostInput, Host};
pub use key::SshKey;
pub use server::{Server, ServerInput};
pub use tunnel::{Tunnel, TunnelInput};
