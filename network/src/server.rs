use quinn::{
    Certificate, CertificateChain, Endpoint, EndpointBuilder, PrivateKey, ServerConfig,
    ServerConfigBuilder, TransportConfig,
};

use std::sync::Arc;
use std::time::Duration;

pub fn make_self_signed() -> anyhow::Result<EndpointBuilder> {
    let cert = rcgen::generate_simple_self_signed(vec!["samp.cef".into()])?;
    let cert_der = cert.serialize_der()?;
    let priv_key = cert.serialize_private_key_der();

    let cert = Certificate::from_der(&cert_der)?;
    let priv_key = PrivateKey::from_der(&priv_key)?;

    let cfg = configure_server(cert, priv_key)?;

    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.listen(cfg);

    Ok(endpoint_builder)
}

fn configure_server(cert: Certificate, priv_key: PrivateKey) -> anyhow::Result<ServerConfig> {
    let mut transport_config = TransportConfig::default();
    transport_config.keep_alive_interval(Some(Duration::from_secs(1)));

    let mut server_config = ServerConfig::default();
    server_config.transport = Arc::new(transport_config);

    let mut cfg_builder = ServerConfigBuilder::new(server_config);
    cfg_builder.certificate(CertificateChain::from_certs(vec![cert]), priv_key)?;

    Ok(cfg_builder.build())
}
