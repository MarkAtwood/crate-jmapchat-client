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
    header_value: HeaderValue,
}

impl BearerAuth {
    /// Construct a `BearerAuth` from a Bearer token string.
    ///
    /// Returns `Err` if the token contains characters that are not valid in an
    /// HTTP header value (i.e. non-visible-ASCII or non-whitespace octets).
    /// Validation happens here so that `auth_header` can never fail silently.
    pub fn new(token: &str) -> Result<Self, ClientError> {
        let header_value = HeaderValue::from_str(&format!("Bearer {token}"))?;
        Ok(Self { header_value })
    }
}

impl AuthProvider for BearerAuth {
    fn build_client(&self) -> Result<reqwest::Client, ClientError> {
        Ok(reqwest::Client::new())
    }

    fn auth_header(&self) -> Option<(HeaderName, HeaderValue)> {
        Some((AUTHORIZATION, self.header_value.clone()))
    }
}

// ---------------------------------------------------------------------------
// BasicAuth
// ---------------------------------------------------------------------------

/// HTTP Basic authentication (`Authorization: Basic <base64(username:password)>`).
///
/// Credentials are encoded per RFC 7617: `base64(username ":" password)`.
pub struct BasicAuth {
    header_value: HeaderValue,
}

impl BasicAuth {
    /// Construct a `BasicAuth` from a username and password.
    ///
    /// Returns `Err` if:
    /// - `username` contains a colon (`:`) — forbidden by RFC 7617 §2.
    /// - The resulting header value contains characters invalid in an HTTP header.
    pub fn new(username: &str, password: &str) -> Result<Self, ClientError> {
        if username.contains(':') {
            return Err(ClientError::Parse(
                "BasicAuth username may not contain ':'".into(),
            ));
        }
        let encoded = BASE64_STANDARD.encode(format!("{username}:{password}").as_bytes());
        let header_value = HeaderValue::from_str(&format!("Basic {encoded}"))?;
        Ok(Self { header_value })
    }
}

impl AuthProvider for BasicAuth {
    fn build_client(&self) -> Result<reqwest::Client, ClientError> {
        Ok(reqwest::Client::new())
    }

    fn auth_header(&self) -> Option<(HeaderName, HeaderValue)> {
        Some((AUTHORIZATION, self.header_value.clone()))
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

    /// Oracle: BearerAuth constructs successfully with a valid ASCII token.
    #[test]
    fn bearer_auth_valid_constructs() {
        assert!(BearerAuth::new("tok123").is_ok());
    }

    /// Oracle: BearerAuth header value is "Bearer " + the literal token string.
    /// Verified by inspection: the Authorization header MUST be "Bearer tok123".
    #[test]
    fn bearer_auth_header() {
        let auth = BearerAuth::new("tok123").expect("valid ASCII token must construct");
        let (name, value) = auth.auth_header().expect("BearerAuth must return a header");
        assert_eq!(name, AUTHORIZATION);
        assert_eq!(value.to_str().unwrap(), "Bearer tok123");
    }

    /// Oracle: BearerAuth constructor rejects tokens containing C0 control characters.
    /// HeaderValue::from_str rejects bytes 0x00-0x08 and 0x0A-0x1F (C0 controls,
    /// excluding HTAB 0x09) and 0x7F (DEL). '\x01' (SOH) is unconditionally invalid
    /// per RFC 7230 §3.2.6 and the http crate's header validation.
    #[test]
    fn bearer_auth_invalid_token_rejected() {
        let result = BearerAuth::new("tok\x01abc");
        assert!(
            result.is_err(),
            "token with C0 control character must be rejected by constructor"
        );
    }

    /// Oracle: BasicAuth constructs successfully with valid username and password.
    #[test]
    fn basic_auth_valid_constructs() {
        assert!(BasicAuth::new("alice", "s3cr3t").is_ok());
    }

    /// Oracle: BasicAuth constructor rejects usernames containing a colon (RFC 7617 §2).
    #[test]
    fn basic_auth_colon_in_username_rejected() {
        let result = BasicAuth::new("ali:ce", "s3cr3t");
        match result {
            Ok(_) => panic!("username with colon must be rejected by constructor"),
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("username"),
                    "error message should mention 'username', got: {err_msg}"
                );
            }
        }
    }

    /// Oracle: `echo -n "alice:s3cr3t" | base64` → `YWxpY2U6czNjcjN0`  (RFC 7617 §2)
    /// This expected value is computed independently of the code under test.
    #[test]
    fn basic_auth_header() {
        let auth = BasicAuth::new("alice", "s3cr3t").expect("valid credentials must construct");
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
