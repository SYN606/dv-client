//! User accounts, Microsoft OAuth PKCE, and profile management subsystem.

pub mod minecraft_auth;
pub mod oauth;

pub use minecraft_auth::{AuthError, MicrosoftAuthClient, MinecraftAccount};
pub use oauth::PkceChallenge;
