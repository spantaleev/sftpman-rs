use std::thread;
use std::time::Duration;

use nix::sys::signal::kill;
use nix::sys::signal::Signal;
use nix::unistd::Pid;

use procfs::process::all_processes as get_all_processes;
use procfs::process::Process;
use procfs::ProcError;

use crate::errors::SftpManError;
use crate::FilesystemMountDefinition;

pub fn sshfs_pid_by_definition(
    definition: &FilesystemMountDefinition,
) -> Result<Option<i32>, SftpManError> {
    let processes = get_all_processes()
        .map_err(|err| SftpManError::Generic(format!("failed to list processes: {0}", err)))?;

    for process in processes.flatten() {
        if let Ok(cmd_line) = process.cmdline() {
            let program = if cmd_line.len() > 1 {
                cmd_line[0].clone()
            } else {
                "".to_owned()
            };

            if program != "sshfs" {
                continue;
            }

            let expected_arg = format!(
                "{0}@[{1}]:{2}",
                definition.user, definition.host, definition.remote_path
            );

            for arg in cmd_line {
                if arg == expected_arg {
                    return Ok(Some(process.pid));
                }
            }
        }
    }

    Ok(None)
}

pub fn ensure_process_killed(
    pid: i32,
    wait_time_before_dead_check: Duration,
    wait_time_before_forcefully_killing: Duration,
) -> Result<(), SftpManError> {
    if let Err(err) = kill_pid_gracefully(pid) {
        log::debug!(
            "Process {0} failed to be killed gracefully: {1:?}",
            pid,
            err
        );
    }

    log::debug!(
        "Sleeping for {0:?} before checking if the process was killed..",
        wait_time_before_dead_check,
    );

    thread::sleep(wait_time_before_dead_check);

    match is_pid_alive(pid) {
        Err(err) => {
            log::debug!(
                "Failed to check if process {0} is still alive after killing it: {1:?}",
                pid,
                err
            );

            // It may or may not be alive, so..
            // Fall through to continue killing forcefully.
        }
        Ok(alive) => {
            if !alive {
                log::debug!("Process {0} was not found to be alive anymore..", pid,);

                return Ok(());
            }

            // Still alive, so we need to do more..
            // Fall through to continue killing forcefully.
            log::debug!(
                "Process {0} was found to be alive after graceful killing..",
                pid,
            );
        }
    }

    log::debug!(
        "Sleeping for {0:?} before killing forcefully..",
        wait_time_before_forcefully_killing,
    );

    thread::sleep(wait_time_before_dead_check);

    if let Err(err) = kill_pid_forcefully(pid) {
        log::debug!(
            "Process {0} failed to be killed forcefully: {1:?}",
            pid,
            err
        );
    }

    match is_pid_alive(pid) {
        Err(err) => {
            log::debug!(
                "Failed to check if process {0} is still alive after killing it: {1:?}",
                pid,
                err
            );

            // It may or may not be alive, so..
            // Fall through to continue killing forcefully.
        }
        Ok(alive) => {
            if !alive {
                return Ok(());
            }

            return Err(SftpManError::Generic(
                "Ultimately failed to kill process".to_owned(),
            ));
        }
    }

    Ok(())
}

fn kill_pid_gracefully(pid: i32) -> Result<(), SftpManError> {
    kill_pid_with_signal(pid, Signal::SIGTERM)
}

fn kill_pid_forcefully(pid: i32) -> Result<(), SftpManError> {
    kill_pid_with_signal(pid, Signal::SIGKILL)
}

fn kill_pid_with_signal(pid: i32, signal: Signal) -> Result<(), SftpManError> {
    let pid = Pid::from_raw(pid);

    match kill(pid, signal) {
        Ok(_) => Ok(()),
        Err(e) => Err(SftpManError::Generic(format!(
            "Failed to send signal {0} to {1}: {2}",
            signal, pid, e,
        ))),
    }
}

fn is_pid_alive(pid: i32) -> Result<bool, ProcError> {
    let p = Process::new(pid);
    match p {
        Ok(_) => Ok(true),
        Err(err) => match err {
            ProcError::NotFound(_) => Ok(false),
            _ => Err(err),
        },
    }
}
