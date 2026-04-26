// JMAP core wire types — RFC 8620 §1.2, §1.4, §2, §3.2, §3.3, §3.4

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// An opaque server-assigned identifier string (RFC 8620 §1.2).
/// Guaranteed non-empty. Serializes/deserializes transparently as a JSON string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Id(String);

impl Id {
    /// Create an Id from a string, returning Err if the string is empty.
    pub fn new(s: impl Into<String>) -> Result<Self, crate::error::ClientError> {
        let s = s.into();
        if s.is_empty() {
            return Err(crate::error::ClientError::Parse(
                "Id may not be empty".into(),
            ));
        }
        Ok(Self(s))
    }

    /// Create an Id without validation. For use in tests and deserialization
    /// contexts where the source is trusted (e.g. server-assigned IDs).
    pub fn from_trusted(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Id {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for Id {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for Id {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Id> for &str {
    fn eq(&self, other: &Id) -> bool {
        *self == other.0
    }
}

/// An RFC 3339 UTC timestamp string (JMAP UTCDate, RFC 8620 §1.4).
/// Guaranteed non-empty. Serializes/deserializes transparently as a JSON string.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct UTCDate(String);

impl UTCDate {
    /// Create a UTCDate from a string, returning Err if the string is empty.
    pub fn new(s: impl Into<String>) -> Result<Self, crate::error::ClientError> {
        let s = s.into();
        if s.is_empty() {
            return Err(crate::error::ClientError::Parse(
                "UTCDate may not be empty".into(),
            ));
        }
        Ok(Self(s))
    }

    /// Create a UTCDate without validation. For use in tests and deserialization
    /// contexts where the source is trusted.
    pub fn from_trusted(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UTCDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for UTCDate {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for UTCDate {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for UTCDate {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for UTCDate {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<UTCDate> for &str {
    fn eq(&self, other: &UTCDate) -> bool {
        *self == other.0
    }
}

/// A single method call or response: `[methodName, arguments, callId]` (RFC 8620 §3.2).
pub type Invocation = (String, serde_json::Value, String);

/// JMAP API request (RFC 8620 §3.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JmapRequest {
    /// Capability URIs the client is using in this request.
    pub using: Vec<String>,
    /// Ordered list of method calls to execute.
    #[serde(rename = "methodCalls")]
    pub method_calls: Vec<Invocation>,
}

/// JMAP API response (RFC 8620 §3.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JmapResponse {
    /// Ordered list of method responses.
    #[serde(rename = "methodResponses")]
    pub method_responses: Vec<Invocation>,
    /// Server session state at the time the response was generated.
    #[serde(rename = "sessionState")]
    pub session_state: String,
    /// Map of client-supplied creation ids to server-assigned ids, if any.
    #[serde(rename = "createdIds", skip_serializing_if = "Option::is_none")]
    pub created_ids: Option<HashMap<String, String>>,
}

// ---------------------------------------------------------------------------
// Session (RFC 8620 §2 + JMAP Chat §3)
// ---------------------------------------------------------------------------

/// JMAP Session object returned by `GET /.well-known/jmap` (RFC 8620 §2).
///
/// JMAP Chat §3 extension fields (`ownerUserId`, `ownerLogin`, `ownerEndpoints`)
/// are included as optional fields.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// Map of capability URI → capability object (RFC 8620 §2).
    pub capabilities: HashMap<String, serde_json::Value>,
    /// Map of accountId → AccountInfo (RFC 8620 §2).
    pub accounts: HashMap<String, AccountInfo>,
    /// Map of capability URI → primary accountId (RFC 8620 §2).
    pub primary_accounts: HashMap<String, String>,
    /// Human-readable username for this session (RFC 8620 §2).
    pub username: String,
    /// URL for JMAP API POST requests (RFC 8620 §2).
    pub api_url: String,
    /// URL template for blob downloads (RFC 8620 §2).
    pub download_url: String,
    /// URL for blob uploads (RFC 8620 §2).
    pub upload_url: String,
    /// URL for the SSE push stream (RFC 8620 §2).
    pub event_source_url: String,
    /// Opaque session state token (RFC 8620 §2).
    pub state: String,

    /// The mailbox owner's ChatContact.id (JMAP Chat §3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<String>,
    /// Human-readable login name for the owner (JMAP Chat §3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_login: Option<String>,
    /// Owner's out-of-band capability endpoints (JMAP Chat §3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_endpoints: Option<Vec<crate::types::Endpoint>>,
}

impl Session {
    /// Returns the primary accountId for the JMAP Chat capability, if present.
    pub fn chat_account_id(&self) -> Option<&str> {
        self.primary_accounts
            .get("urn:ietf:params:jmap:chat")
            .map(String::as_str)
    }

    /// Returns the parsed `ChatCapability` for the given account.
    ///
    /// - `Ok(None)` — the account exists but has no chat capability key.
    /// - `Ok(Some(...))` — the capability is present and valid.
    /// - `Err(ClientError::Parse(...))` — the key is present but malformed.
    pub fn chat_capability(
        &self,
        account_id: &str,
    ) -> Result<Option<ChatCapability>, crate::error::ClientError> {
        let account = match self.accounts.get(account_id) {
            Some(a) => a,
            None => return Ok(None),
        };
        let raw = match account
            .account_capabilities
            .get("urn:ietf:params:jmap:chat")
        {
            Some(r) => r,
            None => return Ok(None),
        };
        serde_json::from_value::<ChatCapability>(raw.clone())
            .map(Some)
            .map_err(|e| {
                crate::error::ClientError::Parse(format!("malformed chat capability: {e}"))
            })
    }
}

/// Per-account metadata in a JMAP Session (RFC 8620 §2).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// Human-readable account name.
    pub name: String,
    /// Whether this is the user's primary/personal account.
    pub is_personal: bool,
    /// Whether this account is read-only.
    pub is_read_only: bool,
    /// Map of capability URI → capability object for this account.
    pub account_capabilities: HashMap<String, serde_json::Value>,
}

/// Chat-capability fields from `accounts[id].accountCapabilities["urn:ietf:params:jmap:chat"]`.
///
/// Spec: draft-atwood-jmap-chat-00 §3
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCapability {
    /// Maximum UTF-8 byte length of a Message body.
    pub max_body_bytes: u64,
    /// Maximum single attachment blob size in bytes.
    pub max_attachment_bytes: u64,
    /// Maximum number of attachments per message.
    pub max_attachments_per_message: u64,
    /// Maximum number of members in a group Chat.
    pub max_group_members: u64,
    /// Maximum number of members in a Space.
    pub max_space_members: u64,
    /// Maximum number of roles per Space.
    pub max_roles_per_space: u64,
    /// Maximum number of channels per Space.
    pub max_channels_per_space: u64,
    /// Maximum number of categories per Space.
    pub max_categories_per_space: u64,
    /// MIME types accepted in `bodyType`; always includes `"text/plain"`.
    pub supported_body_types: Vec<String>,
    /// Whether the server supports the optional thread model.
    pub supports_threads: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture(name: &str) -> serde_json::Value {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/jmap")
            .join(name);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read fixture {name}: {e}"));
        serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("fixture {name} is not valid JSON: {e}"))
    }

    // Oracle: RFC 8620 §3.3 — hand-written fixture derived from spec structure
    #[test]
    fn deserialize_request_from_fixture() {
        let val = fixture("request_chat_get.json");
        let req: JmapRequest = serde_json::from_value(val).expect("deserialize JmapRequest");

        assert_eq!(req.using[0], "urn:ietf:params:jmap:core");
        assert_eq!(req.method_calls[0].0, "Chat/get");
        assert_eq!(req.method_calls[0].2, "r1");
    }

    // Oracle: RFC 8620 §3.3 — serialize matches hand-written fixture exactly
    #[test]
    fn serialize_request_matches_fixture() {
        let req = JmapRequest {
            using: vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:chat".to_string(),
            ],
            method_calls: vec![
                (
                    "Chat/get".to_string(),
                    json!({"accountId": "account1", "ids": null}),
                    "r1".to_string(),
                ),
                (
                    "Message/get".to_string(),
                    json!({"accountId": "account1", "ids": ["msg1", "msg2"]}),
                    "r2".to_string(),
                ),
            ],
        };

        let serialized = serde_json::to_value(&req).expect("serialize JmapRequest");
        let expected = fixture("request_chat_get.json");
        assert_eq!(serialized, expected);
    }

    // Oracle: RFC 8620 §3.4 — hand-written fixture derived from spec structure
    #[test]
    fn deserialize_response_from_fixture() {
        let val = fixture("response_chat_get.json");
        let resp: JmapResponse = serde_json::from_value(val).expect("deserialize JmapResponse");

        assert_eq!(resp.session_state, "session-xyz789");
        assert_eq!(resp.method_responses[0].0, "Chat/get");
        assert!(resp.created_ids.is_none());
    }

    // Oracle: RFC 8620 §3.2 — Invocation is a 3-element JSON array
    #[test]
    fn invocation_serializes_as_array() {
        let inv: Invocation = ("Foo/get".to_string(), json!({}), "c1".to_string());
        let val = serde_json::to_value(&inv).expect("serialize Invocation");
        assert_eq!(val, json!(["Foo/get", {}, "c1"]));
    }

    // Oracle: RFC 8620 §3.4 — createdIds MUST be absent when not present
    #[test]
    fn response_created_ids_absent_when_none() {
        let resp = JmapResponse {
            method_responses: vec![],
            session_state: "s1".to_string(),
            created_ids: None,
        };
        let val = serde_json::to_value(&resp).expect("serialize JmapResponse");
        assert!(!val.as_object().unwrap().contains_key("createdIds"));
    }

    // Oracle: RFC 8620 §3.4 — createdIds MUST be present when populated
    #[test]
    fn response_created_ids_present_when_some() {
        let mut ids = HashMap::new();
        ids.insert("client-id-1".to_string(), "server-id-abc".to_string());
        let resp = JmapResponse {
            method_responses: vec![],
            session_state: "s1".to_string(),
            created_ids: Some(ids),
        };
        let val = serde_json::to_value(&resp).expect("serialize JmapResponse");
        let obj = val.as_object().unwrap();
        assert!(obj.contains_key("createdIds"));
        assert_eq!(obj["createdIds"]["client-id-1"], "server-id-abc");
    }

    // Oracle: RFC 8620 §2 — hand-written fixture matches spec Session structure
    #[test]
    fn session_deserializes_from_fixture() {
        let val = fixture("session.json");
        let session: Session =
            serde_json::from_value(val).expect("session.json must deserialize as Session");

        assert_eq!(session.username, "alice@example.com");
        assert_eq!(session.api_url, "https://jmap.example.com/api");
        assert_eq!(
            session.event_source_url,
            "https://jmap.example.com/eventsource/"
        );
        assert_eq!(session.state, "session-abc123");
        assert!(session.accounts.contains_key("account1"));
        assert!(session
            .capabilities
            .contains_key("urn:ietf:params:jmap:core"));
        assert!(session
            .capabilities
            .contains_key("urn:ietf:params:jmap:chat"));
        // JMAP Chat extension fields are absent in this fixture
        assert!(session.owner_user_id.is_none());
        assert!(session.owner_login.is_none());
        assert!(session.owner_endpoints.is_none());
    }

    // Oracle: RFC 8620 §2 — chat_account_id() extracts the primary account
    // from the fixture's primaryAccounts["urn:ietf:params:jmap:chat"] field.
    #[test]
    fn session_chat_account_id_returns_primary_account() {
        let val = fixture("session.json");
        let session: Session = serde_json::from_value(val).expect("session.json must deserialize");

        assert_eq!(session.chat_account_id(), Some("account1"));
    }

    // Oracle: draft-atwood-jmap-chat-00 §3 — chat_capability() parses the
    // account-level chat capability fields from the fixture.
    #[test]
    fn session_chat_capability_parses_account_capability() {
        let val = fixture("session.json");
        let session: Session = serde_json::from_value(val).expect("session.json must deserialize");

        let cap = session
            .chat_capability("account1")
            .expect("chat_capability must not return Err")
            .expect("account1 must have chat capability");

        assert_eq!(cap.max_body_bytes, 65536);
        assert_eq!(cap.max_attachment_bytes, 10485760);
        assert_eq!(cap.max_attachments_per_message, 10);
        assert_eq!(cap.max_group_members, 100);
        assert_eq!(cap.max_space_members, 500);
        assert_eq!(cap.max_roles_per_space, 50);
        assert_eq!(cap.max_channels_per_space, 200);
        assert_eq!(cap.max_categories_per_space, 25);
        assert_eq!(
            cap.supported_body_types,
            vec!["text/plain", "text/markdown"]
        );
        assert!(cap.supports_threads);
    }

    // Oracle: draft-atwood-jmap-chat-00 §3 — chat_capability() returns Ok(None)
    // when the account exists but lacks the chat capability key.
    #[test]
    fn session_chat_capability_absent_key_returns_ok_none() {
        let val = fixture("session.json");
        let mut session: Session =
            serde_json::from_value(val).expect("session.json must deserialize");

        session
            .accounts
            .get_mut("account1")
            .unwrap()
            .account_capabilities
            .remove("urn:ietf:params:jmap:chat");

        let result = session.chat_capability("account1");
        assert!(
            matches!(result, Ok(None)),
            "expected Ok(None), got {result:?}"
        );
    }

    // Oracle: session_malformed_chat_cap.json — hand-written fixture with
    // maxBodyBytes set to a string instead of a u64, derived from the spec
    // field type (draft-atwood-jmap-chat-00 §3); NOT produced by the code
    // under test.
    #[test]
    fn session_chat_capability_malformed_returns_err() {
        let val = fixture("session_malformed_chat_cap.json");
        let session: Session =
            serde_json::from_value(val).expect("fixture must deserialize as Session");

        let result = session.chat_capability("account1");
        match result {
            Err(crate::error::ClientError::Parse(msg)) => {
                assert!(
                    msg.contains("malformed chat capability"),
                    "error message should mention 'malformed chat capability', got: {msg}"
                );
            }
            other => panic!("expected Err(ClientError::Parse(...)), got {other:?}"),
        }
    }

    // Oracle: RFC 8620 §2 — chat_account_id() returns None when the capability
    // URI is absent from primaryAccounts.
    #[test]
    fn session_chat_account_id_absent_returns_none() {
        let val = fixture("session.json");
        let mut session: Session =
            serde_json::from_value(val).expect("session.json must deserialize");

        session.primary_accounts.remove("urn:ietf:params:jmap:chat");
        assert!(session.chat_account_id().is_none());
    }
}
