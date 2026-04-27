use super::{ChangesResponse, ChatQueryInput, GetResponse, QueryResponse, CALL_ID};

impl crate::client::JmapChatClient {
    /// Fetch Chat objects by IDs (RFC 8620 §5.1 / JMAP Chat §5 Chat/get).
    ///
    /// If `ids` is `None`, the server returns all Chats for the account.
    /// Pass `properties: None` to return all fields.
    pub async fn chat_get(
        &self,
        session: &crate::jmap::Session,
        ids: Option<&[&str]>,
        properties: Option<&[&str]>,
    ) -> Result<GetResponse<crate::types::Chat>, crate::error::ClientError> {
        let (api_url, account_id) = Self::session_parts(session)?;
        let args = serde_json::json!({
            "accountId": account_id,
            "ids": ids,
            "properties": properties,
        });
        let req = super::build_request("Chat/get", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(resp, CALL_ID)
    }

    /// Query Chat IDs with optional filter (RFC 8620 §5.5 / JMAP Chat §5 Chat/query).
    ///
    /// Only keys that are `Some` in `input` are included in the filter object;
    /// an empty filter object is sent as JSON `null`.
    pub async fn chat_query(
        &self,
        session: &crate::jmap::Session,
        input: &ChatQueryInput,
    ) -> Result<QueryResponse, crate::error::ClientError> {
        let (api_url, account_id) = Self::session_parts(session)?;
        let mut filter = serde_json::Map::new();
        if let Some(ref k) = input.filter_kind {
            let kind_str = serde_json::to_value(k).map_err(crate::error::ClientError::Serialize)?;
            filter.insert("kind".into(), kind_str);
        }
        if let Some(m) = input.filter_muted {
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
            "position": input.position,
            "limit": input.limit,
        });
        let req = super::build_request("Chat/query", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(resp, CALL_ID)
    }

    /// Fetch changes to Chat objects since `since_state` (RFC 8620 §5.2 / Chat/changes).
    ///
    /// If `has_more_changes` is true in the response, call again with `new_state`
    /// as `since_state` until the flag is false.
    pub async fn chat_changes(
        &self,
        session: &crate::jmap::Session,
        since_state: &str,
        max_changes: Option<u64>,
    ) -> Result<ChangesResponse, crate::error::ClientError> {
        let (api_url, account_id) = Self::session_parts(session)?;
        let args = serde_json::json!({
            "accountId": account_id,
            "sinceState": since_state,
            "maxChanges": max_changes,
        });
        let req = super::build_request("Chat/changes", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(resp, CALL_ID)
    }
}
