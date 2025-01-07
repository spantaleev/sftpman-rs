use std::process::{Command, Output};

use crate::errors::SftpManError;

pub fn command_to_string(cmd: &Command) -> String {
    let mut combined: Vec<String> = Vec::new();

    combined.push(cmd.get_program().to_str().unwrap().to_owned());

    for arg in cmd.get_args() {
        combined.push(arg.to_str().unwrap().to_owned());
    }

    combined.join(" ")
}

pub fn run_command(mut cmd: Command) -> Result<Output, SftpManError> {
    match cmd.output() {
        Err(err) => Err(SftpManError::CommandExecution(cmd, err)),

        Ok(output) => {
            // A non-error output result does not meant success.
            // We may still get:
            // > Ok(Output { status: ExitStatus(unix_wait_status(256)), stdout: "", stderr: "sshfs: bad ..." })

            if output.status.success() {
                Ok(output)
            } else {
                Err(SftpManError::CommandUnsuccessful(cmd, output))
            }
        }
    }
}

pub fn run_command_background(mut cmd: Command) -> Result<(), SftpManError> {
    match cmd.spawn() {
        Err(err) => Err(SftpManError::CommandExecution(cmd, err)),
        Ok(_) => Ok(()),
    }
}
