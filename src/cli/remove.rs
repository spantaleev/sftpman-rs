use clap::{Arg, ArgMatches, Command};

use crate::{manager::Manager, model::FilesystemMountDefinition};

use super::exit;

pub fn build() -> Command {
    Command::new("rm")
        .about("Removes the specified system or systems")
        .arg(Arg::new("id").num_args(1..).required(true))
}

pub fn run(manager: &Manager, matches: &ArgMatches) -> exit::Status {
    let ids: Vec<&str> = matches
        .get_many::<String>("id")
        .expect("required")
        .map(|s| s.as_str())
        .collect();

    remove(manager, &ids)
}

/// Removes the given filesystems by id.
/// Returns exit::Status::Success if all removing succeeded.
/// Returns exit::Status::DefinitionNotFound if at least one filesystem was not found.
/// Returns exit::Status::Failure if at least one filesystem failed to be removed.
pub fn remove(manager: &Manager, ids: &Vec<&str>) -> exit::Status {
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

    if !remove_definitions(manager, &definitions_to_work_on) {
        exit_status = exit::Status::Failure
    }

    exit_status
}

/// Removes the given filesystems.
fn remove_definitions(manager: &Manager, definitions: &Vec<&FilesystemMountDefinition>) -> bool {
    let mut all_good = true;

    for definition in definitions {
        if let Err(err) = manager.remove(definition) {
            log::error!("Failure removing {0}: {1:?}", definition.id, err);
            all_good = false
        }
    }

    all_good
}
