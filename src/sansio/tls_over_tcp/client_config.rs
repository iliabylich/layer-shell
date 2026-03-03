use rustls::{ClientConfig, RootCertStore, version::TLS13};
use std::sync::{Arc, LazyLock};

static ROOT_CERT_STORE: LazyLock<Arc<RootCertStore>> = LazyLock::new(|| {
    Arc::new(RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    })
});

static CLIENT_CONFIG: LazyLock<Arc<ClientConfig>> = LazyLock::new(|| {
    Arc::new(
        ClientConfig::builder_with_protocol_versions(&[&TLS13])
            .with_root_certificates(Arc::clone(&*ROOT_CERT_STORE))
            .with_no_client_auth(),
    )
});

pub(crate) fn get_client_config() -> Arc<ClientConfig> {
    Arc::clone(&*CLIENT_CONFIG)
}
