use quinn::{Endpoint, ServerConfig, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::sync::Arc;
use std::time::Duration;

pub fn make_self_signed(bind_addr: std::net::SocketAddr) -> anyhow::Result<Endpoint> {
    let cert = rcgen::generate_simple_self_signed(vec!["samp.cef".into()])?;
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());

    let server_config = configure_server(cert_der, priv_key)?;
    Ok(Endpoint::server(server_config, bind_addr)?)
}

fn configure_server(
    cert: CertificateDer<'static>, priv_key: PrivatePkcs8KeyDer<'static>,
) -> anyhow::Result<ServerConfig> {
    let mut transport_config = TransportConfig::default();
    transport_config.keep_alive_interval(Some(Duration::from_secs(1)));

    let mut server_config = ServerConfig::with_single_cert(vec![cert], priv_key.into())?;
    server_config.transport = Arc::new(transport_config);

    Ok(server_config)
}
