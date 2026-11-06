//! TLS for the live sources of a twin (MQTT over `mqtts://`, Kafka over
//! `kafkas://`): rustls with the ring provider and the system's trusted roots.
//! No OpenSSL anywhere, so the Linux binary needs no libssl.

use std::sync::Arc;

/// A client configuration trusting the system's roots; an error naming the
/// scheme when the system has none.
pub fn client_config(scheme: &str) -> Result<Arc<rustls::ClientConfig>, String> {
    let mut roots = rustls::RootCertStore::empty();
    for certificate in rustls_native_certs::load_native_certs().certs {
        let _ = roots.add(certificate);
    }
    if roots.is_empty() {
        return Err(format!("no trusted root certificate on this system for {scheme}"));
    }
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .map_err(|e| e.to_string())?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Arc::new(config))
}
