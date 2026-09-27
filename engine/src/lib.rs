pub mod auth;
pub mod claude;
pub mod cleanup;
pub mod config;
pub mod error;
pub mod gmail;
pub mod model;
pub mod organize;
pub mod rules;
pub mod settings;
mod text;

pub use error::{Error, ErrorCode, Result};
pub use gmail::Gmail;
pub use settings::{Settings, Storage};
