use anyhow::{Context as _, Result, ensure};
use openssl_sys::{
    BIO, BIO_new, BIO_s_mem, SSL, SSL_CTX, SSL_CTX_new, SSL_CTX_set_default_verify_paths,
    SSL_CTX_set_min_proto_version, SSL_CTX_set_verify, SSL_VERIFY_PEER, SSL_free, SSL_get0_param,
    SSL_new, SSL_set_bio, SSL_set_connect_state, SSL_set_tlsext_host_name, TLS_client_method,
    TLS1_2_VERSION, X509_VERIFY_PARAM_set_hostflags, X509_VERIFY_PARAM_set1_host,
};
use std::{ffi::CString, rc::Rc, str::FromStr};

pub(crate) struct OpenSslState {
    pub(crate) ssl: *mut SSL,
    pub(crate) rbio: *mut BIO,
    pub(crate) wbio: *mut BIO,
    _hostname: CString,
}

struct Ctx(*mut SSL_CTX);
unsafe impl Sync for Ctx {}
static mut CTX: Ctx = Ctx(std::ptr::null_mut());

impl OpenSslState {
    pub(crate) fn init() -> Result<()> {
        let ctx = unsafe { SSL_CTX_new(TLS_client_method()) };
        ensure!(!ctx.is_null(), "SSL_CTX is NULL");
        unsafe { SSL_CTX_set_verify(ctx, SSL_VERIFY_PEER, None) };
        let res = unsafe { SSL_CTX_set_default_verify_paths(ctx) };
        ensure!(res == 1, "SSL_CTX_set_default_verify_paths failed");
        let res = unsafe { SSL_CTX_set_min_proto_version(ctx, TLS1_2_VERSION) };
        ensure!(res == 1, "SSL_CTX_set_min_proto_version failed");
        unsafe { CTX.0 = ctx };
        Ok(())
    }

    pub(crate) fn new(hostname: &str) -> Result<Rc<Self>> {
        let ctx = unsafe { CTX.0 };

        let ssl = unsafe { SSL_new(ctx) };
        ensure!(!ssl.is_null(), "SSL is NULL");

        let hostname = CString::from_str(hostname).context("hostname contains NULL")?;
        let res = unsafe { SSL_set_tlsext_host_name(ssl, hostname.as_ptr().cast_mut()) };
        ensure!(res == 1, "SSL_set_tlsext_host_name failed");

        let param = unsafe { SSL_get0_param(ssl) };
        unsafe { X509_VERIFY_PARAM_set_hostflags(param, 0) };
        let res = unsafe { X509_VERIFY_PARAM_set1_host(param, hostname.as_ptr().cast_mut(), 0) };
        ensure!(res == 1, "X509_VERIFY_PARAM_set1_host failed");

        let rbio = unsafe { BIO_new(BIO_s_mem()) };
        ensure!(!rbio.is_null(), "rbio is NULL");
        let wbio = unsafe { BIO_new(BIO_s_mem()) };
        ensure!(!wbio.is_null(), "wbio is NULL");

        unsafe { SSL_set_bio(ssl, rbio, wbio) };

        unsafe { SSL_set_connect_state(ssl) };

        Ok(Rc::new(Self {
            ssl,
            rbio,
            wbio,
            _hostname: hostname,
        }))
    }
}

impl Drop for OpenSslState {
    fn drop(&mut self) {
        unsafe { SSL_free(self.ssl) };
    }
}
