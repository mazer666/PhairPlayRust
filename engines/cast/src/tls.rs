//! The TLS layer of the Cast receiver.
//!
//! WHY: the Google Cast protocol runs over TLS. A sender opens a TLS
//! connection to port 8009 and refuses to talk to a plain TCP socket. The
//! receiver therefore needs a server certificate; because the receiver is a
//! local device on a home network, a self-signed certificate is enough — the
//! sender accepts it, it only protects the link against a passive observer.
//!
//! WHAT: [`self_signed`] generates a certificate and the matching server
//! configuration, [`config_from_der`] builds a configuration from a
//! certificate the caller already has.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_cast::tls::self_signed;
//!
//! let (certificate, config) = self_signed(&["localhost".to_string()]).expect("certificate");
//! assert!(!certificate.as_ref().is_empty());
//! assert_eq!(config.alpn_protocols, Vec::<Vec<u8>>::new());
//! ```

use std::sync::Arc;

use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

/// A failure while preparing the TLS layer.
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    /// The certificate could not be generated.
    #[error("could not generate the certificate: {0}")]
    Certificate(String),
    /// The server configuration could not be built.
    #[error("could not build the TLS configuration: {0}")]
    Configuration(String),
}

/// Generates a self-signed certificate and the matching server configuration.
///
/// The certificate carries the given names as subject alternative names, so a
/// sender that checks the name finds the device it talked to. The generated
/// certificate is returned as well: the caller may want to publish it, and
/// the tests need it to build a client that trusts the receiver.
///
/// # Errors
/// Returns [`TlsError::Certificate`] when the certificate cannot be
/// generated and [`TlsError::Configuration`] when rustls rejects the key or
/// the certificate.
pub fn self_signed(
    names: &[String],
) -> Result<(CertificateDer<'static>, Arc<ServerConfig>), TlsError> {
    let params = rcgen::CertificateParams::new(names.to_vec())
        .map_err(|error| TlsError::Certificate(error.to_string()))?;
    let key_pair =
        rcgen::KeyPair::generate().map_err(|error| TlsError::Certificate(error.to_string()))?;
    let certificate = params
        .self_signed(&key_pair)
        .map_err(|error| TlsError::Certificate(error.to_string()))?;
    let certificate_der = certificate.der().clone();
    let key_der = key_pair.serialize_der();
    let config = config_from_der(certificate_der.clone(), key_der)?;
    Ok((certificate_der, config))
}

/// Builds a server configuration from a certificate and a PKCS#8 key.
///
/// # Errors
/// Returns [`TlsError::Configuration`] when rustls rejects the certificate
/// or the key — for instance when the key does not belong to the
/// certificate.
pub fn config_from_der(
    certificate: CertificateDer<'static>,
    key_der: Vec<u8>,
) -> Result<Arc<ServerConfig>, TlsError> {
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], key)
        .map_err(|error| TlsError::Configuration(error.to_string()))?;
    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_self_signed_certificate_is_generated() {
        let (certificate, _config) = self_signed(&["localhost".to_string()]).expect("certificate");
        assert!(!certificate.as_ref().is_empty());
    }

    #[test]
    fn the_generated_configuration_carries_the_certificate() {
        let (certificate, config) = self_signed(&["localhost".to_string()]).expect("certificate");
        // rustls accepted the key for the certificate, which is the proof
        // that the two halves belong together.
        assert!(!certificate.as_ref().is_empty());
        assert!(config.alpn_protocols.is_empty());
    }

    #[test]
    fn a_key_that_does_not_belong_to_the_certificate_is_rejected() {
        let (certificate, _config) = self_signed(&["localhost".to_string()]).expect("certificate");
        // A second, unrelated key pair: the certificate belongs to the first.
        let params = rcgen::CertificateParams::new(vec!["other".to_string()]).expect("parameters");
        let key_pair = rcgen::KeyPair::generate().expect("key");
        params
            .self_signed(&key_pair)
            .expect("certificate")
            .der()
            .clone();
        let error = config_from_der(certificate, key_pair.serialize_der()).expect_err("must fail");
        assert!(matches!(error, TlsError::Configuration(_)));
    }

    #[test]
    fn a_broken_key_is_rejected() {
        let (certificate, _config) = self_signed(&["localhost".to_string()]).expect("certificate");
        let error = config_from_der(certificate, vec![0x01, 0x02, 0x03]).expect_err("must fail");
        assert!(matches!(error, TlsError::Configuration(_)));
    }
}
