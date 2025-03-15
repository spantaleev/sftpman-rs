use std::process::Command;

use serde::{Deserialize, Serialize};

use validator::{Validate, ValidationError};

use crate::utils::command::command_to_string;

use crate::auth_type::{
    AuthType, deserialize_auth_type_from_string, serialize_auth_type_to_string,
};

use crate::errors::SftpManError;

pub const DEFAULT_MOUNT_PATH_PREFIX: &str = "/mnt/sshfs";

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[validate(schema(
    function = "validate_ssh_key_for_publickey_auth",
    skip_on_field_errors = false
))]
pub struct FilesystemMountDefinition {
    /// Unique identifier for this definition.
    /// If `mount_dest_path` is `None`, this will also influence where the filesystem gets mounted locally (see `local_mount_path()`).
    #[validate(
        length(min = 1, message = "An ID must be provided."),
        custom(
            function = "validate_id",
            message = "The ID must be a valid identifier (alphanumeric characters, underscores, dashes, or dots)."
        )
    )]
    pub id: String,

    /// Hostname or IP address of the remote machine
    #[validate(length(min = 1, message = "A host must be provided."))]
    pub host: String,

    /// Port number of the remote machine (e.g. `22`).
    pub port: u16,

    /// Username to SSH in on the remote machine (e.g. `user`).
    #[validate(length(min = 1, message = "A user must be provided."))]
    pub user: String,

    /// Mount options to pass to sshfs (-o).
    /// Example: [`follow_symlinks`, `rename`]
    #[serde(rename = "mountOptions")]
    pub mount_options: Vec<String>,

    /// Path on the remote server that will be mounted locally (e.g. `/storage`).
    #[serde(rename = "mountPoint")]
    #[validate(
        length(min = 1, message = "A remote path must be provided."),
        custom(
            function = "validate_absolute_path",
            message = "The remote path must be absolute."
        )
    )]
    pub remote_path: String,

    /// Path where the filesystem will be mounted locally (e.g. `/home/user/storage`).
    /// If not provided, it defaults to `{DEFAULT_MOUNT_PATH_PREFIX}/{id}`.
    #[serde(rename = "mountDestPath")]
    #[validate(
        length(min = 1, message = "A local mount destination path must be provided."),
        custom(
            function = "validate_absolute_path",
            message = "The local mount destination path must be absolute."
        )
    )]
    pub mount_dest_path: Option<String>,

    /// Command to run before mounting (e.g. `/bin/true`)
    #[serde(rename = "beforeMount")]
    #[serde(default)]
    pub cmd_before_mount: String,

    /// Authentication method.
    /// Most of the potential values match SSH's `PreferredAuthentications` list, but some are special values that we recognize & handle here.
    #[serde(rename = "authType")]
    #[serde(
        serialize_with = "serialize_auth_type_to_string",
        deserialize_with = "deserialize_auth_type_from_string"
    )]
    pub auth_type: AuthType,

    /// Path to an SSH private key (e.g. `/home/user/.ssh/id_ed25519`) for authentication types (like `AuthType::PublicKey`) that use a key.
    #[serde(rename = "sshKey")]
    pub ssh_key: String,
}

const SSH_DEFAULT_TIMEOUT: u32 = 10;

impl Default for FilesystemMountDefinition {
    fn default() -> Self {
        FilesystemMountDefinition {
            id: String::new(),
            host: String::new(),
            port: 22,
            user: String::new(),
            mount_options: Vec::new(),
            remote_path: String::new(),
            mount_dest_path: None,
            cmd_before_mount: String::new(),
            auth_type: AuthType::PublicKey,
            ssh_key: String::new(),
        }
    }
}

impl FilesystemMountDefinition {
    pub fn from_json_string(contents: &str) -> Result<Self, serde_json::Error> {
        let deserialized: Self = serde_json::from_str(contents)?;
        Ok(deserialized)
    }

    pub fn to_json_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Returns the local mount path for this definition.
    /// If `mount_dest_path` is not `None` for this definition, it will be used.
    /// Otherwise, the default mount path (`DEFAULT_MOUNT_PATH_PREFIX`) will be used (e.g. `/mnt/sshfs/{id}`).
    pub fn local_mount_path(&self) -> String {
        match &self.mount_dest_path {
            Some(path) => path.clone(),
            None => format!("{0}/{1}", DEFAULT_MOUNT_PATH_PREFIX, self.id),
        }
    }

    /// Returns a list of commands for mounting the filesystem definition.
    /// Mounting is performed via `sshfs` and `ssh` commands.
    pub fn mount_commands(&self) -> Result<Vec<Command>, SftpManError> {
        log::debug!("{0}: building list of mount commands", self.id);

        let mut list: Vec<Command> = Vec::new();

        if !self.cmd_before_mount.is_empty() {
            if self.cmd_before_mount == "/bin/true" || self.cmd_before_mount == "true" {
                // sftpman-gtk used to hardcode `/bin/true` or `true` as a before-mount command.
                // We don't really need to run this.
                log::debug!(
                    "{0}: ignoring no-op before-mount command {1}",
                    self.id,
                    self.cmd_before_mount
                );
            } else {
                let mut program_name = "";
                let mut args: Vec<&str> = Vec::new();

                for (idx, arg) in self.cmd_before_mount.split(' ').enumerate() {
                    match idx {
                        0 => {
                            program_name = arg;
                        }
                        _ => {
                            args.push(arg);
                        }
                    }
                }

                if program_name.is_empty() {
                    return Err(SftpManError::MountCommandBuilding(format!(
                        "could not extract program name from {0}",
                        self.cmd_before_mount
                    )));
                }

                let mut cmd_before = Command::new(program_name);
                for arg in args {
                    cmd_before.arg(arg);
                }

                list.push(cmd_before);
            }
        }

        let mut cmd_ssh = Command::new("ssh");
        cmd_ssh
            .arg("-p")
            .arg(self.port.to_string())
            .arg("-o")
            .arg(format!("ConnectTimeout={0}", SSH_DEFAULT_TIMEOUT));

        match &self.auth_type {
            AuthType::PublicKey => {
                cmd_ssh.arg(format!(
                    "-o PreferredAuthentications={0}",
                    AuthType::PublicKey.to_static_str()
                ));
                cmd_ssh.arg(format!("-i {0}", self.ssh_key));
            }
            AuthType::AuthenticationAgent => {
                // By not specifying a key and preferred authentication type,
                // we're hoping to delegate all this to an already running SSH agent, if available.
            }
            any_other => {
                cmd_ssh.arg(format!(
                    "-o PreferredAuthentications={0}",
                    any_other.to_static_str()
                ));
            }
        };

        let mut cmd_sshfs = Command::new("sshfs");
        cmd_sshfs
            // Add mount options prefixed with "-o" (ignored if empty).
            .args(self.mount_options.iter().flat_map(|opt| ["-o", opt]))
            // Add the formatted SSH command as an sshfs option.
            .arg("-o")
            .arg(format!("ssh_command={0}", command_to_string(&cmd_ssh)))
            // We use `[]` around the host to avoid issues with hostnames (IPv6 addresses) containing `:`.
            // This also works well for IPv4 addresses and name-based hostnames.
            .arg(format!(
                "{0}@[{1}]:{2}",
                self.user, self.host, self.remote_path
            ))
            // Set the local mount point for the remote directory.
            .arg(self.local_mount_path());

        list.push(cmd_sshfs);

        Ok(list)
    }

    /// Returns a list of commands for unmounting the filesystem definition.
    ///
    /// Unmounting with this command may fail if the filesystem is busy and a fallback mechanism may be necessary
    /// (killing the `sshfs` process responsible for the mount).
    pub fn umount_commands(&self) -> Result<Vec<Command>, SftpManError> {
        log::debug!("{0}: building list of unmount commands", self.id);

        let mut list: Vec<Command> = Vec::new();

        // Unmounting is done via `fusermount -u`.
        // Using `nix::mount::umount` or `nix::mount::umount2` sounds like a good idea,
        // but those require special privileges (`CAP_SYS_ADMIN``) and return `EPERM` to regular users.

        let mut cmd = Command::new("fusermount");
        cmd.arg("-u").arg(self.local_mount_path());

        list.push(cmd);

        Ok(list)
    }

    /// Returns a command that opens a file manager (via `xdg-open`) at the local mount path (see `local_mount_path()`).
    ///
    /// Opening requires that the filesystem is already mounted.
    pub fn open_command(&self) -> Command {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(self.local_mount_path());

        cmd
    }
}

fn validate_id(id: &str) -> Result<(), ValidationError> {
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(ValidationError::new("invalid_id").with_message(
            "The ID must contain only alphanumeric characters, underscores, dashes, or dots."
                .into(),
        ));
    }
    Ok(())
}

fn validate_absolute_path(path: &str) -> Result<(), ValidationError> {
    if !path.starts_with('/') {
        return Err(ValidationError::new("not_absolute_path")
            .with_message(format!("The path {0} is not absolute.", path).into()));
    }

    Ok(())
}

fn validate_ssh_key_for_publickey_auth(
    entity: &&FilesystemMountDefinition,
) -> Result<(), ValidationError> {
    match entity.auth_type {
        AuthType::PublicKey => {
            if entity.ssh_key.is_empty() {
                Err(
                    ValidationError::new("no_ssh_key_for_publickey_auth").with_message(
                        format!(
                            "The {0} authentication type requires an SSH key to be provided.",
                            AuthType::PublicKey,
                        )
                        .into(),
                    ),
                )
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}
