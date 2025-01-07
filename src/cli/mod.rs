use clap::{Arg, ArgAction, Command};

mod create_update;
mod exit;
mod ls;
mod mount;
mod preflight_check;
mod remove;
mod runner;
mod umount;

pub use exit::Status as ExitStatus;
pub use runner::run;

pub fn build() -> Command {
    Command::new("sftpman")
        .about("sftpman is an application for managing and mounting sshfs (SFTP) filesystems")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
    .arg(
        Arg::new("verbose")
            .short('v')
            .global(true)
            .long("verbose")
            .action(ArgAction::Count)
            .help("Control logging verbosity (none for warn; -v for info; -vv for debug; -vvv for trace)")
    )
    .subcommand(ls::build())
    .subcommand(mount::build())
    .subcommand(mount::build_mount_all())
    .subcommand(umount::build())
    .subcommand(umount::build_umount_all())
    .subcommand(preflight_check::build())
    .subcommand(remove::build())
    .subcommand(create_update::build_create())
    .subcommand(create_update::build_update())
}
