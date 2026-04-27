// SSE types and frame parser for JMAP Chat push notifications.
// Spec: draft-atwood-jmap-chat-00 §7 (Push Notifications)
// Wire format: RFC 8895 (Server-Sent Events)

use std::collections::HashMap;

/// A parsed SSE frame: the event and the `id:` line value (if any).
#[derive(Debug)]
#[non_exhaustive]
pub struct SseFrame {
    pub event: SseEvent,
    pub id: Option<String>,
}

/// A parsed SSE event from the JMAP Chat event source.
///
/// Spec: draft-atwood-jmap-chat-00 §7 (Push Notifications)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SseEvent {
    /// A "state" event: maps accountId → (typeName → newState).
    ///
    /// Triggers a `/changes` call for each type listed. Wire format:
    /// `{"@type":"StateChange","changed":{"<accountId>":{"<TypeName>":"<state>"}}}`
    StateChange {
        changed: HashMap<String, HashMap<String, String>>,
    },
    /// A "typing" indicator event. Not persisted; no state token.
    Typing {
        chat_id: String,
        sender_id: String,
        typing: bool,
    },
    /// A "presence" update event. Not persisted.
    Presence {
        contact_id: String,
        presence: crate::types::ContactPresence,
        last_active_at: Option<String>,
        status_text: Option<String>,
        status_emoji: Option<String>,
    },
    /// Unrecognized event type, keepalive, or parse failure.
    ///
    /// Callers should silently ignore this variant.
    Unknown,
}

/// Parse a single SSE block (the text between two blank lines) into an [`SseFrame`].
///
/// Returns an [`SseFrame`] with `event = SseEvent::Unknown` for empty blocks,
/// keepalives, or unrecognized event types. Never panics. Malformed `data:`
/// JSON is silently ignored and returns `Unknown` rather than propagating an
/// error.
///
/// `SseFrame::id` carries the value of the `id:` line, if present. Callers
/// should track this and send it as `Last-Event-ID` on reconnect per RFC 8620
/// §7.3.
pub fn parse_sse_block(block: &str) -> SseFrame {
    let mut event_type: Option<&str> = None;
    let mut data_lines: Vec<&str> = Vec::new();
    let mut id: Option<String> = None;

    for line in block.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event_type = Some(value.trim());
        } else if let Some(value) = line.strip_prefix("data:") {
            data_lines.push(value.trim());
        } else if let Some(value) = line.strip_prefix("id:") {
            id = Some(value.trim().to_owned());
        }
        // Comments (lines starting with ':') and unknown fields are silently ignored.
    }

    let data = data_lines.join("\n");

    let event = match event_type {
        Some("state") => parse_state_data(&data),
        Some("typing") => parse_typing_data(&data),
        Some("presence") => parse_presence_data(&data),
        _ => SseEvent::Unknown,
    };

    SseFrame { event, id }
}

/// Parse the data payload of a "state" event.
///
/// Accepts both the bare `{"changed":{...}}` shape and the shape with
/// `"@type":"StateChange"` as the spec example includes it.
fn parse_state_data(data: &str) -> SseEvent {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(data) else {
        return SseEvent::Unknown;
    };
    let Some(changed_val) = v.get("changed") else {
        return SseEvent::Unknown;
    };
    let Ok(changed) =
        serde_json::from_value::<HashMap<String, HashMap<String, String>>>(changed_val.clone())
    else {
        return SseEvent::Unknown;
    };
    SseEvent::StateChange { changed }
}

// Wire format defined by JMAP Chat spec §5, ChatMessage/typing event.
#[derive(serde::Deserialize)]
struct TypingPayload {
    #[serde(rename = "chatId")]
    chat_id: String,
    #[serde(rename = "senderId")]
    sender_id: String,
    typing: bool,
}

// Wire format defined by JMAP Chat spec §5, ChatContact/presence event.
#[derive(serde::Deserialize)]
struct PresencePayload {
    #[serde(rename = "contactId")]
    contact_id: String,
    presence: crate::types::ContactPresence,
    #[serde(rename = "lastActiveAt")]
    last_active_at: Option<String>,
    #[serde(rename = "statusText")]
    status_text: Option<String>,
    #[serde(rename = "statusEmoji")]
    status_emoji: Option<String>,
}

/// Parse the data payload of a "typing" event.
fn parse_typing_data(data: &str) -> SseEvent {
    let Ok(p) = serde_json::from_str::<TypingPayload>(data) else {
        return SseEvent::Unknown;
    };
    SseEvent::Typing {
        chat_id: p.chat_id,
        sender_id: p.sender_id,
        typing: p.typing,
    }
}

/// Parse the data payload of a "presence" event.
fn parse_presence_data(data: &str) -> SseEvent {
    let Ok(p) = serde_json::from_str::<PresencePayload>(data) else {
        return SseEvent::Unknown;
    };
    SseEvent::Presence {
        contact_id: p.contact_id,
        presence: p.presence,
        last_active_at: p.last_active_at,
        status_text: p.status_text,
        status_emoji: p.status_emoji,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: spec §7 "state" event format.
    #[test]
    fn parse_state_event() {
        let block = "event: state\ndata: {\"changed\":{\"acc1\":{\"Message\":\"s42\"}}}";
        let SseFrame { event, .. } = parse_sse_block(block);
        match event {
            SseEvent::StateChange { changed } => {
                assert_eq!(
                    changed
                        .get("acc1")
                        .and_then(|m| m.get("Message"))
                        .map(String::as_str),
                    Some("s42"),
                    "changed[acc1][Message] must equal s42"
                );
            }
            other => panic!("expected StateChange, got {other:?}"),
        }
    }

    /// Oracle: spec §7 "state" event format — @type field is present.
    /// The @type field must be accepted and ignored; only "changed" matters.
    #[test]
    fn parse_state_event_with_type_field() {
        let block = "event: state\ndata: {\"@type\":\"StateChange\",\"changed\":{\"acc1\":{\"Message\":\"s42\"}}}";
        let SseFrame { event, .. } = parse_sse_block(block);
        match event {
            SseEvent::StateChange { changed } => {
                assert_eq!(
                    changed
                        .get("acc1")
                        .and_then(|m| m.get("Message"))
                        .map(String::as_str),
                    Some("s42"),
                    "changed[acc1][Message] must equal s42"
                );
            }
            other => panic!("expected StateChange, got {other:?}"),
        }
    }

    /// Oracle: spec §7 "typing" event format.
    #[test]
    fn parse_typing_event() {
        let block = "event: typing\ndata: {\"chatId\":\"c1\",\"senderId\":\"u1\",\"typing\":true}";
        let SseFrame { event, .. } = parse_sse_block(block);
        match event {
            SseEvent::Typing {
                chat_id,
                sender_id,
                typing,
            } => {
                assert_eq!(chat_id, "c1");
                assert_eq!(sender_id, "u1");
                assert!(typing, "typing must be true");
            }
            other => panic!("expected Typing, got {other:?}"),
        }
    }

    /// Oracle: spec §7 "presence" event format — all fields present.
    #[test]
    fn parse_presence_event() {
        let block = concat!(
            "event: presence\n",
            "data: {\"contactId\":\"ct1\",\"presence\":\"online\",",
            "\"lastActiveAt\":\"2024-01-01T00:00:00Z\",",
            "\"statusText\":\"in a meeting\",\"statusEmoji\":\"busy\"}"
        );
        let SseFrame { event, .. } = parse_sse_block(block);
        match event {
            SseEvent::Presence {
                contact_id,
                presence,
                last_active_at,
                status_text,
                status_emoji,
            } => {
                assert_eq!(contact_id, "ct1");
                assert_eq!(presence, crate::types::ContactPresence::Online);
                assert_eq!(last_active_at.as_deref(), Some("2024-01-01T00:00:00Z"));
                assert_eq!(status_text.as_deref(), Some("in a meeting"));
                assert_eq!(status_emoji.as_deref(), Some("busy"));
            }
            other => panic!("expected Presence, got {other:?}"),
        }
    }

    /// Oracle: RFC 8895 §9 — unrecognized event type must yield Unknown.
    #[test]
    fn parse_unknown_event() {
        let block = "event: ping\ndata: {}";
        let SseFrame { event, .. } = parse_sse_block(block);
        assert!(
            matches!(event, SseEvent::Unknown),
            "unrecognized event type must yield Unknown"
        );
    }

    /// Oracle: RFC 8895 §9 — empty block (keepalive) must yield Unknown.
    #[test]
    fn parse_empty_block() {
        let SseFrame { event, id } = parse_sse_block("");
        assert!(
            matches!(event, SseEvent::Unknown),
            "empty block must yield Unknown"
        );
        assert!(id.is_none(), "empty block must have no id");
    }

    /// Oracle: security requirement §G — malformed JSON in data must yield
    /// Unknown, never panic or propagate an error.
    #[test]
    fn parse_malformed_data_json() {
        let block = "event: state\ndata: not-json";
        let SseFrame { event, .. } = parse_sse_block(block);
        assert!(
            matches!(event, SseEvent::Unknown),
            "malformed JSON must yield Unknown, not panic or error"
        );
    }

    /// Oracle: RFC 8895 §9 — `id:` line value must be returned in `SseFrame::id`.
    #[test]
    fn parse_id_line() {
        let block = "id: evt-42\nevent: state\ndata: {\"changed\":{}}";
        let SseFrame { event, id } = parse_sse_block(block);
        assert_eq!(id.as_deref(), Some("evt-42"), "id must be evt-42");
        assert!(
            matches!(event, SseEvent::StateChange { .. }),
            "must still parse as StateChange"
        );
    }

    /// Oracle: RFC 8895 §9 — multiple `data:` lines must be joined with `\n`.
    ///
    /// Two data: lines are collected and joined. If only the first line were
    /// used, a complete single-line typing JSON would parse as Typing. Because
    /// the second data: line is appended (joined with '\n'), the combined
    /// string is invalid JSON, so the result must be Unknown — proving both
    /// lines are captured.
    #[test]
    fn parse_multiline_data() {
        // First data: line alone is a complete, valid typing JSON object.
        // Second data: line appends "extra", making the joined string invalid JSON.
        // Result must be Unknown (not Typing), proving both lines are joined.
        let block = concat!(
            "event: typing\n",
            "data: {\"chatId\":\"c3\",\"senderId\":\"u3\",\"typing\":true}\n",
            "data: extra"
        );
        let SseFrame { event, .. } = parse_sse_block(block);
        assert!(
            matches!(event, SseEvent::Unknown),
            "both data: lines must be joined: first-line-valid JSON + second line = Unknown"
        );
    }
}
