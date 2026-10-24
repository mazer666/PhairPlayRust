//! The server of the dashboard and its address.
//!
//! WHAT: The function that binds the socket of the API and the constant
//! that names the interface it binds to.
//!
//! WHY: The dashboard is a local control panel. It must be reachable from
//! the browser of the same machine and from nowhere else. The address is
//! therefore the loopback interface — written as
//! [`std::net::Ipv4Addr::LOCALHOST`], so that it cannot be changed into
//! `0.0.0.0` by accident — and the binding is the only place where a
//! socket is opened.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_web::server::LOOPBACK_HOST;
//!
//! assert!(std::net::IpAddr::from_str(LOOPBACK_HOST).expect("a valid address").is_loopback());
//! ```

use std::net::{Ipv4Addr, SocketAddr};

use tokio::net::TcpListener;

use crate::api::router;
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

/// Binds the socket of the API to the loopback interface.
///
/// # Errors
/// Returns the failure of the operating system when the port is taken or
/// when the interface is not available.
pub async fn bind(port: u16) -> std::io::Result<TcpListener> {
    TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await
}

/// Serves the API on the loopback interface.
///
/// # Errors
/// Returns the failure of the operating system when the socket cannot be
/// bound or when the server stops with an error.
pub async fn serve(state: AppState, port: u16) -> std::io::Result<SocketAddr> {
    let listener = bind(port).await?;
    let address = listener.local_addr()?;
    tracing::info!("the dashboard is served on http://{address}");
    axum::serve(listener, router(state)).await?;
    Ok(address)
}

#[cfg(test)]
mod tests {
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
        let listener = bind(0).await.expect("the socket must be bindable");
        let address = listener.local_addr().expect("the address must be known");
        assert!(
            address.ip().is_loopback(),
            "{address} is not a loopback address"
        );
    }
}
