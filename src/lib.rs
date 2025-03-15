mod auth_type;

#[cfg(feature = "cli")]
pub mod cli;

mod errors;
mod manager;
mod model;
mod utils;

pub use auth_type::AuthType;
pub use errors::{ManagerInitError, PreflightCheckError, SftpManError};
pub use manager::Manager;
pub use model::{DEFAULT_MOUNT_PATH_PREFIX, FilesystemMountDefinition, MountState};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Re-exports

// Re-export to allow people to use `FilesystemMountDefinition::validate()` (which requires the `validator::Validate` trait)
pub use validator;
