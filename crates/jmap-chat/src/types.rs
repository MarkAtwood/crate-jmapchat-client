use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use crate::jmap::{Id, UTCDate};

// ---------------------------------------------------------------------------
// Attachment
// ---------------------------------------------------------------------------

/// File attachment metadata for a Message.
/// Spec: draft-atwood-jmap-chat-00 §4.1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    /// Opaque server-assigned blob identifier.
    pub blob_id: Id,
    /// Original filename. MUST NOT contain `/`, `\`, or null bytes.
    pub filename: String,
    /// Valid MIME type string.
    pub content_type: String,
    /// Blob size in bytes.
    pub size: u64,
    /// Lowercase hex SHA-256 of blob content.
    pub sha256: String,
}

// ---------------------------------------------------------------------------
// Endpoint
// ---------------------------------------------------------------------------

/// An out-of-band capability endpoint advertised on a ChatContact or Session.
/// Spec: draft-atwood-jmap-chat-00 §4.2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    /// URI identifying the capability type.
    #[serde(rename = "type")]
    pub endpoint_type: String,
    /// The endpoint URI. Format is type-specific.
    pub uri: String,
    /// Human-readable label for this endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Type-specific key-value pairs. Clients MUST ignore unknown keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// MessageAction
// ---------------------------------------------------------------------------

/// A per-message out-of-band action invitation.
/// Spec: draft-atwood-jmap-chat-00 §4.3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageAction {
    /// URI identifying the action type (same namespace as Endpoint.type).
    #[serde(rename = "type")]
    pub action_type: String,
    /// The action URI. Peer-supplied; MUST be treated as untrusted.
    pub uri: String,
    /// Human-readable label for the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Time after which the action is no longer valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<UTCDate>,
    /// Type-specific key-value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Mention
// ---------------------------------------------------------------------------

/// A structured @mention annotation within a message body.
/// Spec: draft-atwood-jmap-chat-00 §4.4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    /// The ChatContact.id (userId) of the mentioned participant.
    pub id: Id,
    /// Byte offset into `body` where the mention text begins.
    pub offset: u64,
    /// Byte length of the mention text.
    pub length: u64,
}

// ---------------------------------------------------------------------------
// MessageRevision
// ---------------------------------------------------------------------------

/// One historical version of a Message body (edit history entry).
/// Spec: draft-atwood-jmap-chat-00 §4.5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageRevision {
    /// The prior body text.
    pub body: String,
    /// The prior MIME type.
    pub body_type: String,
    /// The time this version was superseded by an edit.
    pub edited_at: UTCDate,
}

// ---------------------------------------------------------------------------
// Reaction
// ---------------------------------------------------------------------------

/// An emoji reaction to a Message, stored in the `reactions` map.
/// Spec: draft-atwood-jmap-chat-00 §4.6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    /// A non-empty string identifying the reaction (Unicode emoji or token).
    pub emoji: String,
    /// The id of a Space-scoped custom emoji, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_emoji_id: Option<Id>,
    /// `"self"` for the owner's reaction, or a ChatContact.id.
    pub sender_id: String,
    /// Time the reaction was added.
    pub sent_at: UTCDate,
}

// ---------------------------------------------------------------------------
// ChatContact
// ---------------------------------------------------------------------------

/// A remote user known to this mailbox.
/// Spec: draft-atwood-jmap-chat-00 §4.7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatContact {
    /// The userId provided by the authentication layer.
    pub id: Id,
    /// A non-empty human-readable identifier for this contact.
    pub login: String,
    /// Human-readable display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Time this ChatContact was first recorded.
    pub first_seen_at: UTCDate,
    /// Time of most recent interaction with this ChatContact's mailbox.
    pub last_seen_at: UTCDate,
    /// Last known presence state.
    pub presence: ContactPresence,
    /// Time the ChatContact was last observed to be active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_at: Option<UTCDate>,
    /// Out-of-band capability endpoints advertised by this ChatContact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<Endpoint>>,
    /// When `true`, messages from this ChatContact are silently dropped.
    pub blocked: bool,
}

/// Last known presence state for a ChatContact.
/// Spec: draft-atwood-jmap-chat-00 §4.7
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContactPresence {
    Online,
    Away,
    Offline,
    Unknown,
}

// ---------------------------------------------------------------------------
// ChatMember
// ---------------------------------------------------------------------------

/// One participant in a group Chat.
/// Spec: draft-atwood-jmap-chat-00 §4.8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMember {
    /// The participant's ChatContact.id / userId.
    pub id: Id,
    /// Either `"admin"` or `"member"`.
    pub role: ChatMemberRole,
    /// Time this participant joined the chat.
    pub joined_at: UTCDate,
    /// The ChatContact.id of the member who added this participant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invited_by: Option<Id>,
}

/// Role of a participant in a group Chat.
/// Spec: draft-atwood-jmap-chat-00 §4.8
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatMemberRole {
    Admin,
    Member,
}

// ---------------------------------------------------------------------------
// ChannelPermission
// ---------------------------------------------------------------------------

/// Per-channel permission override for a specific role or member.
/// Spec: draft-atwood-jmap-chat-00 §4.14
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelPermission {
    /// A SpaceRole id or a SpaceMember ChatContact.id.
    pub target_id: Id,
    /// `"role"` or `"member"`.
    pub target_type: ChannelPermissionTargetType,
    /// Permissions explicitly granted in this channel.
    pub allow: Vec<String>,
    /// Permissions explicitly denied in this channel.
    pub deny: Vec<String>,
}

/// Whether a ChannelPermission targets a role or a member.
/// Spec: draft-atwood-jmap-chat-00 §4.14
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelPermissionTargetType {
    Role,
    Member,
}

// ---------------------------------------------------------------------------
// Chat
// ---------------------------------------------------------------------------

/// A JMAP Chat object (JMAP Chat §4.1).
///
/// This type is **deserialization-only**: it is populated from server responses.
/// Field applicability by `kind` is enforced by the server, not by this struct.
/// Constructing a `Chat` in application code is not supported — use the
/// `Chat/get` method instead.
///
/// A conversation between two or more participants.
/// Spec: draft-atwood-jmap-chat-00 §4.9
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    /// A ULID assigned per §4.9.1.
    pub id: Id,
    /// `"direct"`, `"group"`, or `"channel"`.
    pub kind: ChatKind,

    // --- direct only ---
    /// Direct chats only: ChatContact.id of the other participant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_id: Option<Id>,

    // --- group and channel ---
    /// Group and channel Chats: display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    // --- group only ---
    /// Group chats only: short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Group chats only: blobId of the group avatar image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_blob_id: Option<Id>,
    /// Group chats only: full membership list including the owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<ChatMember>>,

    // --- channel only ---
    /// Channel Chats only: the id of the containing Space.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<Id>,
    /// Channel Chats only: the Category id within the Space.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Id>,
    /// Channel Chats only: sort order within the category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    /// Channel Chats only: short description shown in the channel header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Channel Chats only: minimum seconds between messages per member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slow_mode_seconds: Option<u64>,
    /// Channel Chats only: per-channel permission overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_overrides: Option<Vec<ChannelPermission>>,

    // --- all kinds ---
    /// Time this chat was first recorded on this mailbox.
    pub created_at: UTCDate,
    /// Received time of the most recent message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_at: Option<UTCDate>,
    /// Count of unread Messages for this Chat.
    pub unread_count: u64,
    /// Ordered list of pinned Message ids, most-recently-pinned first.
    pub pinned_message_ids: Vec<Id>,
    /// When `true`, push notifications for this chat are suppressed.
    pub muted: bool,
    /// Muting expires at this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute_until: Option<UTCDate>,
    /// Local expiry policy: messages older than this many seconds are deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_expiry_seconds: Option<u64>,
}

/// The kind of a Chat conversation.
/// Spec: draft-atwood-jmap-chat-00 §4.9
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatKind {
    Direct,
    Group,
    Channel,
}

// ---------------------------------------------------------------------------
// DeliveryReceipt (nested in Message.deliveryReceipts)
// ---------------------------------------------------------------------------

/// Per-recipient delivery/read receipt for group message delivery tracking.
/// Spec: draft-atwood-jmap-chat-00 §4.10
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryReceipt {
    /// Time the message was delivered to this recipient, or null.
    pub delivered_at: Option<UTCDate>,
    /// Time this recipient read the message, or null.
    pub read_at: Option<UTCDate>,
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

/// A single transmission within a Chat.
/// Spec: draft-atwood-jmap-chat-00 §4.10
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Receiver-assigned ULID.
    pub id: Id,
    /// The sender-assigned ULID carried in `Peer/deliver`.
    pub sender_msg_id: Id,
    /// ID of the containing Chat.
    pub chat_id: Id,
    /// `"self"` for owner-composed messages; the sender's ChatContact.id otherwise.
    pub sender_id: String,
    /// Message content.
    pub body: String,
    /// MIME type of `body`.
    pub body_type: String,
    /// File attachments.
    pub attachments: Vec<Attachment>,
    /// Structured @mention annotations.
    pub mentions: Vec<Mention>,
    /// Out-of-band action invitations.
    pub actions: Vec<MessageAction>,
    /// Emoji reactions, keyed by `senderReactionId`.
    pub reactions: HashMap<String, Reaction>,

    /// The receiver-assigned `id` of the Message this replies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Id>,
    /// The receiver-assigned `id` of the thread root message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_root_id: Option<Id>,
    /// Count of messages in this chat with `replyTo` equal to this message's `id`.
    pub reply_count: u64,
    /// Count of unread replies to this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread_reply_count: Option<u64>,

    /// Sender's claimed composition time.
    pub sent_at: UTCDate,
    /// Time this mailbox stored the message.
    pub received_at: UTCDate,
    /// Sender-set hard-deletion deadline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_expires_at: Option<UTCDate>,
    /// When `true`, permanently hard-delete after the owner reads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burn_on_read: Option<bool>,

    /// Delivery state across all recipients.
    pub delivery_state: DeliveryState,
    /// Per-recipient delivery/read receipts (group, owner-sent messages only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_receipts: Option<HashMap<String, DeliveryReceipt>>,
    /// Time the first outbound delivery was acknowledged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered_at: Option<UTCDate>,
    /// Time the owner acknowledged reading this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_at: Option<UTCDate>,

    /// Time of the most recent edit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<UTCDate>,
    /// Prior versions, oldest first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_history: Option<Vec<MessageRevision>>,

    /// Time the message was deleted (tombstone).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<UTCDate>,
    /// `true` when deletion was propagated to all participants.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_for_all: Option<bool>,
}

/// Delivery state of a Message.
/// Spec: draft-atwood-jmap-chat-00 §4.10
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryState {
    Pending,
    Delivered,
    Failed,
    Received,
}

// ---------------------------------------------------------------------------
// SpaceRole
// ---------------------------------------------------------------------------

/// A named set of permissions within a Space.
/// Spec: draft-atwood-jmap-chat-00 §4.11
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceRole {
    /// A ULID assigned by the server.
    pub id: Id,
    /// Display name of the role.
    pub name: String,
    /// Hex color string (e.g., `"#5865f2"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Named permissions this role grants.
    pub permissions: Vec<String>,
    /// Role hierarchy position. Higher values outrank lower ones.
    pub position: u64,
}

// ---------------------------------------------------------------------------
// SpaceMember
// ---------------------------------------------------------------------------

/// One participant in a Space.
/// Spec: draft-atwood-jmap-chat-00 §4.12
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceMember {
    /// The participant's ChatContact.id.
    pub id: Id,
    /// SpaceRole ids held by this member. Empty means only `@everyone`.
    pub role_ids: Vec<Id>,
    /// Space-specific display name override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick: Option<String>,
    /// Time this member joined the Space.
    pub joined_at: UTCDate,
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

/// A named grouping of channels within a Space.
/// Spec: draft-atwood-jmap-chat-00 §4.13
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    /// A ULID assigned by the server.
    pub id: Id,
    /// Display name of the category.
    pub name: String,
    /// Sort order among categories. Lower values appear first.
    pub position: u64,
    /// Ordered list of channel Chat ids in this category.
    pub channel_ids: Vec<Id>,
}

// ---------------------------------------------------------------------------
// Space
// ---------------------------------------------------------------------------

/// A named container for channel Chats, members, roles, and categories.
/// Spec: draft-atwood-jmap-chat-00 §4.15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    /// A ULID assigned by the server.
    pub id: Id,
    /// Display name of the Space.
    pub name: String,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// blobId of the Space icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_blob_id: Option<Id>,
    /// Named roles defined for this Space, ordered by `position` descending.
    pub roles: Vec<SpaceRole>,
    /// Full membership list including the owner.
    pub members: Vec<SpaceMember>,
    /// Categories, ordered by `position`.
    pub categories: Vec<Category>,
    /// Ordered list of channel Chat ids not assigned to any category.
    pub uncategorized_channel_ids: Vec<Id>,
    /// Time this Space was created.
    pub created_at: UTCDate,
    /// If `true`, any user may join without an invite code.
    pub is_public: bool,
    /// If `true`, non-members may query this Space via `Space/query`.
    pub is_publicly_previewable: bool,
    /// Current number of members in this Space.
    pub member_count: u64,
}

// ---------------------------------------------------------------------------
// CustomEmoji
// ---------------------------------------------------------------------------

/// A server- or Space-scoped custom emoji image.
/// Spec: draft-atwood-jmap-chat-00 §4.16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomEmoji {
    /// A ULID assigned by the server.
    pub id: Id,
    /// The shortcode name, without colons (e.g., `catjam`).
    pub name: String,
    /// blobId of the emoji image.
    pub blob_id: Id,
    /// The id of the Space this emoji belongs to; absent means server-global.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<Id>,
    /// ChatContact.id of the user who created this emoji.
    pub created_by: Id,
    /// Time this emoji was created.
    pub created_at: UTCDate,
}

// ---------------------------------------------------------------------------
// SpaceInvite
// ---------------------------------------------------------------------------

/// A pending invitation to join a Space via a shared invite code.
/// Spec: draft-atwood-jmap-chat-00 §4.17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceInvite {
    /// Opaque server-assigned JMAP identifier for this invite.
    pub id: Id,
    /// The user-shareable invite code.
    pub code: String,
    /// The Space this invite grants access to.
    pub space_id: Id,
    /// Chat id of the channel to highlight when a new member arrives.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_channel_id: Option<Id>,
    /// ChatContact.id of the member who created this invite.
    pub created_by: Id,
    /// Expiry time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<UTCDate>,
    /// Maximum redemption count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u64>,
    /// Number of times this invite has been redeemed.
    pub uses: u64,
    /// Time this invite was created.
    pub created_at: UTCDate,
}

// ---------------------------------------------------------------------------
// SpaceBan
// ---------------------------------------------------------------------------

/// A ban record preventing a user from participating in a Space.
/// Spec: draft-atwood-jmap-chat-00 §4.18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceBan {
    /// A ULID assigned by the server.
    pub id: Id,
    /// The id of the Space this ban applies to.
    pub space_id: Id,
    /// The ChatContact.id of the banned user.
    pub user_id: Id,
    /// The ChatContact.id of the Space member who issued this ban.
    pub banned_by: Id,
    /// Human-readable reason for the ban.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Time this ban was created.
    pub created_at: UTCDate,
    /// If present, the ban expires at this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<UTCDate>,
}

// ---------------------------------------------------------------------------
// ReadPosition
// ---------------------------------------------------------------------------

/// Tracks the owner's read state within a Chat.
/// Spec: draft-atwood-jmap-chat-00 §4.19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadPosition {
    /// A ULID assigned by the server.
    pub id: Id,
    /// The id of the Chat this position tracks.
    pub chat_id: Id,
    /// The `id` of the most recent Message the owner has read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_read_message_id: Option<Id>,
    /// Time the `lastReadMessageId` was last updated.
    pub last_read_at: UTCDate,
}

// ---------------------------------------------------------------------------
// PresenceStatus
// ---------------------------------------------------------------------------

/// The owner's self-reported availability and custom status (singleton).
/// Spec: draft-atwood-jmap-chat-00 §4.20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceStatus {
    /// A ULID assigned by the server.
    pub id: Id,
    /// The owner's self-reported availability.
    pub presence: OwnerPresence,
    /// A short custom status message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_text: Option<String>,
    /// A single emoji or shortcode representing the owner's status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_emoji: Option<String>,
    /// If set, clear `statusText`/`statusEmoji` and reset `presence` at this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<UTCDate>,
    /// Time the owner last updated this record.
    pub updated_at: UTCDate,
}

/// Self-reported availability for a PresenceStatus record.
/// Spec: draft-atwood-jmap-chat-00 §4.20
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OwnerPresence {
    Online,
    Away,
    Busy,
    Invisible,
    Offline,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture(name: &str) -> String {
        let path = format!(
            "{}/tests/fixtures/types/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read fixture {path}: {e}"))
    }

    /// Oracle: spec §4.9 — Chat object fields, hand-written from spec definition.
    #[test]
    fn test_chat_fixture_deserializes_correctly() {
        let json = fixture("chat.json");
        let chat: Chat = serde_json::from_str(&json).expect("chat.json must parse");

        // Oracle: spec §4.9 — kind values are "direct", "group", "channel"
        assert_eq!(chat.kind, ChatKind::Group);
        assert_eq!(chat.id, "01HV5Z6QKWJ7N3P8R2X4YTMD3G");
        assert_eq!(chat.name.as_deref(), Some("Engineering Team"));
        assert_eq!(chat.unread_count, 3);
        assert!(!chat.muted);
        assert!(chat.pinned_message_ids.is_empty());

        let members = chat.members.as_ref().expect("group chat must have members");
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].id, "user:alice@example.com");
        assert_eq!(members[0].role, ChatMemberRole::Admin);
        assert_eq!(members[1].id, "user:bob@example.com");
        assert_eq!(members[1].role, ChatMemberRole::Member);
    }

    /// Oracle: spec §4.10 — Message object fields, hand-written from spec definition.
    #[test]
    fn test_message_fixture_deserializes_correctly() {
        let json = fixture("message.json");
        let msg: Message = serde_json::from_str(&json).expect("message.json must parse");

        // Oracle: spec §4.10 — senderId is "self" for owner-composed messages
        assert_eq!(msg.id, "01HV5Z6QKWJ7N3P8R2X4YTMD00");
        assert_eq!(msg.sender_id, "self");
        assert_eq!(msg.body, "Hello, world!");
        assert_eq!(msg.body_type, "text/plain");
        assert_eq!(msg.delivery_state, DeliveryState::Delivered);
        assert!(msg.attachments.is_empty());
        assert!(msg.mentions.is_empty());
        assert!(msg.actions.is_empty());
        assert!(msg.reactions.is_empty());
        assert_eq!(msg.reply_count, 0);
        assert!(msg.reply_to.is_none());
        assert!(msg.deleted_at.is_none());
    }

    /// Oracle: spec §4.9 — optional fields absent from JSON deserialize to None.
    #[test]
    fn test_chat_optional_fields_absent_become_none() {
        let json = fixture("chat_direct.json");
        let chat: Chat = serde_json::from_str(&json).expect("chat_direct.json must parse");

        // Oracle: spec §4.9 — direct chats have no name, no members, no spaceId
        assert_eq!(chat.kind, ChatKind::Direct);
        assert!(chat.name.is_none(), "direct chat must not have name");
        assert!(chat.members.is_none(), "direct chat must not have members");
        assert!(chat.space_id.is_none(), "direct chat must not have spaceId");
        assert!(chat.description.is_none());
        assert!(chat.last_message_at.is_some());
        // contactId is required for direct chats
        assert_eq!(chat.contact_id.as_deref(), Some("user:carol@example.com"));
    }

    /// Oracle: spec §4.10 — DeliveryState serializes to the correct lowercase string.
    #[test]
    fn test_delivery_state_serializes_to_spec_string() {
        // Oracle: spec §4.10 text: "pending", "delivered", "failed", "received"
        let pending = serde_json::to_string(&DeliveryState::Pending).unwrap();
        let delivered = serde_json::to_string(&DeliveryState::Delivered).unwrap();
        let failed = serde_json::to_string(&DeliveryState::Failed).unwrap();
        let received = serde_json::to_string(&DeliveryState::Received).unwrap();

        assert_eq!(pending, "\"pending\"");
        assert_eq!(delivered, "\"delivered\"");
        assert_eq!(failed, "\"failed\"");
        assert_eq!(received, "\"received\"");
    }

    /// Oracle: spec §4.20 — OwnerPresence serializes to spec-defined lowercase strings.
    #[test]
    fn test_owner_presence_serializes_to_spec_string() {
        // Oracle: spec §4.20: "online", "away", "busy", "invisible", "offline"
        let cases = [
            (OwnerPresence::Online, "\"online\""),
            (OwnerPresence::Away, "\"away\""),
            (OwnerPresence::Busy, "\"busy\""),
            (OwnerPresence::Invisible, "\"invisible\""),
            (OwnerPresence::Offline, "\"offline\""),
        ];
        for (variant, expected) in cases {
            let got = serde_json::to_string(&variant).unwrap();
            assert_eq!(
                got, expected,
                "OwnerPresence::{variant:?} wrong serialization"
            );
        }
    }

    /// Oracle: spec §4.8 — ChatMemberRole serializes to "admin" or "member".
    #[test]
    fn test_chat_member_role_serializes_to_spec_string() {
        // Oracle: spec §4.8: role is "admin" or "member"
        assert_eq!(
            serde_json::to_string(&ChatMemberRole::Admin).unwrap(),
            "\"admin\""
        );
        assert_eq!(
            serde_json::to_string(&ChatMemberRole::Member).unwrap(),
            "\"member\""
        );
    }

    /// Oracle: spec §4.19 — ReadPosition with absent lastReadMessageId deserializes to None.
    #[test]
    fn test_read_position_absent_last_read_is_none() {
        let json = fixture("read_position.json");
        let rp: ReadPosition = serde_json::from_str(&json).expect("read_position.json must parse");

        // Oracle: spec §4.19 — lastReadMessageId is optional; absent means no messages read
        assert!(
            rp.last_read_message_id.is_none(),
            "lastReadMessageId must be None when absent from JSON"
        );
        assert_eq!(rp.chat_id, "01HV5Z6QKWJ7N3P8R2X4YTMD3G");
    }
}
