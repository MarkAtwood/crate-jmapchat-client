use super::{
    ChangesResponse, ChatContactQueryInput, ChatContactSetInput, GetResponse, QueryChangesResponse,
    QueryResponse, SetResponse,
};

impl crate::client::JmapChatClient {
    /// Fetch ChatContact objects by IDs (JMAP Chat §5 ChatContact/get).
    ///
    /// If `ids` is `None`, returns all ChatContacts for the account.
    pub async fn chat_contact_get(
        &self,
        session: &crate::jmap::Session,
        ids: Option<&[&str]>,
        properties: Option<&[&str]>,
    ) -> Result<GetResponse<crate::types::ChatContact>, crate::error::ClientError> {
        let (api_url, account_id) = Self::session_parts(session)?;
        let args = serde_json::json!({
            "accountId": account_id,
            "ids": ids,
            "properties": properties,
        });
        let (call_id, req) = super::build_request("ChatContact/get", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(resp, call_id)
    }

    /// Fetch changes to ChatContact objects since `since_state` (RFC 8620 §5.2).
    pub async fn chat_contact_changes(
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
        let (call_id, req) = super::build_request("ChatContact/changes", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(resp, call_id)
    }

    /// Update ChatContact properties (JMAP Chat §ChatContact/set).
    ///
    /// Supports `blocked` (Boolean) and `displayName` (nullable String).
    /// Create and destroy are not supported by spec; the server returns `forbidden`.
    pub async fn chat_contact_set(
        &self,
        session: &crate::jmap::Session,
        input: &ChatContactSetInput<'_>,
    ) -> Result<SetResponse, crate::error::ClientError> {
        let (api_url, account_id) = Self::session_parts(session)?;
        let mut patch = serde_json::Map::new();
        if let Some(b) = input.blocked {
            patch.insert("blocked".into(), b.into());
        }
        if let Some(dn) = &input.display_name {
            patch.insert(
                "displayName".into(),
                dn.map(serde_json::Value::from)
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        let args = serde_json::json!({
            "accountId": account_id,
            "update": { input.id: serde_json::Value::Object(patch) },
        });
        let (call_id, req) = super::build_request("ChatContact/set", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(resp, call_id)
    }

    /// Query ChatContact IDs with optional filter (JMAP Chat §ChatContact/query).
    ///
    /// Supported filter keys: `blocked`, `presence`. Supported sort properties:
    /// `"lastSeenAt"`, `"login"`, `"lastActiveAt"`.
    pub async fn chat_contact_query(
        &self,
        session: &crate::jmap::Session,
        input: &ChatContactQueryInput<'_>,
    ) -> Result<QueryResponse, crate::error::ClientError> {
        let (api_url, account_id) = Self::session_parts(session)?;
        let mut filter = serde_json::Map::new();
        if let Some(b) = input.filter_blocked {
            filter.insert("blocked".into(), b.into());
        }
        if let Some(p) = input.filter_presence {
            filter.insert("presence".into(), p.into());
        }
        let filter_val = if filter.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::Value::Object(filter)
        };
        let mut args = serde_json::json!({
            "accountId": account_id,
            "filter": filter_val,
        });
        if let Some(sp) = input.sort_property {
            args["sort"] = serde_json::json!([{
                "property": sp,
                "isAscending": input.sort_ascending.unwrap_or(false),
            }]);
        }
        if let Some(p) = input.position {
            args["position"] = p.into();
        }
        if let Some(l) = input.limit {
            args["limit"] = l.into();
        }
        let (call_id, req) = super::build_request("ChatContact/query", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(resp, call_id)
    }

    /// Fetch query-result changes for ChatContact since `since_query_state`
    /// (RFC 8620 §5.6 / ChatContact/queryChanges).
    pub async fn chat_contact_query_changes(
        &self,
        session: &crate::jmap::Session,
        since_query_state: &str,
        max_changes: Option<u64>,
    ) -> Result<QueryChangesResponse, crate::error::ClientError> {
        let (api_url, account_id) = Self::session_parts(session)?;
        let mut args = serde_json::json!({
            "accountId": account_id,
            "sinceQueryState": since_query_state,
        });
        if let Some(mc) = max_changes {
            args["maxChanges"] = mc.into();
        }
        let (call_id, req) = super::build_request("ChatContact/queryChanges", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(resp, call_id)
    }
}
