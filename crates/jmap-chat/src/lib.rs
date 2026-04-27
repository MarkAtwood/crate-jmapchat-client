pub mod auth;
pub mod blob;
pub mod client;
pub mod error;
pub mod jmap;
pub mod methods;
pub mod sse;
pub mod types;
pub mod utils;
pub mod ws;

// Core client types
pub use client::JmapChatClient;
pub use methods::SessionClient;

// Error type
pub use error::ClientError;

// Auth providers
pub use auth::{AuthProvider, BasicAuth, BearerAuth, CustomCaAuth, NoneAuth};

// JMAP core types
pub use jmap::{Id, Session, UTCDate};
