# Research Report: Steps 5–8 of jmap-chat Library Crate

Scope: `crates/jmap-chat/src/client.rs` (Steps 5–7) and
`crates/jmap-chat/src/methods.rs` (Step 8).

---

## A. Session Struct Fields (from spec §2 and §3)

The Session object is defined in RFC 8620 §2 as the top-level object returned
by `GET /.well-known/jmap`. The JMAP Chat spec (draft-atwood-jmap-chat-00 §3)
adds extension fields.

### RFC 8620 §2 Core Fields

| JSON name | Rust type | Optional | Notes |
|---|---|---|---|
| `capabilities` | `HashMap<String, serde_json::Value>` | required | Includes `"urn:ietf:params:jmap:core"` and `"urn:ietf:params:jmap:chat"` |
| `accounts` | `HashMap<String, AccountInfo>` | required | Map of accountId → AccountInfo |
| `primaryAccounts` | `HashMap<String, String>` | required | capability URI → primary accountId |
| `username` | `String` | required | Human-readable username |
| `apiUrl` | `String` | required | URL for JMAP API POST requests |
| `downloadUrl` | `String` | required | URL template for blob download |
| `uploadUrl` | `String` | required | URL for blob upload |
| `eventSourceUrl` | `String` | required | URL for SSE stream |
| `state` | `String` | required | Opaque session state token |

### JMAP Chat §3 Extension Fields (on the Session object)

| JSON name | Rust type | Optional | Notes |
|---|---|---|---|
| `ownerUserId` | `String` | optional | The mailbox owner's ChatContact.id |
| `ownerLogin` | `String` | optional | Human-readable login name for the owner |
| `ownerEndpoints` | `Vec<Endpoint>` | optional | Owner's out-of-band capability endpoints |

### Account-Level Chat Capability Fields (inside `accounts[id].accountCapabilities["urn:ietf:params:jmap:chat"]`)

These appear inside each `AccountInfo` struct, not at the top level of Session.

| JSON name | Rust type | Notes |
|---|---|---|
| `maxBodyBytes` | `u64` | Maximum UTF-8 byte length of a Message body |
| `maxAttachmentBytes` | `u64` | Maximum single attachment blob size |
| `maxAttachmentsPerMessage` | `u64` | |
| `maxGroupMembers` | `u64` | |
| `maxSpaceMembers` | `u64` | |
| `maxRolesPerSpace` | `u64` | |
| `maxChannelsPerSpace` | `u64` | |
| `maxCategoriesPerSpace` | `u64` | |
| `supportedBodyTypes` | `Vec<String>` | MIME types accepted in `bodyType`; always includes `"text/plain"` |
| `supportsThreads` | `bool` | Whether the server supports the optional thread model |

### Rust Struct Definitions

```rust
// In crates/jmap-chat/src/types.rs (or a new session.rs module)

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub capabilities: HashMap<String, serde_json::Value>,
    pub accounts: HashMap<String, AccountInfo>,
    pub primary_accounts: HashMap<String, String>,
    pub username: String,
    pub api_url: String,
    pub download_url: String,
    pub upload_url: String,
    pub event_source_url: String,
    pub state: String,

    // JMAP Chat extensions (§3)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_endpoints: Option<Vec<Endpoint>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub name: String,
    pub is_personal: bool,
    pub is_read_only: bool,
    pub account_capabilities: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCapability {
    pub max_body_bytes: u64,
    pub max_attachment_bytes: u64,
    pub max_attachments_per_message: u64,
    pub max_group_members: u64,
    pub max_space_members: u64,
    pub max_roles_per_space: u64,
    pub max_channels_per_space: u64,
    pub max_categories_per_space: u64,
    pub supported_body_types: Vec<String>,
    pub supports_threads: bool,
}
```

### Helper on Session

```rust
impl Session {
    /// Returns the primary accountId for the JMAP Chat capability.
    pub fn chat_account_id(&self) -> Option<&str> {
        self.primary_accounts
            .get("urn:ietf:params:jmap:chat")
            .map(String::as_str)
    }

    /// Returns the parsed ChatCapability for the given account, if present.
    pub fn chat_capability(&self, account_id: &str) -> Option<ChatCapability> {
        let account = self.accounts.get(account_id)?;
        let raw = account.account_capabilities
            .get("urn:ietf:params:jmap:chat")?;
        serde_json::from_value(raw.clone()).ok()
    }
}
```

---

## B. fetch_session() Signature and Algorithm

### Exact Function Signature

```rust
/// Fetch the JMAP Session object from `{base_url}/.well-known/jmap`.
///
/// The `base_url` must NOT include a trailing slash or a path component.
/// Example: `"https://100.64.1.1:8008"`.
///
/// Auth headers from the `AuthProvider` are injected on the request.
pub async fn fetch_session(
    client: &reqwest::Client,
    auth: &dyn AuthProvider,
    base_url: &str,
) -> Result<Session, ClientError>
```

Note: the `reqwest::Client` has already been constructed by `auth.build_client()`.
The client is passed in separately so callers can reuse it across `call()` and
`subscribe_events()` without rebuilding. The `auth` parameter is for the
per-request header only.

Alternatively, `JmapChatClient` can hold both and the function becomes a method:

```rust
impl JmapChatClient {
    pub async fn fetch_session(&self) -> Result<Session, ClientError>
}
```

### Step-by-Step Algorithm

1. Construct URL: `format!("{base_url}/.well-known/jmap")`.
2. Build the request: `client.get(&url)`.
3. Inject auth header: if `auth.auth_header()` returns `Some((name, value))`, call
   `.header(name, value)` on the request builder.
4. Send: `.send().await?` — maps `reqwest::Error` → `ClientError`.
5. Check HTTP status: call `.error_for_status()?` — maps 4xx/5xx → `ClientError`.
   Special-case 401/403 as `ClientError::AuthFailed(status)` before calling
   `error_for_status()`, so the caller can distinguish auth failures that will not
   resolve on retry.
6. Parse JSON: `.json::<Session>().await` — maps parse error →
   `ClientError::Parse(String)`.
7. Validate minimum structure: confirm `session.api_url` is non-empty,
   `session.event_source_url` is non-empty. Return
   `ClientError::InvalidSession("field missing")` on failure.
8. Return `Ok(session)`.

### Error Cases

| Condition | Error variant |
|---|---|
| Network error (connection refused, timeout) | `ClientError::Http(reqwest::Error)` |
| HTTP 401 or 403 | `ClientError::AuthFailed(u16)` |
| HTTP 4xx or 5xx (other) | `ClientError::Http(reqwest::Error)` |
| Response body is not valid JSON | `ClientError::Parse(String)` |
| Response body is valid JSON but not a Session | `ClientError::Parse(String)` |
| `api_url` or `event_source_url` is empty | `ClientError::InvalidSession(&'static str)` |

---

## C. call() Signature and Algorithm

### Exact Function Signature

```rust
/// POST a JmapRequest to the session's `apiUrl` and return the parsed JmapResponse.
///
/// `account_id` should come from `session.chat_account_id()`.
/// `session` is passed to extract `api_url` and (optionally) the session state
/// for `sessionState` validation.
pub async fn call(
    client: &reqwest::Client,
    auth: &dyn AuthProvider,
    api_url: &str,
    req: &JmapRequest,
) -> Result<JmapResponse, ClientError>
```

As a `JmapChatClient` method:

```rust
pub async fn call(&self, req: &JmapRequest) -> Result<JmapResponse, ClientError>
```

### Step-by-Step Algorithm

1. Build the request: `client.post(api_url)`.
2. Inject auth header: same pattern as `fetch_session`.
3. Set JSON body: `.json(req)` — serializes `JmapRequest` to JSON and sets
   `Content-Type: application/json`.
4. Set timeout: `.timeout(Duration::from_secs(30))` — prevents indefinite stall.
5. Send: `.send().await?`.
6. Check HTTP status: special-case 401/403 → `ClientError::AuthFailed(u16)`,
   then `error_for_status()?`.
7. Parse JSON: `.json::<JmapResponse>().await` — maps error → `ClientError::Parse`.
8. Validate: `resp.method_responses` should be non-empty if the request had
   method calls. Log a warning (not an error) if it is empty; return `Ok`.
9. Return `Ok(resp)`.

### Error Cases

Same as `fetch_session` plus:

| Condition | Error variant |
|---|---|
| JSON serialization of `JmapRequest` fails | `ClientError::Serialize(serde_json::Error)` |
| Server returns `methodError` in response body | These are returned as `Ok(JmapResponse)` — callers inspect `method_responses` for `error` invocations. |

---

## D. subscribe_events() / SSE Pattern

### How SSE Works in reqwest

The `stream` feature is already enabled in `Cargo.toml` (confirmed in workspace
`Cargo.toml`: `reqwest = { version = "0.12", features = ["json", "stream", ...] }`).

The key API chain:

```rust
use futures::StreamExt;

let resp = client
    .get(event_source_url)
    .header("Accept", "text/event-stream")
    .send()
    .await?;

// resp.bytes_stream() returns impl Stream<Item = Result<Bytes, reqwest::Error>>
let mut stream = resp.bytes_stream();

while let Some(chunk) = stream.next().await {
    let bytes = chunk?;
    // accumulate bytes into a String buffer, split on "\n\n"
}
```

`bytes_stream()` is defined on `reqwest::Response`. It requires the `stream`
feature. It returns `impl futures_core::Stream<Item = crate::Result<Bytes>>`.
Call `.next().await` to get the next chunk. Each chunk is raw bytes; there is no
built-in line splitter.

### StateChange Event Format (from spec §7 Push Notifications)

The spec defines three SSE event types:

**state** — triggers `/changes` calls:
```
event: state
data: {"@type":"StateChange","changed":{"<accountId>":{"Message":"<state>","Chat":"<state>","ChatContact":"<state>"}}}
```

**typing** — not stored, no state token:
```
event: typing
data: {"chatId":"<id>","senderId":"<contact-id>","typing":<bool>}
```

**presence** — not stored:
```
event: presence
data: {"contactId":"<id>","presence":"<state>","lastActiveAt":"<ts>","statusText":"<string>|null","statusEmoji":"<string>|null"}
```

The kith implementation notes that the wire format for the `state` event also
includes an `id:` field for `Last-Event-ID` resumption. The `@type` field is
present in the spec example but the kith implementation parses only `changed`.
For interoperability, the Rust parser should accept both shapes (with and without
`@type`).

### Rust Data Types for SSE Events

```rust
/// Parsed SSE event.
#[derive(Debug)]
pub enum SseEvent {
    /// A "state" event: maps type name → new state token, per account.
    StateChange {
        /// accountId → (typeName → newState)
        changed: HashMap<String, HashMap<String, String>>,
    },
    /// A "typing" indicator event.
    Typing {
        chat_id: String,
        sender_id: String,
        typing: bool,
    },
    /// A "presence" update event.
    Presence {
        contact_id: String,
        presence: String,
        last_active_at: Option<String>,
        status_text: Option<String>,
        status_emoji: Option<String>,
    },
    /// An unrecognized or keepalive event (silently ignored).
    Unknown,
}
```

### subscribe_events() Signature

```rust
/// Open an SSE connection to `event_source_url` and return a stream of parsed events.
///
/// The returned stream ends when the HTTP connection closes. Callers are
/// responsible for reconnection; see the reconnect strategy below.
pub async fn subscribe_events(
    client: &reqwest::Client,
    auth: &dyn AuthProvider,
    event_source_url: &str,
    last_event_id: Option<&str>,
) -> Result<impl futures_core::Stream<Item = Result<SseEvent, ClientError>>, ClientError>
```

Or, returning a channel-based background task (matching the kith pattern):

```rust
pub fn spawn_event_stream(
    client: reqwest::Client,
    auth: Arc<dyn AuthProvider>,
    event_source_url: String,
) -> (mpsc::Receiver<SseEvent>, mpsc::Receiver<StreamStatus>, JoinHandle<()>)
```

The channel-based approach (matching kith's `spawn_sse`) is preferred for the
TUI use case because it decouples the SSE background task from the event loop
and handles reconnection internally.

### SSE Parsing Algorithm (run_sse inner loop)

1. Build GET request to `event_source_url`.
2. Add `Accept: text/event-stream` header.
3. If `last_event_id` is `Some`, add `Last-Event-ID: <value>` header.
4. Inject auth header from `auth.auth_header()`.
5. Send and check status (special-case 401/403).
6. Signal `StreamStatus::Connected` on the status channel.
7. Call `resp.bytes_stream()` to get the byte stream.
8. Maintain a `String` buffer `buf`.
9. On each chunk:
   a. Append `String::from_utf8_lossy(&bytes)` to `buf`.
   b. Normalize line endings in the full `buf`: `buf.replace("\r\n", "\n").replace('\r', "\n")`.
      (Must normalize the full buffer, not per-chunk, to handle CRLF split across chunk boundaries.)
   c. Guard against unbounded buffer growth: if `buf.len() > MAX_BUF` (e.g., 1 MiB),
      return `Err(ClientError::SseFrameTooLarge)`.
   d. While `buf` contains `"\n\n"`: extract the block before the first `"\n\n"`,
      drain it from `buf` (including the two newlines), parse the block.
10. For each parsed block:
    a. Parse lines: `event:`, `data:` (multiple data lines joined with `\n`), `id:`.
    b. If `event: state`, parse `data` as `StateChange` JSON and emit.
    c. If `event: typing`, parse and emit.
    d. If `event: presence`, parse and emit.
    e. Otherwise, ignore silently.
    f. If `id:` is present, update `last_event_id` for the next reconnect.
11. When the stream ends (server EOF), return `Ok(last_event_id)`.
12. On error, return `Err(...)`.

### Reconnect Strategy

Matching the kith pattern (and RFC 8620 §7.3 recommendation):

- Exponential backoff: start at 2 seconds, double on each failure, cap at 60 seconds.
- Reset backoff to 2 seconds on clean close (server EOF is normal).
- On 401/403: stop retrying, surface `StreamStatus::AuthFailed(code)`.
- Track `last_event_id` across reconnect attempts; send `Last-Event-ID` header
  on reconnect to allow server-side resumption.
- Exit the reconnect loop only when the receiver channel is dropped.

```
MAX_SSE_BUF: usize = 1024 * 1024  // 1 MiB
INITIAL_BACKOFF_SECS: u64 = 2
MAX_BACKOFF_SECS: u64 = 60
```

---

## E. Typed Method Wrapper Pattern

### Pattern Overview

Each typed method wrapper:
1. Takes typed Rust arguments (account_id, specific filter/sort options).
2. Constructs a `JmapRequest` with the appropriate `using` array and one
   `Invocation` in `method_calls`.
3. Calls `client.call(&req).await?`.
4. Finds the matching method response in `resp.method_responses` by call_id or
   by method name at index 0.
5. Deserializes the response arguments (`serde_json::Value`) into the typed
   response struct.
6. Returns the typed result.

### Call ID Convention

Use a fixed call id (e.g., `"r1"`) for single-method requests since all
wrappers make one method call per `JmapRequest`. This keeps extraction simple:
find the first entry in `method_responses` where the call_id matches.

### Typed Response Structs (RFC 8620 standard shapes)

**`/get` response:**
```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetResponse<T> {
    pub account_id: String,
    pub state: String,
    pub list: Vec<T>,
    pub not_found: Option<Vec<String>>,
}
```

**`/query` response:**
```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    pub account_id: String,
    pub query_state: String,
    pub can_calculate_changes: bool,
    pub position: u64,
    pub ids: Vec<String>,
    pub total: Option<u64>,
    pub limit: Option<u64>,
}
```

**`/changes` response:**
```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangesResponse {
    pub account_id: String,
    pub old_state: String,
    pub new_state: String,
    pub has_more_changes: bool,
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub destroyed: Vec<String>,
}
```

**`/set` response (create only):**
```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetResponse {
    pub account_id: String,
    pub old_state: Option<String>,
    pub new_state: String,
    pub created: Option<HashMap<String, serde_json::Value>>,
    pub updated: Option<HashMap<String, serde_json::Value>>,
    pub destroyed: Option<Vec<String>>,
    pub not_created: Option<HashMap<String, SetError>>,
    pub not_updated: Option<HashMap<String, SetError>>,
    pub not_destroyed: Option<HashMap<String, SetError>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub description: Option<String>,
}
```

### Extraction Helper

```rust
fn extract_response<T: serde::de::DeserializeOwned>(
    resp: &JmapResponse,
    call_id: &str,
) -> Result<T, ClientError> {
    let inv = resp.method_responses
        .iter()
        .find(|(_, _, id)| id == call_id)
        .ok_or_else(|| ClientError::MethodNotFound(call_id.to_string()))?;

    // Check for JMAP method error
    if inv.0 == "error" {
        let desc = inv.1.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let err_type = inv.1.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("serverError")
            .to_string();
        return Err(ClientError::MethodError { error_type: err_type, description: desc });
    }

    serde_json::from_value(inv.1.clone())
        .map_err(|e| ClientError::Parse(e.to_string()))
}
```

---

## F. Method Signatures for All 11 Methods

All methods are `async fn` on `JmapChatClient` (or free functions taking
`client, auth, api_url, account_id`). All return `Result<_, ClientError>`.

The `using` array for all Chat methods is:
```rust
vec![
    "urn:ietf:params:jmap:core".to_string(),
    "urn:ietf:params:jmap:chat".to_string(),
]
```

---

### 1. Chat/get

```rust
/// Fetch Chat objects by IDs.
///
/// If `ids` is `None`, returns all Chats for the account (per RFC 8620 §5.1).
/// The `properties` parameter may be `None` to return all fields.
pub async fn chat_get(
    &self,
    account_id: &str,
    ids: Option<&[&str]>,
    properties: Option<&[&str]>,
) -> Result<GetResponse<Chat>, ClientError>
```

Request arguments:
```json
{
    "accountId": "<account_id>",
    "ids": <ids or null>,
    "properties": <properties or null>
}
```

---

### 2. Chat/query

```rust
/// Query Chat IDs with optional filter and sort.
///
/// `filter_kind` filters by Chat kind: `Some("direct")`, `Some("group")`,
/// `Some("channel")`, or `None` for all.
/// `filter_muted` filters by mute state.
pub async fn chat_query(
    &self,
    account_id: &str,
    filter_kind: Option<&str>,
    filter_muted: Option<bool>,
    position: Option<u64>,
    limit: Option<u64>,
) -> Result<QueryResponse, ClientError>
```

Filter object (only include keys that are `Some`):
```json
{
    "kind": "<value>",
    "muted": <bool>
}
```

---

### 3. Chat/changes

```rust
/// Fetch changes to Chat objects since `since_state`.
///
/// `max_changes` limits the number of changes returned per RFC 8620 §5.2.
/// If `has_more_changes` is true in the response, call again with `new_state`.
pub async fn chat_changes(
    &self,
    account_id: &str,
    since_state: &str,
    max_changes: Option<u64>,
) -> Result<ChangesResponse, ClientError>
```

---

### 4. Message/get

```rust
/// Fetch Message objects by IDs.
///
/// `ids` is required (unlike Chat/get where null means "all") because fetching
/// all messages is impractical. Pass a non-empty slice.
/// `properties` may be `None` to return all fields.
pub async fn message_get(
    &self,
    account_id: &str,
    ids: &[&str],
    properties: Option<&[&str]>,
) -> Result<GetResponse<Message>, ClientError>
```

---

### 5. Message/query

```rust
/// Query Message IDs within a Chat (or across all Chats for hasMention).
///
/// Either `chat_id` or `has_mention: true` must be provided per spec §5 Message/query.
/// The spec states: servers MUST return `unsupportedFilter` for requests that
/// omit `chatId` without also including `hasMention: true`.
///
/// Default sort is `receivedAt` ascending when `chat_id` is present.
pub async fn message_query(
    &self,
    account_id: &str,
    chat_id: Option<&str>,
    has_mention: Option<bool>,
    has_attachment: Option<bool>,
    position: Option<u64>,
    limit: Option<u64>,
) -> Result<QueryResponse, ClientError>
```

---

### 6. Message/changes

```rust
/// Fetch changes to Message objects since `since_state`.
pub async fn message_changes(
    &self,
    account_id: &str,
    since_state: &str,
    max_changes: Option<u64>,
) -> Result<ChangesResponse, ClientError>
```

---

### 7. Message/set (create only)

```rust
/// Create a new Message (send).
///
/// Only the `create` operation is supported in Step 8; update/destroy
/// are deferred to Phase 4.
///
/// `client_id` is a caller-supplied ULID used as the creation key in the
/// `create` map. The server maps it to the server-assigned Message id in
/// `SetResponse.created`.
pub async fn message_create(
    &self,
    account_id: &str,
    client_id: &str,
    chat_id: &str,
    body: &str,
    body_type: &str,
    sent_at: &str,
    reply_to: Option<&str>,
) -> Result<SetResponse, ClientError>
```

Request shape:
```json
{
    "accountId": "<account_id>",
    "create": {
        "<client_id>": {
            "chatId": "<chat_id>",
            "body": "<body>",
            "bodyType": "<body_type>",
            "sentAt": "<sent_at>",
            "replyTo": "<reply_to>"
        }
    }
}
```

---

### 8. ChatContact/get

```rust
/// Fetch ChatContact objects by IDs.
///
/// If `ids` is `None`, returns all ChatContacts for the account.
pub async fn chat_contact_get(
    &self,
    account_id: &str,
    ids: Option<&[&str]>,
    properties: Option<&[&str]>,
) -> Result<GetResponse<ChatContact>, ClientError>
```

---

### 9. ReadPosition/get

```rust
/// Fetch ReadPosition objects by IDs.
///
/// If `ids` is `None`, returns all ReadPosition records for the account.
/// The server creates one ReadPosition per Chat automatically.
pub async fn read_position_get(
    &self,
    account_id: &str,
    ids: Option<&[&str]>,
) -> Result<GetResponse<ReadPosition>, ClientError>
```

---

### 10. ReadPosition/set

```rust
/// Update the read position for a Chat by setting `lastReadMessageId`.
///
/// `read_position_id` is the server-assigned ReadPosition.id (obtained from
/// `read_position_get`). `last_read_message_id` is the Message.id of the most
/// recent message read.
///
/// The server sets `lastReadAt` and recomputes `Chat.unreadCount`.
pub async fn read_position_set(
    &self,
    account_id: &str,
    read_position_id: &str,
    last_read_message_id: &str,
) -> Result<SetResponse, ClientError>
```

Request shape:
```json
{
    "accountId": "<account_id>",
    "update": {
        "<read_position_id>": {
            "lastReadMessageId": "<last_read_message_id>"
        }
    }
}
```

Note: `create` and `destroy` are forbidden by the spec. This wrapper only
issues `update` patches.

---

### 11. PresenceStatus/get

```rust
/// Fetch the singleton PresenceStatus record for the account.
///
/// Per spec §5 PresenceStatus/get, there is exactly one PresenceStatus per
/// account. Pass `ids: None` to retrieve it.
pub async fn presence_status_get(
    &self,
    account_id: &str,
) -> Result<GetResponse<PresenceStatus>, ClientError>
```

---

## G. Security Invariants

### Untrusted Inputs

All data arriving from the server (Session fields, method responses, SSE event
data) is attacker-controlled. The connection may be MITMed unless mTLS or a
custom CA root is used.

Specific untrusted fields with required treatment:

| Field | Risk | Required treatment |
|---|---|---|
| `Session.api_url`, `event_source_url`, `download_url`, `upload_url` | SSRF; the server could redirect the client to internal addresses | Validate that URLs are non-empty strings. Do NOT follow redirects to non-HTTPS URLs. Consider validating URL scheme is `https://` (or `http://` only when explicitly configured for dev). |
| `Session.ownerEndpoints[*].uri`, `ChatContact.endpoints[*].uri`, `Message.actions[*].uri` | SSRF; arbitrary URI supplied by peer | Must not be fetched or acted upon automatically; only on explicit user action. |
| `Message.body` (plaintext types) | XSS in a future GUI, path injection if saved to disk | Treat as raw text; never execute or evaluate. |
| `Message.body` (encrypted types, `application/mls-ciphertext`) | Opaque bytes — do not attempt to display or decode | Display as `[encrypted]`. |
| `Attachment.filename` | Path traversal if saved to disk | The spec forbids `/`, `\`, and null bytes in filenames; reject on receipt if they are present. |
| `Attachment.size`, `Attachment.sha256` | May not match actual blob | Verify after download before presenting to user (deferred to Phase 5 blob work). |
| `Message.sentAt`, `Message.editedAt` | Can be set to any value by sender | Never use for ordering or expiry; display only. Order by `receivedAt`. |
| SSE `data:` field | Malformed JSON; oversized frame | Silently skip malformed events. Enforce `MAX_SSE_BUF` (1 MiB) to prevent unbounded buffer growth. |
| `JmapResponse.method_responses` | Unexpected method names, missing fields | Validate invocation shape: 3-element array, first element matches expected method or `"error"`. Use `serde_json::from_value` with proper error mapping rather than indexing blindly. |
| `StateChange.changed` keys (accountId, typeName) | Unknown type names | Ignore unknown type names silently; do not panic on unrecognized types. |

### What Must Never Be Logged

- Bearer token value (from `BearerAuth.token`).
- HTTP Basic password (from `BasicAuth.password`).
- DER cert bytes or private key bytes (from `CustomCaAuth.der_cert` or a future `ClientCertAuth`).
- The full `Authorization` header value returned by `auth.auth_header()`.
- Any `data:` line from an SSE stream that could contain message content (this
  is low risk in a terminal logger but should not appear in production log lines
  at info level or below).

### Validation Required at Client Side

Even though the server is authoritative, the client must guard against:

1. **Empty `api_url` or `event_source_url`**: return `ClientError::InvalidSession`.
2. **Non-HTTPS URLs in Session** (when not in dev mode): log a warning or return
   an error, depending on configuration.
3. **`method_responses` containing an `"error"` invocation**: parse the `type`
   and `description` and return `ClientError::MethodError`.
4. **`SetResponse.not_created` non-empty**: the message was rejected; surface
   the `SetError.type` and `SetError.description` to the caller.
5. **`ChangesResponse.has_more_changes: true`**: the caller must loop, calling
   `/changes` again with `new_state` as `since_state`, until `has_more_changes`
   is false.
6. **`ChangesResponse` with `cannotCalculateChanges` method error** (returned as
   an `"error"` invocation): fall back to a full `/get` call.

### No Credentials in Errors

`ClientError` variants that wrap `reqwest::Error` must not include the request
URL or headers in their `Display` output if those contain credentials. In
practice, `reqwest::Error`'s display does not include headers, only the URL
(which is safe). Passwords in URL userinfo (`http://user:pass@host`) must never
be constructed.

### Auth Header Lifetime

The `auth_header()` method returns a fresh `HeaderValue` on each call. The
`HeaderValue` must not be cloned into long-lived data structures or logged.
After the request is sent, the `HeaderValue` is consumed by reqwest and not
retained.
