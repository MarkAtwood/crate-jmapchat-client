use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use reqwest::header::{HeaderName, HeaderValue, AUTHORIZATION};

use crate::error::ClientError;

/// Abstracts HTTP client construction and per-request authentication header injection.
///
/// Implementors control both how the underlying [`reqwest::Client`] is built (e.g. custom
/// trust roots, client certificates) and what `Authorization` header (if any) is attached
/// to each request.
pub trait AuthProvider: Send + Sync {
    /// Build the [`reqwest::Client`] for this auth configuration.
    fn build_client(&self) -> Result<reqwest::Client, ClientError>;

    /// Return an optional `(header-name, header-value)` pair to attach to every request.
    ///
    /// Implementations **must not** log the returned value; it may contain credentials.
    fn auth_header(&self) -> Option<(HeaderName, HeaderValue)>;
}

// ---------------------------------------------------------------------------
// NoneAuth
// ---------------------------------------------------------------------------

/// No authentication: default [`reqwest::Client`], no `Authorization` header.
pub struct NoneAuth;

impl AuthProvider for NoneAuth {
    fn build_client(&self) -> Result<reqwest::Client, ClientError> {
        Ok(reqwest::Client::new())
    }

    fn auth_header(&self) -> Option<(HeaderName, HeaderValue)> {
        None
    }
}

// ---------------------------------------------------------------------------
// BearerAuth
// ---------------------------------------------------------------------------

/// Bearer-token authentication (`Authorization: Bearer <token>`).
pub struct BearerAuth {
    pub token: String,
}

impl AuthProvider for BearerAuth {
    fn build_client(&self) -> Result<reqwest::Client, ClientError> {
        Ok(reqwest::Client::new())
    }

    fn auth_header(&self) -> Option<(HeaderName, HeaderValue)> {
        let value = format!("Bearer {}", self.token);
        // HeaderValue::from_str only fails for non-visible-ASCII; Bearer tokens are ASCII.
        HeaderValue::from_str(&value)
            .ok()
            .map(|hv| (AUTHORIZATION, hv))
    }
}

// ---------------------------------------------------------------------------
// BasicAuth
// ---------------------------------------------------------------------------

/// HTTP Basic authentication (`Authorization: Basic <base64(username:password)>`).
///
/// Credentials are encoded per RFC 7617: `base64(username ":" password)`.
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

impl AuthProvider for BasicAuth {
    fn build_client(&self) -> Result<reqwest::Client, ClientError> {
        Ok(reqwest::Client::new())
    }

    fn auth_header(&self) -> Option<(HeaderName, HeaderValue)> {
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded = BASE64_STANDARD.encode(credentials.as_bytes());
        let value = format!("Basic {encoded}");
        HeaderValue::from_str(&value)
            .ok()
            .map(|hv| (AUTHORIZATION, hv))
    }
}

// ---------------------------------------------------------------------------
// CustomCaAuth
// ---------------------------------------------------------------------------

/// Custom CA trust root (DER-encoded). No `Authorization` header is injected.
///
/// Used when the server presents a certificate signed by a private CA (e.g. kith).
pub struct CustomCaAuth {
    pub der_cert: Vec<u8>,
}

impl AuthProvider for CustomCaAuth {
    fn build_client(&self) -> Result<reqwest::Client, ClientError> {
        let cert = reqwest::Certificate::from_der(&self.der_cert)?;
        let client = reqwest::ClientBuilder::new()
            .add_root_certificate(cert)
            .build()?;
        Ok(client)
    }

    fn auth_header(&self) -> Option<(HeaderName, HeaderValue)> {
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: NoneAuth has no authentication header — verified by inspection of the spec.
    #[test]
    fn none_auth_no_header() {
        assert!(NoneAuth.auth_header().is_none());
    }

    /// Oracle: BearerAuth header value is "Bearer " + the literal token string.
    #[test]
    fn bearer_auth_header() {
        let auth = BearerAuth {
            token: "tok123".into(),
        };
        let (name, value) = auth.auth_header().expect("BearerAuth must return a header");
        assert_eq!(name, AUTHORIZATION);
        assert_eq!(value.to_str().unwrap(), "Bearer tok123");
    }

    /// Oracle: `echo -n "alice:s3cr3t" | base64` → `YWxpY2U6czNjcjN0`  (RFC 7617 §2)
    #[test]
    fn basic_auth_header() {
        let auth = BasicAuth {
            username: "alice".into(),
            password: "s3cr3t".into(),
        };
        let (name, value) = auth.auth_header().expect("BasicAuth must return a header");
        assert_eq!(name, AUTHORIZATION);
        assert_eq!(value.to_str().unwrap(), "Basic YWxpY2U6czNjcjN0");
    }

    /// Oracle: CustomCaAuth injects no auth header — no server identity is involved.
    #[test]
    fn custom_ca_auth_no_header() {
        let auth = CustomCaAuth { der_cert: vec![] };
        assert!(auth.auth_header().is_none());
    }

    /// Oracle: NoneAuth uses the default reqwest::Client which always builds successfully.
    #[tokio::test]
    async fn none_auth_builds_client() {
        NoneAuth
            .build_client()
            .expect("NoneAuth::build_client must succeed");
    }
}
