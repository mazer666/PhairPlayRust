//! The binary of the dashboard.
//!
//! WHAT: The program that starts the local control panel of PhairPlayRust.
//!
//! WHY: The user needs one program, not two: the binary serves the API and
//! the files of the dashboard. It binds to the loopback interface and tells
//! the user where the panel is.
//!
//! EXAMPLE:
//!
//! ```text
//! phairplay-web --port 7878
//! ```

use std::process::ExitCode;

use phairplay_core::config::AppConfig;
use phairplay_ffi::supervisor::ReceiverCore;
use phairplay_web::server::{DEFAULT_PORT, serve};
use phairplay_web::state::AppState;

/// The name of the environment variable that chooses the port.
const PORT_VARIABLE: &str = "PHAIRPLAY_WEB_PORT";

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let port = match port() {
        Ok(port) => port,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let receiver = match ReceiverCore::new(AppConfig::default()) {
        Ok(receiver) => receiver,
        Err(error) => {
            eprintln!("the receiver could not start: {error}");
            return ExitCode::FAILURE;
        }
    };
    let state = AppState::new(receiver);

    match serve(state, port).await {
        Ok(address) => {
            println!("the dashboard is served on http://{address}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("the server could not start: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The port the server shall bind to.
fn port() -> Result<u16, String> {
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--port" | "-p" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--port needs a value".to_string())?;
                return value
                    .parse::<u16>()
                    .map_err(|_| format!("`{value}` is not a port number"));
            }
            "--help" | "-h" => {
                return Err(format!(
                    "phairplay-web [--port <port>]\n\n\
                     The port may also be given in the environment variable {PORT_VARIABLE}.\n\
                     The server always binds to the loopback interface."
                ));
            }
            other => return Err(format!("`{other}` is not an option")),
        }
    }
    match std::env::var(PORT_VARIABLE) {
        Ok(value) => value
            .parse::<u16>()
            .map_err(|_| format!("`{value}` is not a port number")),
        Err(_) => Ok(DEFAULT_PORT),
    }
}
