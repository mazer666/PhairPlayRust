//! The server of the dashboard and its address.
//!
//! WHAT: The function that binds the socket of the API, the constant that
//! names the interface it binds to by default, and the signal that ends the
//! server.
//!
//! WHY: The dashboard is a local control panel. It must be reachable from
//! the browser of the same machine and from nowhere else. The address is
//! therefore the loopback interface — written as
//! [`std::net::Ipv4Addr::LOCALHOST`], so that it cannot be changed into
//! `0.0.0.0` by accident — and the binding is the only place where a
//! socket of the dashboard is opened.
//!
//! WHY the shutdown is part of the server: a receiver that is stopped with
//! `SIGTERM` has to release its sockets. The server therefore waits for
//! the signal, lets the running requests finish and only then returns —
//! the caller stops the engines afterwards.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_web::server::LOOPBACK_HOST;
//! use std::str::FromStr;
//!
//! assert!(std::net::IpAddr::from_str(LOOPBACK_HOST).expect("a valid address").is_loopback());
//! ```

use std::net::{IpAddr, SocketAddr};

use tokio::net::TcpListener;

use crate::api::router_with;
use crate::state::AppState;

/// The interface the server binds to.
///
/// WHY: the loopback interface is the only one that a program on the same
/// machine can reach. The receiver is a device on the network of the user;
/// a dashboard that anybody in the network can reach would be a door into
/// the receiver.
pub const LOOPBACK_HOST: &str = "127.0.0.1";

/// The port the server binds to, when the user did not choose one.
pub const DEFAULT_PORT: u16 = 7878;

/// Binds the socket of the API.
///
/// # Errors
/// Returns the failure of the operating system when the port is taken or
/// when the interface is not available.
pub async fn bind(port: u16, address: IpAddr) -> std::io::Result<TcpListener> {
    TcpListener::bind(SocketAddr::new(address, port)).await
}

/// Serves the API.
///
/// # Errors
/// Returns the failure of the operating system when the socket cannot be
/// bound or when the server stops with an error.
pub async fn serve(
    state: AppState,
    port: u16,
    address: IpAddr,
    dashboard: bool,
) -> std::io::Result<SocketAddr> {
    let listener = bind(port, address).await?;
    let bound = listener.local_addr()?;
    tracing::debug!("the dashboard is served on http://{bound}");
    axum::serve(listener, router_with(state, dashboard))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(bound)
}

/// Waits for the signal that ends the program.
///
/// WHY: `SIGINT` (the ctrl-c of a terminal) and `SIGTERM` (the stop of a
/// service manager) are the two ways a user ends a receiver. Both have to
/// end it the same way: the running requests are finished, the sockets are
/// released and the process leaves with the code zero.
pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let interrupt = signal(SignalKind::interrupt());
        let terminate = signal(SignalKind::terminate());
        match (interrupt, terminate) {
            (Ok(mut interrupt), Ok(mut terminate)) => {
                tokio::select! {
                    _ = interrupt.recv() => {}
                    _ = terminate.recv() => {}
                }
            }
            // WHY: a system that does not know the unix signals still has
            // the ctrl-c of the terminal. The program waits for it instead
            // of refusing to stop.
            _ => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn the_address_of_the_server_is_the_loopback_interface() {
        // WHY: the whole privacy promise of the receiver depends on this
        // one line. A server that binds to 0.0.0.0 would be reachable from
        // the network of the user.
        let address: std::net::IpAddr = LOOPBACK_HOST
            .parse()
            .unwrap_or_else(|_| panic!("{LOOPBACK_HOST} must be an address"));
        assert!(address.is_loopback(), "{address} is not a loopback address");
    }

    #[tokio::test]
    async fn the_bound_socket_is_on_the_loopback_interface() {
        let listener = bind(0, IpAddr::V4(Ipv4Addr::LOCALHOST))
            .await
            .expect("the socket must be bindable");
        let address = listener.local_addr().expect("the address must be known");
        assert!(
            address.ip().is_loopback(),
            "{address} is not a loopback address"
        );
    }

    #[tokio::test]
    async fn the_socket_can_be_bound_to_another_interface() {
        // WHY: the user may put the dashboard into the network. The
        // program warns about it — the warning is not the permission.
        let listener = bind(0, IpAddr::V4(Ipv4Addr::UNSPECIFIED))
            .await
            .expect("the socket must be bindable");
        let address = listener.local_addr().expect("the address must be known");
        assert_eq!(address.ip(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    }
}
