use clap::{Arg, ArgMatches, Command};

use crate::{manager::Manager, model::FilesystemMountDefinition};

use super::exit;

pub fn build() -> Command {
    Command::new("umount")
        .about("Unmounts the specified SFTP system or systems, unless already unmounted")
        .arg(Arg::new("id").num_args(1..).required(true))
}

pub fn run(manager: &Manager, matches: &ArgMatches) -> exit::Status {
    let ids: Vec<&str> = matches
        .get_many::<String>("id")
        .expect("required")
        .map(|s| s.as_str())
        .collect();

    umount(manager, &ids)
}

pub fn build_umount_all() -> Command {
    Command::new("umount_all").about("Unmounts all known SFTP systems")
}

pub fn run_umount_all(manager: &Manager) -> exit::Status {
    umount_all(manager)
}

/// Unmounts the given filesystems by id.
/// Returns exit::Status::Success if all unmounting succeeded.
/// Returns exit::Status::DefinitionNotFound if at least one filesystem was not found.
/// Returns exit::Status::Failure if at least one filesystem failed to unmount.
pub fn umount(manager: &Manager, ids: &Vec<&str>) -> exit::Status {
    let definitions = manager.definitions().unwrap();

    let mut exit_status = exit::Status::Success;

    let mut definitions_to_work_on: Vec<&FilesystemMountDefinition> = Vec::new();

    for id in ids {
        let definition_or_none = definitions.iter().find(|&x| &x.id == id);

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

    if !umount_definitions(manager, &definitions_to_work_on) {
        exit_status = exit::Status::Failure
    }

    exit_status
}

/// Unmounts all known filesystems which are currently mounted.
/// Returns exit::Status::Success if all unmounting succeeded.
/// Returns exit::Status::Failure if at least one filesystem failed to unmount.
pub fn umount_all(manager: &Manager) -> exit::Status {
    let definitions_to_work_on: Vec<FilesystemMountDefinition> = manager
        .full_state()
        .unwrap()
        .into_iter()
        .filter(|state| state.mounted)
        .map(|state| state.definition)
        .collect();

    if umount_definitions(manager, &definitions_to_work_on.iter().collect()) {
        exit::Status::Success
    } else {
        exit::Status::Failure
    }
}

/// Unmounts the given filesystems.
fn umount_definitions(manager: &Manager, definitions: &Vec<&FilesystemMountDefinition>) -> bool {
    let mut all_good = true;

    for definition in definitions {
        if let Err(err) = manager.umount(definition) {
            log::error!("Failure unmounting {0}: {1:?}", definition.id, err);
            all_good = false
        }
    }

    all_good
}
