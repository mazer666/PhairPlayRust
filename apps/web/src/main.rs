//! The binary of the receiver.
//!
//! WHAT: The program that starts the receiver of PhairPlayRust. It reads
//! the configuration, registers the engines, starts every protocol that the
//! configuration enables and serves the dashboard.
//!
//! WHY: The user needs one program, not two: the binary serves the API and
//! the files of the dashboard. It is also the program that runs on a device
//! without a screen — on a Raspberry Pi there is nobody who clicks a switch
//! in a browser, so the protocols are started by the program itself.
//!
//! WHY the order of the start is the order of the product: the
//! configuration is read first, then the engines are registered, then the
//! protocols are started, and only then the panel is served. A protocol
//! that cannot start is reported with its reason and does not stop the
//! others — the receiver runs with the protocols that work.
//!
//! EXAMPLE:
//!
//! ```text
//! phairplay-web --profile raspberry-pi --port 7878
//! ```

use std::process::ExitCode;

use phairplay_web::cli::{self, Command, PROGRAM};
use phairplay_web::receiver::{self, Startup};
use phairplay_web::server;
use phairplay_web::state::AppState;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let command = match cli::parse(&arguments) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{PROGRAM}: {error}");
            return ExitCode::FAILURE;
        }
    };

    let options = match command {
        Command::Help => {
            println!("{}", cli::help());
            return ExitCode::SUCCESS;
        }
        Command::Version => {
            println!("{}", cli::version());
            return ExitCode::SUCCESS;
        }
        Command::Run(options) => options,
    };

    // WHY: a receiver whose control panel is in the network is a different
    // security model than one whose panel is on the loopback interface. The
    // warning is printed before anything is started, so that it is in the
    // log of the device and not only on the screen of the user.
    if !options.bind.is_loopback() {
        eprintln!(
            "{PROGRAM}: WARNING — the dashboard is bound to {address}, not to the loopback \
             interface. Every device of the local network can open the control panel of the \
             receiver, read its configuration and switch its protocols off. The default is \
             127.0.0.1: the panel is a local control panel.",
            address = options.bind
        );
    }

    let config = match receiver::configuration(&options) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{PROGRAM}: {message}");
            return ExitCode::FAILURE;
        }
    };

    let core = match receiver::build(&config) {
        Ok(core) => core,
        Err(error) => {
            eprintln!("{PROGRAM}: the receiver could not start: {error}");
            return ExitCode::FAILURE;
        }
    };

    let startup = receiver::start(&core);
    report(&startup);

    let state = AppState::new(core.clone());
    let served = server::serve(state, options.port, options.bind, options.dashboard).await;

    // WHY: the sockets of the protocols are released before the process
    // leaves. A receiver that is stopped with SIGTERM and that keeps its
    // ports bound cannot be started again.
    receiver::stop(&core);

    match served {
        Ok(address) => {
            // WHY: with `--no-dashboard` the files of the panel are not
            // served — only the API is. A message that promises a panel
            // that is not there sends the user to a URL that answers 404.
            if options.dashboard {
                println!("{PROGRAM}: the dashboard is served on http://{address}");
            } else {
                println!(
                    "{PROGRAM}: the API is served on http://{address} (the dashboard is switched \
                     off)"
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{PROGRAM}: the server could not start: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Prints the report of the start.
///
/// WHY: the user has to see what the receiver does without opening a
/// browser. A protocol that runs is named with its state, a protocol that
/// could not start is named with the reason.
fn report(startup: &Startup) {
    println!(
        "{PROGRAM}: the receiver starts {} protocols:",
        startup.outcomes.len()
    );
    for outcome in &startup.outcomes {
        match &outcome.reason {
            None => println!(
                "  {:<16} {}",
                outcome.protocol.display_name(),
                outcome.state
            ),
            Some(reason) => eprintln!(
                "  {:<16} not started — {reason}",
                outcome.protocol.display_name()
            ),
        }
    }
    if startup.failed() == 0 {
        println!(
            "{PROGRAM}: {} of {} protocols are running.",
            startup.running(),
            startup.outcomes.len()
        );
    } else {
        println!(
            "{PROGRAM}: {} of {} protocols are running; the reason of every failure is in the \
             status of the protocol.",
            startup.running(),
            startup.outcomes.len()
        );
    }
}
