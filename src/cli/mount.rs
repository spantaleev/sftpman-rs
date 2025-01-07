use clap::{Arg, ArgMatches, Command};

use crate::{manager::Manager, model::FilesystemMountDefinition};

use super::exit;

pub fn build() -> Command {
    Command::new("mount")
        .about("Mounts the specified SFTP system or systems, unless already mounted")
        .arg(Arg::new("id").num_args(1..).required(true))
}

pub fn run(manager: &Manager, matches: &ArgMatches) -> exit::Status {
    let ids: Vec<&str> = matches
        .get_many::<String>("id")
        .expect("required")
        .map(|s| s.as_str())
        .collect();

    mount(manager, ids)
}

pub fn run_mount_all(manager: &Manager) -> exit::Status {
    mount_all(manager)
}

pub fn build_mount_all() -> Command {
    Command::new("mount_all").about("Mounts all known SFTP systems")
}

/// Mounts the given filesystems by id.
/// Returns exit::Status::Success if all mounting succeeded.
/// Returns exit::Status::DefinitionNotFound if at least one filesystem was not found.
/// Returns exit::Status::Failure if at least one filesystem failed to mount.
pub fn mount(manager: &Manager, ids: Vec<&str>) -> exit::Status {
    let definitions = manager.definitions().unwrap();

    let mut exit_status = exit::Status::Success;

    let mut definitions_to_work_on: Vec<&FilesystemMountDefinition> = Vec::new();

    for id in ids {
        let definition_or_none = definitions.iter().find(|&x| x.id == id);

        match definition_or_none {
            None => {
                log::error!("Failed to find filesystem with an id of: {0}", id);
                exit_status = exit::Status::DefinitionNotFound;
            }

            Some(definition) => {
                definitions_to_work_on.push(definition);
            }
        };
    }

    if !mount_definitions(manager, &definitions_to_work_on) {
        exit_status = exit::Status::Failure
    }

    exit_status
}

/// Mounts all known filesystems.
/// Returns exit::Status::Success if all mounting succeeded.
/// Returns exit::Status::Failure if at least one filesystem failed to mount.
pub fn mount_all(manager: &Manager) -> exit::Status {
    if mount_definitions(manager, &manager.definitions().unwrap().iter().collect()) {
        exit::Status::Success
    } else {
        exit::Status::Failure
    }
}

/// Mounts the given filesystems.
fn mount_definitions(manager: &Manager, definitions: &Vec<&FilesystemMountDefinition>) -> bool {
    let mut all_good = true;

    for definition in definitions {
        if let Err(err) = manager.mount(definition) {
            log::error!("Failure mounting {0}: {1:?}", definition.id, err);
            all_good = false
        }
    }

    all_good
}
