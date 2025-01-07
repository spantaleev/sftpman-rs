use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManagerInitError {
    /// Happens when `directories::ProjectDirs` cannot retrieve a valid home directory path from the operating system.
    #[error("The config directory could not be determined")]
    NoConfigDirectory,
}

#[derive(Error, Debug)]
pub enum PreflightCheckError {
    /// Happens when a required command (e.g. `sshfs`) cannot be executed.
    #[error("A required command could not be executed")]
    CommandExecution(std::process::Command, std::io::Error),

    /// Happens when a required command was executed, but was unsuccessful.
    #[error("A required command was executed, but was unsuccessful")]
    CommandUnsuccessful(std::process::Command, std::process::Output),

    /// Happens when the default mount path (e.g. `/mnt/sshfs`) does not exist and cannot be prepared.
    #[error("The default mount path (/mnt/sshfs) could not be prepared. Mounting there will fail until this is fixed")]
    DefaultBasePathIO(std::path::PathBuf, std::io::Error),

    /// Happens when a test directory (e.g. `/mnt/sshfs/_sftpman_test_1234567890`) under the default mount path could not be prepared.
    #[error("A test directory under the default mount path (/mnt/sshfs) could not be prepared. Mounting there will fail until this is fixed")]
    TestUnderBasePathIO(std::path::PathBuf, std::io::Error),
}

#[derive(Error, Debug)]
pub enum SftpManError {
    /// Happens when any other generic error occurs.
    #[error("Generic error")]
    Generic(String),

    /// Happens when the mounts configuration directory does not exist.
    #[error("The mounts configuration directory does not exist")]
    NoMountsConfigDirectory,

    /// Happens when `mnt::get_submounts` fails to parse the mount list.
    #[error("The current mounts could not be parsed")]
    MountListParse(#[from] mnt::ParseError),

    /// Happens when the mount config definition cannot be read.
    #[error("The mount config definition could not be read")]
    FilesystemMountDefinitionRead(std::path::PathBuf, std::io::Error),

    /// Happens when the mount config definition file cannot be removed.
    #[error("The mount config definition could not be removed")]
    FilesystemMountDefinitionRemove(std::path::PathBuf, std::io::Error),

    /// Happens when the mount config definition cannot be parsed as JSON.
    #[error("The mount config definition could not be parsed")]
    JSON(std::path::PathBuf, serde_json::Error),

    /// Happens when a given mount path was found, but it was not of the expected type (e.g. `fuse.sshfs`).
    #[error("The mount path  was found, but it was not of the expected type")]
    MountVfsTypeMismatch {
        path: std::path::PathBuf,
        found_vfs_type: String,
        expected_vfs_type: String,
    },

    /// Happens when the mount command cannot be constructed.
    #[error("The mount command could not be constructed")]
    MountCommandBuilding(String),

    /// Happens when the mount command cannot be executed.
    #[error("The mount command could not be executed")]
    CommandExecution(std::process::Command, std::io::Error),

    /// Happens when the mount command was executed, but was unsuccessful.
    #[error("The command was executed, but was unsuccessful")]
    CommandUnsuccessful(std::process::Command, std::process::Output),

    /// Happens when the mount directory could not be prepared.
    #[error("The mount directory could not be prepared")]
    IO(std::path::PathBuf, std::io::Error),
}
