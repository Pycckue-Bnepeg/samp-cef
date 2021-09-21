use quinn::{ClientConfig, ClientConfigBuilder, Endpoint, EndpointBuilder};
use std::sync::Arc;

struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self, _roots: &rustls::RootCertStore, _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef, _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}

pub fn make_insecure_client() -> EndpointBuilder {
    let client_cfg = configure_insecure_client();
    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.default_client_config(client_cfg);

    endpoint_builder
}

fn configure_insecure_client() -> ClientConfig {
    let mut cfg = ClientConfigBuilder::default().build();
    let tls_cfg: &mut rustls::ClientConfig = Arc::get_mut(&mut cfg.crypto).unwrap();

    tls_cfg
        .dangerous()
        .set_certificate_verifier(SkipServerVerification::new());

    cfg
}
