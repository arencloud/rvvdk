use crate::{Error, Result, xml};
use reqwest::{Client, Url, header};
use rustls::{
    DigitallySignedStruct, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

/// No implicit trust or fallback: the exact leaf DER SHA-256 is the trust anchor.
/// Pin mode intentionally replaces CA/hostname/expiry validation; obtain the pin
/// independently. TLS handshake signatures still prove possession of its key.
#[derive(Clone)]
pub struct ConnectionPolicy {
    pub(crate) endpoint: Url,
    pin: [u8; 32],
    pub(crate) timeouts: Timeouts,
    pub(crate) reuse: ConnectionReuse,
}
impl fmt::Debug for ConnectionPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConnectionPolicy([redacted endpoint and pin])")
    }
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionReuse {
    Reuse,
    Fresh,
}
#[derive(Clone, Copy, Debug)]
pub struct Timeouts {
    pub connect: Duration,
    pub inactivity: Duration,
    pub request: Duration,
    pub discovery: Duration,
    pub cleanup: Duration,
}
impl Default for Timeouts {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(5),
            inactivity: Duration::from_secs(5),
            request: Duration::from_secs(15),
            discovery: Duration::from_secs(120),
            cleanup: Duration::from_secs(15),
        }
    }
}
impl ConnectionPolicy {
    pub fn pinned(endpoint: &str, sha256: &str) -> Result<Self> {
        if endpoint.len() > 2048 || sha256.len() != 64 {
            return Err(Error::InvalidInput);
        }
        let mut endpoint = Url::parse(endpoint).map_err(|_| Error::InvalidInput)?;
        if endpoint.scheme() != "https"
            || endpoint.host_str().is_none()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || !matches!(endpoint.path(), "/" | "/sdk")
        {
            return Err(Error::InvalidInput);
        }
        endpoint.set_path("/sdk");
        let mut pin = [0; 32];
        for (i, pair) in sha256.as_bytes().chunks_exact(2).enumerate() {
            let s = std::str::from_utf8(pair).map_err(|_| Error::InvalidInput)?;
            pin[i] = u8::from_str_radix(s, 16).map_err(|_| Error::InvalidInput)?;
        }
        Ok(Self {
            endpoint,
            pin,
            timeouts: Timeouts::default(),
            reuse: ConnectionReuse::Reuse,
        })
    }
    pub fn with_timeouts(mut self, timeouts: Timeouts) -> Result<Self> {
        if [
            timeouts.connect,
            timeouts.inactivity,
            timeouts.request,
            timeouts.discovery,
            timeouts.cleanup,
        ]
        .iter()
        .any(|d| d.is_zero() || *d > Duration::from_secs(600))
        {
            return Err(Error::InvalidInput);
        }
        self.timeouts = timeouts;
        Ok(self)
    }
    /// Fresh connections exist for qualification of the reuse policy, not disk tuning.
    pub fn with_connection_reuse(mut self, reuse: ConnectionReuse) -> Self {
        self.reuse = reuse;
        self
    }
}

struct PinVerifier {
    pin: [u8; 32],
    peer_sha1: Arc<std::sync::Mutex<Option<[u8; 20]>>>,
    provider: Arc<rustls::crypto::CryptoProvider>,
    handshakes: Arc<AtomicUsize>,
}
impl fmt::Debug for PinVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PinVerifier([redacted])")
    }
}
impl ServerCertVerifier for PinVerifier {
    fn verify_server_cert(
        &self,
        cert: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        let actual: [u8; 32] = Sha256::digest(cert.as_ref()).into();
        if actual != self.pin {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::ApplicationVerificationFailure,
            ));
        }
        if let Ok(mut peer) = self.peer_sha1.lock() {
            *peer = Some(sha1::Sha1::digest(cert.as_ref()).into());
        }
        self.handshakes.fetch_add(1, Ordering::Relaxed);
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}
#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) enum Method {
    RetrieveServiceContent,
    Login,
    Logout,
    RetrievePropertiesEx,
    CancelRetrievePropertiesEx,
    ExportVm,
    ShutdownGuest,
    HttpNfcLeaseAbort,
    HttpNfcLeaseComplete,
    HttpNfcLeaseProgress,
    HttpNfcLeaseGetManifest,
}
impl Method {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::RetrieveServiceContent => "RetrieveServiceContent",
            Self::Login => "Login",
            Self::Logout => "Logout",
            Self::RetrievePropertiesEx => "RetrievePropertiesEx",
            Self::CancelRetrievePropertiesEx => "CancelRetrievePropertiesEx",
            Self::ExportVm => "ExportVm",
            Self::ShutdownGuest => "ShutdownGuest",
            Self::HttpNfcLeaseAbort => "HttpNfcLeaseAbort",
            Self::HttpNfcLeaseComplete => "HttpNfcLeaseComplete",
            Self::HttpNfcLeaseProgress => "HttpNfcLeaseProgress",
            Self::HttpNfcLeaseGetManifest => "HttpNfcLeaseGetManifest",
        }
    }
}
#[derive(Debug, Serialize)]
pub struct RequestMeasurement {
    pub method: &'static str,
    pub elapsed_ms: f64,
    pub response_bytes: usize,
    pub error: Option<Error>,
}

pub(crate) struct Transport {
    client: Client,
    policy: ConnectionPolicy,
    cookie: Option<Zeroizing<String>>,
    pub(crate) measurements: Vec<RequestMeasurement>,
    handshakes: Arc<AtomicUsize>,
    peer_sha1: Arc<std::sync::Mutex<Option<[u8; 20]>>>,
}
impl Transport {
    pub(crate) fn new(policy: ConnectionPolicy) -> Result<Self> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let handshakes = Arc::new(AtomicUsize::new(0));
        let peer_sha1 = Arc::new(std::sync::Mutex::new(None));
        let mut tls = rustls::ClientConfig::builder_with_provider(provider.clone())
            .with_safe_default_protocol_versions()
            .map_err(|_| Error::Transport)?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(PinVerifier {
                pin: policy.pin,
                peer_sha1: peer_sha1.clone(),
                provider,
                handshakes: handshakes.clone(),
            }))
            .with_no_client_auth();
        tls.resumption = rustls::client::Resumption::disabled();
        tls.enable_early_data = false;
        let client = Client::builder()
            .tls_backend_preconfigured(tls)
            .https_only(true)
            .http1_only()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .connect_timeout(policy.timeouts.connect)
            .read_timeout(policy.timeouts.inactivity)
            .timeout(policy.timeouts.request)
            .pool_max_idle_per_host(if matches!(policy.reuse, ConnectionReuse::Reuse) {
                1
            } else {
                0
            })
            .http1_max_headers(64)
            .build()
            .map_err(|_| Error::Transport)?;
        Ok(Self {
            client,
            policy,
            cookie: None,
            measurements: Vec::new(),
            handshakes,
            peer_sha1,
        })
    }
    pub(crate) fn admit_data_url(&self, raw: &str, thumbprint: &str) -> Result<Url> {
        if raw.len() > 4096 {
            return Err(Error::InvalidInput);
        }
        let mut url = Url::parse(raw).map_err(|_| Error::InvalidInput)?;
        if url.host_str() == Some("*") {
            url.set_host(self.policy.endpoint.host_str())
                .map_err(|_| Error::DataEndpoint)?;
        }
        if url.scheme() != "https"
            || url.host_str() != self.policy.endpoint.host_str()
            || url.port_or_known_default() != self.policy.endpoint.port_or_known_default()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || !(url.path().starts_with("/nfc/") || url.path().starts_with("/ha-nfc/"))
        {
            return Err(Error::DataEndpoint);
        }
        let hex = thumbprint.replace(':', "").to_ascii_lowercase();
        let expected = if hex.len() == 64 {
            hex_string(&self.policy.pin)
        } else if hex.len() == 40 {
            let peer = self.peer_sha1.lock().map_err(|_| Error::Transport)?;
            hex_string(&peer.ok_or(Error::Transport)?)
        } else {
            return Err(Error::DataEndpoint);
        };
        if hex != expected {
            return Err(Error::DataEndpoint);
        }
        Ok(url)
    }
    pub(crate) fn data_request(
        &self,
        url: Url,
        deadline: Instant,
    ) -> Result<reqwest::RequestBuilder> {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(Error::Deadline)?;
        // The client has no cookie store/default authorization. API credentials
        // are attached only by exchange(), never to this GET.
        Ok(self
            .client
            .get(url)
            .header(header::ACCEPT_ENCODING, "identity")
            .timeout(remaining))
    }
    pub(crate) fn handshakes(&self) -> usize {
        self.handshakes.load(Ordering::Relaxed)
    }
    pub(crate) fn has_cookie(&self) -> bool {
        self.cookie.is_some()
    }
    pub(crate) fn clear_cookie(&mut self) {
        self.cookie = None;
    }
    pub(crate) async fn call(
        &mut self,
        method: Method,
        body: String,
        deadline: Instant,
    ) -> Result<Zeroizing<String>> {
        if self.measurements.len() >= 4096
            && !matches!(method, Method::Logout | Method::HttpNfcLeaseAbort)
        {
            return Err(Error::RequestLimit);
        }
        let started = Instant::now();
        let mut bytes = 0;
        let result = match deadline.checked_duration_since(started) {
            None => Err(Error::Deadline),
            Some(remaining) => match tokio::time::timeout(
                remaining.min(self.policy.timeouts.request),
                self.exchange(method, Zeroizing::new(body), &mut bytes),
            )
            .await
            {
                Ok(result) => result,
                Err(_) => Err(Error::Deadline),
            },
        };
        self.measurements.push(RequestMeasurement {
            method: method.name(),
            elapsed_ms: started.elapsed().as_secs_f64() * 1000.,
            response_bytes: bytes,
            error: result.as_ref().err().copied(),
        });
        result
    }
    async fn exchange(
        &mut self,
        method: Method,
        body: Zeroizing<String>,
        bytes: &mut usize,
    ) -> Result<Zeroizing<String>> {
        let payload = format!(
            "<s:Envelope xmlns:s='http://schemas.xmlsoap.org/soap/envelope/' xmlns:xsi='http://www.w3.org/2001/XMLSchema-instance'><s:Body><{name} xmlns='urn:vim25'>{body}</{name}></s:Body></s:Envelope>",
            name = method.name(),
            body = body.as_str()
        );
        let mut request = self
            .client
            .post(self.policy.endpoint.clone())
            .header(header::CONTENT_TYPE, "text/xml; charset=utf-8")
            .header("SOAPAction", "\"urn:vim25/8.0\"")
            .header(header::ACCEPT_ENCODING, "identity")
            .body(payload);
        if let Some(cookie) = &self.cookie {
            let mut value = header::HeaderValue::from_str(cookie).map_err(|_| Error::Cookie)?;
            value.set_sensitive(true);
            request = request.header(header::COOKIE, value);
        }
        if matches!(self.policy.reuse, ConnectionReuse::Fresh) {
            request = request.header(header::CONNECTION, "close");
        }
        let mut response = request.send().await.map_err(map_http)?;
        let status = response.status().as_u16();
        if status != 200 && status != 500 {
            return Err(Error::Http);
        }
        // Capture the login cookie before body validation, so malformed/truncated
        // successful-login responses can still be followed by authenticated Logout.
        if matches!(method, Method::Login) {
            let mut candidate = None;
            for value in response.headers().get_all(header::SET_COOKIE) {
                let value = value.to_str().map_err(|_| Error::Cookie)?;
                if value.starts_with("vmware_soap_session=") {
                    if candidate.is_some() {
                        return Err(Error::Cookie);
                    }
                    let pair = value.split(';').next().ok_or(Error::Cookie)?;
                    let token = pair
                        .strip_prefix("vmware_soap_session=")
                        .ok_or(Error::Cookie)?;
                    if token.is_empty()
                        || token.len() > 1024
                        || token == "\"\""
                        || !token
                            .bytes()
                            .all(|b| b.is_ascii_graphic() && b != b';' && b != b',')
                    {
                        return Err(Error::Cookie);
                    }
                    candidate = Some(Zeroizing::new(pair.to_owned()));
                }
            }
            if candidate.is_some() {
                self.cookie = candidate;
            }
        }
        if response
            .headers()
            .get(header::CONTENT_ENCODING)
            .is_some_and(|h| h != "identity")
        {
            return Err(Error::Http);
        }
        if response
            .content_length()
            .is_some_and(|n| n > xml::RESPONSE_LIMIT as u64)
        {
            return Err(Error::ResponseLimit);
        }
        // Reserve from the admitted length, or the fixed cap for streaming
        // bodies, so capacity growth cannot overshoot the response budget.
        let capacity = response
            .content_length()
            .unwrap_or(xml::RESPONSE_LIMIT as u64) as usize;
        let mut raw = Zeroizing::new(Vec::with_capacity(capacity));
        while let Some(chunk) = response.chunk().await.map_err(map_http)? {
            *bytes = bytes.saturating_add(chunk.len());
            if *bytes > xml::RESPONSE_LIMIT {
                return Err(Error::ResponseLimit);
            }
            raw.extend_from_slice(&chunk);
        }
        let raw = Zeroizing::new(
            std::str::from_utf8(&raw)
                .map_err(|_| Error::Xml)?
                .to_owned(),
        );
        let doc = xml::parse(&raw)?;
        xml::response(&doc, method.name())?;
        if status != 200 {
            return Err(Error::Http);
        }
        Ok(raw)
    }
}
fn map_http(error: reqwest::Error) -> Error {
    if error.is_timeout() {
        Error::Deadline
    } else {
        Error::Transport
    }
}

pub(crate) fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_authority_and_secret_safe_debug() {
        for endpoint in [
            "http://example.invalid",
            "https://user:SECRET@example.invalid",
            "https://example.invalid/sdk?ticket=SECRET",
            "https://example.invalid/other",
            "https://example.invalid/#SECRET",
        ] {
            assert_eq!(
                ConnectionPolicy::pinned(endpoint, &"a".repeat(64)).unwrap_err(),
                Error::InvalidInput
            );
        }
        let policy =
            ConnectionPolicy::pinned("https://private.invalid:8443", &"a".repeat(64)).unwrap();
        assert_eq!(policy.endpoint.as_str(), "https://private.invalid:8443/sdk");
        assert!(!format!("{policy:?}").contains("private.invalid"));
        assert!(ConnectionPolicy::pinned("https://example.invalid", &"z".repeat(64)).is_err());
    }
}
