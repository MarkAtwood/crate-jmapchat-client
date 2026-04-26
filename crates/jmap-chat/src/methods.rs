// Typed JMAP Chat method wrappers (Step 8)
//
// Response types mirror RFC 8620 standard shapes (§5.1 /get, §5.5 /query,
// §5.2 /changes, §5.3 /set). Method implementations live on JmapChatClient
// and are the primary public API for callers that already hold a Session.

use std::collections::HashMap;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// RFC 8620 §5.1 — /get response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetResponse<T> {
    pub account_id: String,
    pub state: String,
    pub list: Vec<T>,
    pub not_found: Option<Vec<String>>,
}

/// RFC 8620 §5.5 — /query response.
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

/// RFC 8620 §5.2 — /changes response.
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

/// RFC 8620 §5.3 — /set response.
///
/// Used for both create (`message_create`) and update (`read_position_set`)
/// operations. Only the fields relevant to those two operations are modelled
/// here; `destroy` is deferred to Phase 4.
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

/// A /set operation failure for a single object (RFC 8620 §5.3).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Method implementations on JmapChatClient
// ---------------------------------------------------------------------------

impl crate::client::JmapChatClient {
    /// Fetch Chat objects by IDs (RFC 8620 §5.1 / JMAP Chat §5 Chat/get).
    ///
    /// If `ids` is `None`, the server returns all Chats for the account.
    /// Pass `properties: None` to return all fields.
    pub async fn chat_get(
        &self,
        api_url: &str,
        account_id: &str,
        ids: Option<&[&str]>,
        properties: Option<&[&str]>,
    ) -> Result<GetResponse<crate::types::Chat>, crate::error::ClientError> {
        let args = serde_json::json!({
            "accountId": account_id,
            "ids": ids,
            "properties": properties,
        });
        let req = build_request("Chat/get", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Query Chat IDs with optional filter (RFC 8620 §5.5 / JMAP Chat §5 Chat/query).
    ///
    /// `filter_kind`: `Some("direct")`, `Some("group")`, `Some("channel")`, or `None`.
    /// `filter_muted`: filter by mute state when `Some`.
    /// Only keys that are `Some` are included in the filter object; an empty
    /// filter object is sent as JSON `null`.
    pub async fn chat_query(
        &self,
        api_url: &str,
        account_id: &str,
        filter_kind: Option<&str>,
        filter_muted: Option<bool>,
        position: Option<u64>,
        limit: Option<u64>,
    ) -> Result<QueryResponse, crate::error::ClientError> {
        let mut filter = serde_json::Map::new();
        if let Some(k) = filter_kind {
            filter.insert("kind".into(), k.into());
        }
        if let Some(m) = filter_muted {
            filter.insert("muted".into(), m.into());
        }
        let filter_val = if filter.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::Value::Object(filter)
        };
        let args = serde_json::json!({
            "accountId": account_id,
            "filter": filter_val,
            "position": position,
            "limit": limit,
        });
        let req = build_request("Chat/query", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Fetch changes to Chat objects since `since_state` (RFC 8620 §5.2 / Chat/changes).
    ///
    /// If `has_more_changes` is true in the response, call again with `new_state`
    /// as `since_state` until the flag is false.
    pub async fn chat_changes(
        &self,
        api_url: &str,
        account_id: &str,
        since_state: &str,
        max_changes: Option<u64>,
    ) -> Result<ChangesResponse, crate::error::ClientError> {
        let args = serde_json::json!({
            "accountId": account_id,
            "sinceState": since_state,
            "maxChanges": max_changes,
        });
        let req = build_request("Chat/changes", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Fetch Message objects by IDs (RFC 8620 §5.1 / JMAP Chat §5 Message/get).
    ///
    /// `ids` is required (non-empty); fetching all messages is impractical.
    /// Pass `properties: None` to return all fields.
    pub async fn message_get(
        &self,
        api_url: &str,
        account_id: &str,
        ids: &[&str],
        properties: Option<&[&str]>,
    ) -> Result<GetResponse<crate::types::Message>, crate::error::ClientError> {
        let args = serde_json::json!({
            "accountId": account_id,
            "ids": ids,
            "properties": properties,
        });
        let req = build_request("Message/get", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Query Message IDs within a Chat (RFC 8620 §5.5 / JMAP Chat §5 Message/query).
    ///
    /// Per spec, either `chat_id` or `has_mention: Some(true)` must be provided.
    /// Servers MUST return `unsupportedFilter` if neither condition holds.
    #[allow(clippy::too_many_arguments)]
    pub async fn message_query(
        &self,
        api_url: &str,
        account_id: &str,
        chat_id: Option<&str>,
        has_mention: Option<bool>,
        has_attachment: Option<bool>,
        position: Option<u64>,
        limit: Option<u64>,
    ) -> Result<QueryResponse, crate::error::ClientError> {
        if chat_id.is_none() && has_mention.is_none() {
            return Err(crate::error::ClientError::Parse(
                "message_query: at least one of chat_id or has_mention must be provided".into(),
            ));
        }
        let mut filter = serde_json::Map::new();
        if let Some(id) = chat_id {
            filter.insert("chatId".into(), id.into());
        }
        if let Some(m) = has_mention {
            filter.insert("hasMention".into(), m.into());
        }
        if let Some(a) = has_attachment {
            filter.insert("hasAttachment".into(), a.into());
        }
        let filter_val = if filter.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::Value::Object(filter)
        };
        let args = serde_json::json!({
            "accountId": account_id,
            "filter": filter_val,
            "position": position,
            "limit": limit,
        });
        let req = build_request("Message/query", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Fetch changes to Message objects since `since_state` (RFC 8620 §5.2 / Message/changes).
    pub async fn message_changes(
        &self,
        api_url: &str,
        account_id: &str,
        since_state: &str,
        max_changes: Option<u64>,
    ) -> Result<ChangesResponse, crate::error::ClientError> {
        let args = serde_json::json!({
            "accountId": account_id,
            "sinceState": since_state,
            "maxChanges": max_changes,
        });
        let req = build_request("Message/changes", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Create (send) a new Message (RFC 8620 §5.3 / JMAP Chat §5 Message/set).
    ///
    /// `client_id` is a caller-supplied ULID used as the creation key. The server
    /// maps it to the server-assigned Message id in `SetResponse.created`.
    /// Only the `create` operation is implemented here; update/destroy are Phase 4.
    #[allow(clippy::too_many_arguments)]
    pub async fn message_create(
        &self,
        api_url: &str,
        account_id: &str,
        client_id: &str,
        chat_id: &str,
        body: &str,
        body_type: &str,
        sent_at: &str,
        reply_to: Option<&str>,
    ) -> Result<SetResponse, crate::error::ClientError> {
        let mut create_obj = serde_json::json!({
            "chatId": chat_id,
            "body": body,
            "bodyType": body_type,
            "sentAt": sent_at,
        });
        if let Some(rt) = reply_to {
            create_obj["replyTo"] = rt.into();
        }
        let args = serde_json::json!({
            "accountId": account_id,
            "create": { client_id: create_obj },
        });
        let req = build_request("Message/set", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Fetch ChatContact objects by IDs (JMAP Chat §5 ChatContact/get).
    ///
    /// If `ids` is `None`, returns all ChatContacts for the account.
    pub async fn chat_contact_get(
        &self,
        api_url: &str,
        account_id: &str,
        ids: Option<&[&str]>,
        properties: Option<&[&str]>,
    ) -> Result<GetResponse<crate::types::ChatContact>, crate::error::ClientError> {
        let args = serde_json::json!({
            "accountId": account_id,
            "ids": ids,
            "properties": properties,
        });
        let req = build_request("ChatContact/get", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Fetch ReadPosition objects by IDs (JMAP Chat §5 ReadPosition/get).
    ///
    /// If `ids` is `None`, returns all ReadPosition records for the account.
    /// The server creates one ReadPosition per Chat automatically.
    pub async fn read_position_get(
        &self,
        api_url: &str,
        account_id: &str,
        ids: Option<&[&str]>,
    ) -> Result<GetResponse<crate::types::ReadPosition>, crate::error::ClientError> {
        let args = serde_json::json!({
            "accountId": account_id,
            "ids": ids,
        });
        let req = build_request("ReadPosition/get", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Update the read position for a Chat (JMAP Chat §5 ReadPosition/set).
    ///
    /// `read_position_id` is the server-assigned ReadPosition.id (from
    /// `read_position_get`). `last_read_message_id` is the Message.id of the
    /// most recent message read. The server updates `lastReadAt` and
    /// recomputes `Chat.unreadCount`.
    ///
    /// `create` and `destroy` are forbidden by the spec; only `update` is issued.
    pub async fn read_position_set(
        &self,
        api_url: &str,
        account_id: &str,
        read_position_id: &str,
        last_read_message_id: &str,
    ) -> Result<SetResponse, crate::error::ClientError> {
        let args = serde_json::json!({
            "accountId": account_id,
            "update": {
                read_position_id: { "lastReadMessageId": last_read_message_id }
            },
        });
        let req = build_request("ReadPosition/set", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }

    /// Fetch the singleton PresenceStatus record (JMAP Chat §5 PresenceStatus/get).
    ///
    /// Per spec there is exactly one PresenceStatus per account; `ids: null`
    /// retrieves it.
    pub async fn presence_status_get(
        &self,
        api_url: &str,
        account_id: &str,
    ) -> Result<GetResponse<crate::types::PresenceStatus>, crate::error::ClientError> {
        let args = serde_json::json!({
            "accountId": account_id,
            "ids": serde_json::Value::Null,
        });
        let req = build_request("PresenceStatus/get", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(&resp, "r1")
    }
}

// ---------------------------------------------------------------------------
// Internal helper
// ---------------------------------------------------------------------------

// Each request contains exactly one method call, identified by call-id "r1".
// extract_response() relies on this. Do not add multi-call batching here
// without updating extract_response() to accept a call-id parameter.
fn build_request(method_name: &str, args: serde_json::Value) -> crate::jmap::JmapRequest {
    crate::jmap::JmapRequest {
        using: chat_using().clone(),
        method_calls: vec![(method_name.to_string(), args, "r1".to_string())],
    }
}

fn chat_using() -> &'static Vec<String> {
    static CHAT_USING_VEC: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    CHAT_USING_VEC.get_or_init(|| {
        vec![
            "urn:ietf:params:jmap:core".to_string(),
            "urn:ietf:params:jmap:chat".to_string(),
        ]
    })
}
