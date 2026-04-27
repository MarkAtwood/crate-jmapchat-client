use super::{GetResponse, CALL_ID};

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
        let req = super::build_request("ChatContact/get", args);
        let resp = self.call(api_url, &req).await?;
        crate::client::extract_response(resp, CALL_ID)
    }
}
