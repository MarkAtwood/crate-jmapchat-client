use clap::Parser;
use jmap_chat::{
    AuthProvider, BasicAuth, BearerAuth, ClientError, CustomCaTransport, DefaultTransport,
    NoneAuth, TransportConfig,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jmap-chat-egui", about = "JMAP Chat GUI client")]
pub struct Config {
    /// JMAP server base URL (e.g. https://chat.example.com)
    #[arg(long, value_name = "URL")]
    pub server_url: String,

    /// Bearer token for authentication
    #[arg(long, value_name = "TOKEN")]
    pub bearer_token: Option<String>,

    /// Username for Basic authentication
    #[arg(long, value_name = "USER")]
    pub basic_user: Option<String>,

    /// Password for Basic authentication
    #[arg(long, value_name = "PASS")]
    pub basic_pass: Option<String>,

    /// Path to DER-encoded custom CA certificate
    #[arg(long, value_name = "FILE")]
    pub ca_cert: Option<PathBuf>,
}

impl Config {
    /// Build the transport configuration from the parsed CLI flags.
    ///
    /// Returns a [`CustomCaTransport`] when `--ca-cert` is provided, otherwise
    /// a [`DefaultTransport`]. Transport and credentials are independent:
    /// `--ca-cert` may now be combined with `--bearer-token` or
    /// `--basic-user`/`--basic-pass`.
    ///
    /// # Errors
    ///
    /// - [`ClientError::InvalidArgument`] if `--ca-cert` is provided but the
    ///   file cannot be read.
    pub fn transport(&self) -> Result<Box<dyn TransportConfig>, ClientError> {
        if let Some(path) = &self.ca_cert {
            let der = std::fs::read(path).map_err(|e| {
                ClientError::InvalidArgument(format!(
                    "cannot read CA certificate '{}': {}",
                    path.display(),
                    e
                ))
            })?;
            return Ok(Box::new(CustomCaTransport::new(der)));
        }
        Ok(Box::new(DefaultTransport))
    }

    /// Build the credential provider from the parsed CLI flags.
    ///
    /// # Errors
    ///
    /// - [`ClientError::InvalidArgument`] if `--bearer-token` and
    ///   `--basic-user`/`--basic-pass` are both supplied (mutually exclusive).
    /// - [`ClientError::InvalidArgument`] if only one of `--basic-user` /
    ///   `--basic-pass` is set.
    /// - Propagates [`ClientError`] from the underlying auth constructors
    ///   (e.g. empty or invalid token, colon in username).
    pub fn auth(&self) -> Result<Box<dyn AuthProvider>, ClientError> {
        let has_bearer = self.bearer_token.is_some();
        let has_basic_user = self.basic_user.is_some();
        let has_basic_pass = self.basic_pass.is_some();

        if has_bearer && (has_basic_user || has_basic_pass) {
            return Err(ClientError::InvalidArgument(
                "--bearer-token and --basic-user/--basic-pass are mutually exclusive".into(),
            ));
        }

        if has_basic_user != has_basic_pass {
            return Err(ClientError::InvalidArgument(
                "--basic-user and --basic-pass must both be provided together".into(),
            ));
        }

        if let Some(token) = &self.bearer_token {
            return Ok(Box::new(BearerAuth::new(token)?));
        }

        if let (Some(user), Some(pass)) = (&self.basic_user, &self.basic_pass) {
            return Ok(Box::new(BasicAuth::new(user, pass)?));
        }

        Ok(Box::new(NoneAuth))
    }
}
