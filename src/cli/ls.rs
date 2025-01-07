use clap::{Arg, ArgMatches, Command};

use crate::manager::Manager;

use super::exit;

pub fn build() -> Command {
    Command::new("ls")
        .about("Lists the available/mounted/unmounted SFTP systems.")
        .arg(
            Arg::new("what")
                .default_value("available")
                .help("Specifies what to operate on")
                .value_parser(["available", "mounted", "unmounted"]),
        )
}

pub fn run(manager: &Manager, matches: &ArgMatches) -> exit::Status {
    let what = matches.get_one::<String>("what").expect("required");
    do_ls(manager, what)
}

pub fn do_ls(manager: &Manager, what: &str) -> exit::Status {
    match what {
        "available" => {
            for definition in manager.definitions().unwrap() {
                println!("{0}", definition.id)
            }
        }

        "mounted" => {
            for state in manager.full_state().unwrap() {
                if !state.mounted {
                    continue;
                }

                println!("{0}", state.definition.id)
            }
        }

        "unmounted" => {
            for state in manager.full_state().unwrap() {
                if state.mounted {
                    continue;
                }

                println!("{0}", state.definition.id)
            }
        }

        _ => unreachable!(),
    }

    exit::Status::Success
}
