use clap::Command;

use crate::manager::Manager;

use super::exit;

pub fn build() -> Command {
    Command::new("preflight_check")
        .about("Detects whether we have everything needed to mount SFTP systems")
}

pub fn preflight_check(manager: &Manager) -> exit::Status {
    match manager.preflight_check() {
        Ok(()) => {
            log::info!("All checks pass! You can use sftpman");
            exit::Status::Success
        }
        Err(errs) => {
            for err in errs {
                log::error!("Preflight-check failure: {0:?}", err);
            }
            exit::Status::Failure
        }
    }
}
