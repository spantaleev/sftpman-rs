use std::process;

#[cfg(feature = "cli")]
use libsftpman::cli;

use libsftpman::Manager;

#[cfg(feature = "cli")]
fn main() {
    let arg_matches: clap::ArgMatches = cli::build().get_matches();

    let log_level = match arg_matches.get_count("verbose") {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let mut builder = env_logger::Builder::new();
    builder.filter_level(log_level);
    builder.init();

    let manager = Manager::new().unwrap();

    process::exit(cli::run(&manager, &arg_matches).into());
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("This is a library, not an executable.");
}
