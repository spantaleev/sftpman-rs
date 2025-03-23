use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use once_cell::sync::Lazy;

const FUSERMOUNT3_COMMAND: &str = "fusermount3";
const FUSERMOUNT_COMMAND: &str = "fusermount";

static FUSERMOUNT3_AVAILABLE: Lazy<AtomicBool> =
    Lazy::new(|| AtomicBool::new(is_fusermount3_available()));

fn is_fusermount3_available() -> bool {
    create_check_command(FUSERMOUNT3_COMMAND).output().is_ok()
}

fn create_check_command(program_name: &str) -> Command {
    let mut cmd = Command::new(program_name);
    cmd.arg("-V");
    cmd
}

pub fn create_fusermount3_check_command() -> Command {
    create_check_command(FUSERMOUNT3_COMMAND)
}

pub fn create_fusermount_check_command() -> Command {
    create_check_command(FUSERMOUNT_COMMAND)
}

// Determines the fusermount command to use.
// We favor `fusermount3`, but will also make do with `fusermount` if `fusermount3` is not available.
// See: https://github.com/spantaleev/sftpman-rs/issues/3
pub fn get_fusermount_command() -> &'static str {
    if FUSERMOUNT3_AVAILABLE.load(Ordering::Relaxed) {
        FUSERMOUNT3_COMMAND
    } else {
        FUSERMOUNT_COMMAND
    }
}
