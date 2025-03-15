use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use validator::Validate;
use validator::ValidationErrors;

use crate::AuthType;
use crate::Manager;
use crate::errors::SftpManError;
use crate::model::{DEFAULT_MOUNT_PATH_PREFIX, FilesystemMountDefinition};
use crate::utils::validation::errors_to_string_list;

use super::exit;

const ARG_ID: &str = "id";
const ARG_HOST: &str = "host";
const ARG_PORT: &str = "port";
const ARG_USER: &str = "user";
const ARG_REMOTE_PATH: &str = "remote_path";
const ARG_MOUNT_OPT: &str = "mount_opt";
const ARG_MOUNT_PATH: &str = "mount_path";
const ARG_AUTH_TYPE: &str = "auth_type";
const ARG_SSH_KEY: &str = "ssh_key";
const ARG_CMD_BEFORE_MOUNT: &str = "cmd_before_mount";

pub fn build_create() -> Command {
    Command::new("create")
        .about("Creates a new filesystem mount definition")
        .arg(
            Arg::new(ARG_ID)
                .long(ARG_ID)
                .num_args(1)
                .required(true)
                .help("Unique identifier. Example: my-machine")
        )
        .arg(
            Arg::new(ARG_HOST)
                .long(ARG_HOST)
                .num_args(1)
                .required(true)
                .help("Hostname or IP address")
        )
        .arg(
            Arg::new(ARG_PORT)
                .long(ARG_PORT)
                .num_args(1)
                .value_parser(value_parser!(u16).range(0..65535))
                .default_value("22")
                .required(false)
                .help("SSH port number")
        )
        .arg(
            Arg::new(ARG_USER)
                .long(ARG_USER)
                .num_args(1)
                .required(true)
                .help("Username to authenticate with")
        )
        .arg(
            Arg::new(ARG_MOUNT_OPT)
                .long(ARG_MOUNT_OPT)
                .num_args(1)
                .help("Options to pass to sshfs (via -o), separated by comma. Example: follow_symlinks,workaround=rename")
        )
        .arg(
            Arg::new(ARG_REMOTE_PATH)
                .long(ARG_REMOTE_PATH)
                .required(true)
                .help("Path on the remote machine that will be mounted locally. Example: /srv/http")
        )
        .arg(
            Arg::new(ARG_MOUNT_PATH)
                .long(ARG_MOUNT_PATH)
                .help(format!(
                    "Path on the current machine where the remote path would be mounted. Example: /home/user/Desktop/http. Default: {0}/my-machine",
                    DEFAULT_MOUNT_PATH_PREFIX
                ))
        )
        .arg(
            Arg::new(ARG_AUTH_TYPE)
                .long(ARG_AUTH_TYPE)
                .required(true)
                .value_parser(clap::builder::EnumValueParser::<AuthType>::new())
                .help("SSH authentication type")
        )
        .arg(
            Arg::new(ARG_SSH_KEY)
                .long(ARG_SSH_KEY)
                .required_if_eq(ARG_AUTH_TYPE, AuthType::PublicKey.to_static_str())
                .value_parser(clap::builder::PathBufValueParser::new())
                .help(format!(
                    "SSH private key path. Only applies when --auth_type={0}. Example: /home/user/.ssh/id_ed25519",
                    AuthType::PublicKey.to_static_str(),
                ))
        )
        .arg(
            Arg::new(ARG_CMD_BEFORE_MOUNT)
                .long(ARG_CMD_BEFORE_MOUNT)
                .required(false)
                .help("Custom command to run every time before mounting. Example: /bin/true")
        )
}

pub fn run_create(manager: &Manager, matches: &ArgMatches) -> exit::Status {
    let id = matches.get_one::<String>(ARG_ID).expect("required");

    match manager.definition(id) {
        Ok(_) => {
            log::error!("There already is a definition with an id of: {0}.", id);
            log::error!("Consider updating it or removing & creating it anew.");
            exit::Status::DefinitionAlreadyExists
        }

        Err(err) => match err {
            SftpManError::JSON(_path, _serde_error) => {
                log::error!(
                    "There already is a definition with an id of: {0}, but its data cannot be parsed",
                    id
                );
                exit::Status::Failure
            }

            // Any other error most likely indicates that the definition does not exist.
            // We should be safe to proceed with creation.
            _ => create(manager, id, matches),
        },
    }
}

fn create(manager: &Manager, id: &str, matches: &ArgMatches) -> exit::Status {
    let mut definition = FilesystemMountDefinition {
        id: id.to_owned(),
        ..Default::default()
    };

    bind_command_arguments_to_definition(matches, &mut definition, true);

    if let Err(errors) = definition.validate() {
        return abort_with_validation_errors(errors);
    }

    if let Err(err) = manager.persist(&definition) {
        log::error!("Failed to persist definition: {0}", err);
        return exit::Status::Failure;
    }

    exit::Status::Success
}

/// Creates the update subcommand based on the create subcommand, with only the id argument being required
pub fn build_update() -> Command {
    let mut cmd = Command::new("update").about("Updates an existing filesystem mount definition");

    for arg_ref in build_create().get_arguments() {
        let mut arg = arg_ref.to_owned();

        if *arg.get_id() != ARG_ID {
            arg = arg.required(false);
        }

        if *arg.get_id() == ARG_SSH_KEY {
            // This one has `required_if_eq` tied to ARG_AUTH_TYPE.
            // We can't clear it on an existing instance, so we'll create a new one.
            let value_parser = arg.get_value_parser();

            let mut arg_ssh_key = Arg::new(arg.get_id())
                .long(ARG_SSH_KEY)
                .value_parser(value_parser.to_owned());

            if let Some(help) = arg.get_help() {
                arg_ssh_key = arg_ssh_key.help(help.to_string());
            }

            arg = arg_ssh_key
        }

        cmd = cmd.arg(arg);
    }

    cmd
}

pub fn run_update(manager: &Manager, matches: &ArgMatches) -> exit::Status {
    let id = matches.get_one::<String>(ARG_ID).expect("required");

    match manager.definition(id) {
        Ok(mut definition) => update(manager, &mut definition, matches),

        Err(err) => {
            log::error!("Failed to find or load definition: {0}: {1}", id, err);
            exit::Status::DefinitionNotFound
        }
    }
}

fn update(
    manager: &Manager,
    definition: &mut FilesystemMountDefinition,
    matches: &ArgMatches,
) -> exit::Status {
    bind_command_arguments_to_definition(matches, definition, false);

    if let Err(errors) = definition.validate() {
        return abort_with_validation_errors(errors);
    }

    if let Err(err) = manager.persist(definition) {
        log::error!("Failed to persist definition: {0}", err);
        return exit::Status::Failure;
    }

    exit::Status::Success
}

/// Binds the provided arguments to the definition.
///
/// If arguments are missing, it intentionally doesn't complain,
/// as this function is meant to be generic enough so that both the create and update subcommands could use it.
/// Each subcommand defines its own requirements anyway.
/// Additionally, it's expected that the final result would get validated somewhere upstream.
fn bind_command_arguments_to_definition(
    matches: &ArgMatches,
    definition: &mut FilesystemMountDefinition,
    is_new: bool,
) {
    if let Some(value) = matches.get_one::<String>(ARG_HOST) {
        definition.host = value.clone().to_owned();
    }

    if let Some(value) = matches.get_one::<u16>(ARG_PORT) {
        definition.port = *value;
    }

    if let Some(value) = matches.get_one::<String>(ARG_USER) {
        definition.user = value.clone().to_owned();
    }

    if let Some(value) = matches.get_one::<String>(ARG_MOUNT_OPT) {
        definition.mount_options.clear();

        for v in value.split(',') {
            if !v.is_empty() {
                definition.mount_options.push(v.to_owned());
            }
        }
    }

    if let Some(value) = matches.get_one::<String>(ARG_REMOTE_PATH) {
        definition.remote_path = value.clone().to_owned();
    }

    if let Some(value) = matches.get_one::<String>(ARG_MOUNT_PATH) {
        if value.is_empty() {
            definition.mount_dest_path = None;
        } else {
            definition.mount_dest_path = Some(value.clone().to_owned());
        }
    }

    if let Some(value) = matches.get_one::<String>(ARG_CMD_BEFORE_MOUNT) {
        definition.cmd_before_mount = value.clone().to_owned();
    }

    if let Some(value) = matches.get_one::<AuthType>(ARG_AUTH_TYPE) {
        definition.auth_type = value.clone().to_owned();
    }
    // When binding to existing records, make changing the auth type to one that doesn't use SSH keys also unset the SSH key.
    if !is_new && definition.auth_type != AuthType::PublicKey {
        definition.ssh_key = "".to_owned();
    }

    if let Some(value) = matches.get_one::<PathBuf>(ARG_SSH_KEY) {
        definition.ssh_key = value.to_string_lossy().into();
    }
}

fn abort_with_validation_errors(errors: ValidationErrors) -> exit::Status {
    log::error!("Validation failed with the following errors:");

    for err in errors_to_string_list(errors) {
        log::error!("- {0}", err);
    }

    exit::Status::ValidationFailure
}
