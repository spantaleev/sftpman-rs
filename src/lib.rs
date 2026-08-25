// Our error types (see `errors.rs`) intentionally carry the `std::process::Command` that failed
// and its `std::process::Output`, so that callers can report exactly what was run and why it failed.
// This makes the error enums fairly large, which clippy warns about (`result_large_err`).
// Boxing these payloads would shrink the enums, but would also make the public API more awkward
// (and break compatibility for library consumers), so we accept the size instead.
#![allow(clippy::result_large_err)]

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
