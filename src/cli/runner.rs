use clap::ArgMatches;

use crate::manager::Manager;

use super::exit;
use super::preflight_check::preflight_check;

pub fn run(manager: &Manager, arg_matches: &ArgMatches) -> exit::Status {
    match arg_matches.subcommand() {
        Some(("ls", sub_matches)) => super::ls::run(manager, sub_matches),

        Some(("mount", sub_matches)) => super::mount::run(manager, sub_matches),
        Some(("mount_all", _sub_matches)) => super::mount::run_mount_all(manager),

        Some(("umount", sub_matches)) => super::umount::run(manager, sub_matches),
        Some(("umount_all", _sub_matches)) => super::umount::run_umount_all(manager),

        Some(("preflight_check", _sub_matches)) => preflight_check(manager),

        Some(("rm", sub_matches)) => super::remove::run(manager, sub_matches),

        Some(("create", sub_matches)) => super::create_update::run_create(manager, sub_matches),
        Some(("update", sub_matches)) => super::create_update::run_update(manager, sub_matches),

        Some((cmd, _)) => {
            log::error!(
                "Unknown subcommand {0}. Try removing it and running --help",
                cmd
            );
            exit::Status::UnknownCommand
        }

        None => {
            log::error!("Unknown subcommand. Try removing it and running --help");
            exit::Status::UnknownCommand
        }
    }
}
