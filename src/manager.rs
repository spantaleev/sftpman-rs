use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::model::DEFAULT_MOUNT_PATH_PREFIX;

use super::errors::{ManagerInitError, PreflightCheckError, SftpManError};
use super::model::{FilesystemMountDefinition, MountState};

use super::utils::command::{run_command, run_command_background};
use super::utils::fs::{
    ensure_directory_recursively_created, get_mounts_under_path_prefix, remove_empty_directory,
};
use super::utils::process::{ensure_process_killed, sshfs_pid_by_definition};

const VFS_TYPE_SSHFS: &str = "fuse.sshfs";

#[derive(Default, Clone)]
pub struct Manager {
    config_path: PathBuf,
}

impl Manager {
    pub fn new() -> Result<Self, ManagerInitError> {
        let d = directories::ProjectDirs::from("sftpman", "Devture Ltd", "sftpman")
            .ok_or(ManagerInitError::NoConfigDirectory)?;

        Ok(Self {
            config_path: d.config_dir().to_path_buf().to_owned(),
        })
    }

    /// Returns the list of all known (stored in the config directory) filesystem definitions.
    pub fn definitions(&self) -> Result<Vec<FilesystemMountDefinition>, SftpManError> {
        let dir_path = self.config_path_mounts();

        if !dir_path.is_dir() {
            log::debug!(
                "Mount config directory {0} doesn't exist. Returning an empty definitions list ...",
                dir_path.display()
            );
            return Ok(vec![]);
        }

        let mut list: Vec<FilesystemMountDefinition> = Vec::new();

        let directory_entries =
            fs::read_dir(dir_path).map_err(|err| SftpManError::Generic(err.to_string()))?;

        for entry in directory_entries {
            let entry = entry.map_err(|err| SftpManError::Generic(err.to_string()))?;

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let name = path.file_name();
            if name.is_none() {
                continue;
            }
            if !name.unwrap().to_string_lossy().ends_with(".json") {
                continue;
            }

            match Self::definition_from_config_path(&path) {
                Ok(cfg) => list.push(cfg),
                Err(err) => return Err(err),
            }
        }

        list.sort_by_key(|item| item.id.clone());

        Ok(list)
    }

    /// Returns the filesystem definition (as stored in the config directory) for the given ID.
    pub fn definition(&self, id: &str) -> Result<FilesystemMountDefinition, SftpManError> {
        Self::definition_from_config_path(&self.config_path_for_definition_id(id))
    }

    /// Returns the full state (configuration and mount status) of all known (stored in the config directory) filesystem definitions.
    pub fn full_state(&self) -> Result<Vec<MountState>, SftpManError> {
        let mut mounted_sshfs_paths_map: HashMap<String, bool> = HashMap::new();

        for mount in get_mounts_under_path_prefix("/")? {
            if mount.vfstype != VFS_TYPE_SSHFS {
                continue;
            }

            mounted_sshfs_paths_map
                .insert(mount.file.as_os_str().to_str().unwrap().to_owned(), true);
        }

        let mut list: Vec<MountState> = Vec::new();

        for definition in self.definitions()? {
            let mounted = mounted_sshfs_paths_map.contains_key(&definition.local_mount_path());
            list.push(MountState::new(definition, mounted));
        }

        Ok(list)
    }

    /// Tells if the given filesystem definition is currently mounted.
    pub fn is_definition_mounted(
        &self,
        definition: &FilesystemMountDefinition,
    ) -> Result<bool, SftpManError> {
        let local_mount_path = definition.local_mount_path();

        for mount in get_mounts_under_path_prefix(local_mount_path.as_str())? {
            if *mount.file.as_os_str().to_str().unwrap() != local_mount_path {
                continue;
            }

            if mount.vfstype != VFS_TYPE_SSHFS {
                return Err(SftpManError::MountVfsTypeMismatch {
                    path: std::path::Path::new(&local_mount_path).to_path_buf(),
                    found_vfs_type: mount.vfstype.to_string(),
                    expected_vfs_type: VFS_TYPE_SSHFS.to_string(),
                });
            }

            return Ok(true);
        }

        Ok(false)
    }

    /// Mounts a filesystem definition unless already mounted.
    pub fn mount(&self, definition: &FilesystemMountDefinition) -> Result<(), SftpManError> {
        if self.is_definition_mounted(definition)? {
            log::info!("{0}: already mounted, nothing to do..", definition.id);
            return Ok(());
        }

        log::info!("{0}: mounting..", definition.id);

        ensure_directory_recursively_created(&definition.local_mount_path())?;

        let cmds = definition.mount_commands().unwrap();

        for cmd in cmds {
            log::debug!("{0}: executing mount command: {1:?}", definition.id, cmd);

            if let Err(err) = run_command(cmd) {
                log::error!(
                    "{0}: failed to run mount command: {1:?}",
                    definition.id,
                    err
                );

                log::debug!("{0}: performing umount to clean up", definition.id);

                // This will most likely fail, but we should try to do it anyway.
                if let Err(err) = self.umount(definition) {
                    log::debug!(
                        "{0}: failed to perform cleanup-umount: {1:?}",
                        definition.id,
                        err
                    );
                }

                self.clean_up_after_unmount(definition);

                return Err(err);
            }
        }

        Ok(())
    }

    /// Unmounts a filesystem definition (unless already unmounted) and removes its mount path from the filesystem hierarchy.
    ///
    /// Unmounting is performed via a command call to `fusermount3 -u ..` (preferred) or `fusermount -u ..` (fallback),
    /// which may fail on filesystems that are currently busy.
    /// In such cases, a fallback is performed - the `sshfs` process responsible for the mount gets terminated.
    pub fn umount(&self, definition: &FilesystemMountDefinition) -> Result<(), SftpManError> {
        if !self.is_definition_mounted(definition)? {
            log::info!("{0}: not mounted, nothing to do..", definition.id);
            return Ok(());
        }

        log::info!("{0}: unmounting..", definition.id);

        match self.do_umount(definition) {
            Ok(_) => Ok(()),

            Err(err) => {
                // It's likely that this is a "Device is busy" error.

                log::warn!("{0} failed to get unmounted: {1:?}", definition.id, err);

                self.kill_sshfs_for_definition(definition)?;

                // Killing successfully is good enough to unmount.
                // We don't need to call do_umount() again, as calling `fusermount -u ..` (etc), may fail with:
                // > CommandUnsuccessfulError("fusermount" "-u" "/home/user/mounts/storage", Output { status: ExitStatus(unix_wait_status(256)), stdout: "", stderr: "fusermount: entry for /path not found in /etc/mtab\n" })
                // We only need to clean up now.

                self.clean_up_after_unmount(definition);

                Ok(())
            }
        }
    }

    fn do_umount(&self, definition: &FilesystemMountDefinition) -> Result<(), SftpManError> {
        let cmds = definition.umount_commands().unwrap();

        for cmd in cmds {
            log::debug!("{0}: executing unmount command: {1:?}", definition.id, cmd);

            if let Err(err) = run_command(cmd) {
                log::error!(
                    "{0}: failed to run unmount command: {1:?}",
                    definition.id,
                    err
                );

                // We weren't successful to unmount, but it may be because the mount point already got unmounted.
                // It doesn't hurt to try and clean up.
                self.clean_up_after_unmount(definition);

                return Err(err);
            }
        }

        self.clean_up_after_unmount(definition);

        Ok(())
    }

    /// Unmounts the given filesystem (if mounted) and removes the configuration file for it.
    pub fn remove(&self, definition: &FilesystemMountDefinition) -> Result<(), SftpManError> {
        log::info!("{0}: removing..", definition.id);

        self.umount(definition)?;

        let definition_config_path = self.config_path_for_definition_id(&definition.id);

        log::debug!(
            "{0}: deleting file {1}",
            definition.id,
            definition_config_path.display()
        );

        fs::remove_file(&definition_config_path).map_err(|err| {
            SftpManError::FilesystemMountDefinitionRemove(definition_config_path, err)
        })?;

        Ok(())
    }

    /// Checks if we have everything needed to mount/unmount sshfs/SFTP filesystems.
    pub fn preflight_check(&self) -> Result<(), Vec<PreflightCheckError>> {
        let mut cmd_alternative_groups: Vec<Vec<Command>> = Vec::new();

        let mut cmd_sshfs = Command::new("sshfs");
        cmd_sshfs.arg("-h");
        cmd_alternative_groups.push(vec![cmd_sshfs]);

        let mut cmd_ssh = Command::new("ssh");
        cmd_ssh.arg("-V");
        cmd_alternative_groups.push(vec![cmd_ssh]);

        // We favor `fusermount3`, but will also make do with `fusermount` if `fusermount3` is not available.
        // See: https://github.com/spantaleev/sftpman-rs/issues/3
        let mut cmd_fusermount3 = Command::new("fusermount3");
        cmd_fusermount3.arg("-V");
        let mut cmd_fusermount = Command::new("fusermount");
        cmd_fusermount.arg("-V");
        cmd_alternative_groups.push(vec![cmd_fusermount3, cmd_fusermount]);

        let mut errors: Vec<PreflightCheckError> = Vec::new();

        for cmd_group in cmd_alternative_groups {
            let mut cmd_group_successful = false;
            let mut cmd_group_errors: Vec<PreflightCheckError> = Vec::new();

            for cmd in cmd_group {
                log::debug!("Executing preflight-check command: {0:?}", cmd);

                if let Err(err) = run_command(cmd) {
                    log::warn!("Failed to run preflight-check command: {0:?}", err);

                    let preflight_check_error = match err {
                        SftpManError::CommandExecution(cmd, err) => {
                            Some(PreflightCheckError::CommandExecution(cmd, err))
                        }
                        SftpManError::CommandUnsuccessful(cmd, output) => {
                            Some(PreflightCheckError::CommandUnsuccessful(cmd, output))
                        }
                        _ => {
                            // This should never happen since run_command() only returns these two error variants
                            log::error!("Unexpected error type: {0:?}", err);
                            None
                        }
                    };

                    if let Some(preflight_check_error) = preflight_check_error {
                        cmd_group_errors.push(preflight_check_error);
                    }
                } else {
                    log::debug!("Preflight-check command succeeded");
                    cmd_group_successful = true;
                    break;
                }
            }

            if !cmd_group_successful {
                errors.extend(cmd_group_errors);
            }
        }

        let default_mount_path = PathBuf::from(DEFAULT_MOUNT_PATH_PREFIX);
        let mut default_mount_path_ok = false;
        let random_test_path = default_mount_path.join(format!(
            "_{}_test_{}",
            env!("CARGO_PKG_NAME"),
            rand::random::<u32>()
        ));

        if default_mount_path.exists() {
            log::debug!(
                "Default mount path {} already exists",
                DEFAULT_MOUNT_PATH_PREFIX
            );
            default_mount_path_ok = true;
        } else {
            log::warn!(
                "Default mount path {} does not exist, attempting to create it",
                DEFAULT_MOUNT_PATH_PREFIX
            );

            if let Err(err) = fs::create_dir_all(&default_mount_path) {
                log::error!(
                    "Failed to create mount path {}: {}",
                    DEFAULT_MOUNT_PATH_PREFIX,
                    err
                );

                errors.push(PreflightCheckError::DefaultBasePathIO(
                    default_mount_path,
                    err,
                ));
            } else {
                default_mount_path_ok = true;
            }
        }

        if default_mount_path_ok {
            log::debug!(
                "Testing if we can create and remove directory: {}",
                random_test_path.display()
            );

            if let Err(err) = fs::create_dir_all(&random_test_path) {
                log::error!(
                    "Failed to create test directory {}: {}",
                    random_test_path.display(),
                    err
                );
                errors.push(PreflightCheckError::TestUnderBasePathIO(
                    random_test_path,
                    err,
                ));
            } else if let Err(err) = fs::remove_dir(&random_test_path) {
                log::error!(
                    "Failed to remove test directory {}: {}",
                    random_test_path.display(),
                    err
                );
                errors.push(PreflightCheckError::TestUnderBasePathIO(
                    random_test_path,
                    err,
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Persists (creates or updates) a filesystem definition.
    ///
    /// If the definition already exists, it will be unmounted before persisting and will be remounted after.
    pub fn persist(&self, definition: &FilesystemMountDefinition) -> Result<(), SftpManError> {
        let mut is_existing_and_mounted = false;
        if let Ok(old) = self.definition(&definition.id) {
            is_existing_and_mounted = self.is_definition_mounted(&old)?;

            if is_existing_and_mounted {
                log::debug!(
                    "{0} was found to be an existing and currently mounted definition. Unmounting..",
                    definition.id
                );

                if let Err(err) = self.umount(&old) {
                    log::error!("{0} failed to be unmounted: {1:?}", definition.id, err);
                }
            }
        }

        let path = self.config_path_for_definition_id(&definition.id);

        let config_dir_path = path
            .parent()
            .expect("Config directory path should have a parent");

        if !config_dir_path.exists() {
            log::info!(
                "Config directory {} does not exist, attempting to create it",
                config_dir_path.display()
            );

            if let Err(err) = fs::create_dir_all(config_dir_path) {
                log::error!(
                    "Failed to create config directory {}: {}",
                    config_dir_path.display(),
                    err
                );
                return Err(SftpManError::IO(path.clone(), err));
            }
        }

        let serialized = definition
            .to_json_string()
            .map_err(|err| SftpManError::JSON(path.clone(), err))?;

        fs::write(&path, serialized).map_err(|err| SftpManError::IO(path.clone(), err))?;

        if is_existing_and_mounted {
            log::debug!(
                "{0} is being mounted, because it was before updating..",
                definition.id
            );

            if let Err(err) = self.mount(definition) {
                log::error!(
                    "{0} failed get re-mounted afte rupdating: {1:?}",
                    definition.id,
                    err
                );
            }
        }

        Ok(())
    }

    /// Opens the directory where the given filesystem definition is mounted.
    pub fn open(&self, definition: &FilesystemMountDefinition) -> Result<(), SftpManError> {
        if let Err(err) = run_command_background(definition.open_command()) {
            log::error!("{0}: failed to run open command: {1:?}", definition.id, err);
        }
        Ok(())
    }

    fn kill_sshfs_for_definition(
        &self,
        definition: &FilesystemMountDefinition,
    ) -> Result<(), SftpManError> {
        log::debug!(
            "Trying to determine the sshfs process for {0}",
            definition.id
        );

        let pid = sshfs_pid_by_definition(definition)?;

        match pid {
            Some(pid) => {
                log::debug!(
                    "Process id for {0} determined to be: {1}. Killing..",
                    definition.id,
                    pid
                );

                ensure_process_killed(pid, Duration::from_millis(500), Duration::from_millis(2000))
            }

            None => Err(SftpManError::Generic(format!(
                "Failed to determine pid for: {0}",
                definition.id
            ))),
        }
    }

    fn clean_up_after_unmount(&self, definition: &FilesystemMountDefinition) {
        log::debug!("{0}: cleaning up after unmounting", definition.id);

        if let Err(err) = remove_empty_directory(&definition.local_mount_path()) {
            log::debug!(
                "{0}: failed to remove local mount point: {1:?}",
                definition.id,
                err
            );
        }
    }

    fn config_path_mounts(&self) -> PathBuf {
        self.config_path.join("mounts")
    }

    fn config_path_for_definition_id(&self, id: &str) -> PathBuf {
        self.config_path_mounts().join(format!("{0}.json", id))
    }

    fn definition_from_config_path(
        path: &PathBuf,
    ) -> Result<FilesystemMountDefinition, SftpManError> {
        let contents = fs::read_to_string(path)
            .map_err(|err| SftpManError::FilesystemMountDefinitionRead(path.clone(), err))?;

        let mount_config_result = FilesystemMountDefinition::from_json_string(&contents);

        match mount_config_result {
            Ok(cfg) => Ok(cfg),
            Err(err) => Err(SftpManError::JSON(path.clone(), err)),
        }
    }
}
