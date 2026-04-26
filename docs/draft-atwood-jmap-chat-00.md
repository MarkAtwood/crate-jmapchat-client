---
title: JMAP for Chat
abbrev: JMAP Chat
docname: draft-atwood-jmap-chat-00
category: exp
stream: ietf

ipr: trust200902

stand_alone: yes
smart_quotes: no
pi: [toc, sortrefs, symrefs]

author:
  -
    fullname: Mark Atwood
    email: mark@reviewcommit.com

normative:
  RFC2119:
  RFC8174:
  RFC8620:
  ULID:
    title: Universally Unique Lexicographically Sortable Identifier
    target: https://github.com/ulid/spec

informative:
  RFC7763:
  RFC8621:
  RFC9404:
  RFC9420:
  MIMI-CONTENT:
    title: More Instant Messaging Interoperability (MIMI) Content Format
    target: https://datatracker.ietf.org/doc/draft-ietf-mimi-content/
  MIMI-PROTOCOL:
    title: More Instant Messaging Interoperability (MIMI) Protocol
    target: https://datatracker.ietf.org/doc/draft-ietf-mimi-protocol/
  JMAP-CHAT-FED:
    title: JMAP Chat Federation
    target: https://datatracker.ietf.org/doc/draft-atwood-jmap-chat-federation/
  JMAP-OBJ-HISTORY:
    title: JMAP Object History
    target: https://datatracker.ietf.org/doc/draft-gondwana-jmap-object-history/
  RFC9610:
  RFC9670:
  JMAP-FILENODE:
    title: JMAP Filenode
    target: https://datatracker.ietf.org/doc/draft-ietf-jmap-filenode/

--- abstract

This document defines JMAP for Chat, a JMAP capability ({{RFC8620}}) for direct and group text messaging. It supports two deployment topologies: a mailbox-per-user model in which each user operates their own server, and a relay model in which a shared server routes end-to-end encrypted messages.

The specification defines the `urn:ietf:params:jmap:chat` capability; twenty data types (Attachment, Endpoint, MessageAction, Mention, MessageRevision, Reaction, ChatContact, ChatMember, Chat, Message, SpaceRole, SpaceMember, Category, ChannelPermission, Space, SpaceInvite, CustomEmoji, SpaceBan, ReadPosition, and PresenceStatus); and JMAP owner methods for each top-level type. Server-to-server federation methods are defined in {{JMAP-CHAT-FED}}.

The protocol covers the feature set common to contemporary messaging systems: group chat with membership roles, message reactions, editing, deletion, threading, @mentions, typing indicators, read receipts per participant, presence, pinned messages, per-chat notification settings, sender-controlled message expiry, and burn-on-read. Spaces provide a named multi-channel container with a role-based permission system, analogous to what other systems call a server, workspace, or team.


--- middle

# Introduction

JMAP {{RFC8620}} defines a JSON-based protocol for accessing and mutating application data. The core protocol is intentionally generic; application semantics are expressed through capability URIs declared in the JMAP Session object. {{RFC8621}} defines JMAP for Mail. This document defines an analogous capability for real-time chat.

## Deployment Topologies

This specification accommodates two primary deployment topologies.

In the **mailbox-per-user** model, each participant runs their own JMAP server storing only their own messages. Mailboxes exchange messages directly using the server-to-server methods defined in {{JMAP-CHAT-FED}}.

In the **relay** model, a shared server routes messages between clients. The Peer/* methods are implemented by the relay rather than by individual user-controlled mailboxes. Relay deployments are designed to handle only opaque ciphertext; see {{e2ee}} for the normative requirements.

Both topologies are fully compatible with this specification. Transport, identity, and encryption choices are confined to the deployment layer.

## Authentication Model

Authentication is handled entirely at the transport layer. The protocol requires only that the authentication layer provide a stable, opaque user identity string for each connection. How that identity is established — overlay network membership, mutual TLS, bearer tokens, or any other mechanism — is outside the scope of this document. Authorization rules derived from this identity are defined in {{authorization}}.

In the mailbox-per-user deployment topology, the peer authentication model is defined in {{JMAP-CHAT-FED}}.

## Relationship to MIMI

The IETF MIMI (More Instant Messaging Interoperability) working group {{MIMI-PROTOCOL}} {{MIMI-CONTENT}} is developing a separate approach to messaging interoperability, primarily targeting provider-to-provider federation between large existing messaging platforms under regulatory interoperability mandates. MIMI's client-server API layer is intentionally outside the MIMI charter scope; this document fills that gap for JMAP-based deployments.

The two specifications are complementary rather than competing. MIMI defines `application/mimi-content` {{MIMI-CONTENT}}, a CBOR-encoded message body format designed to operate as an MLS PrivateMessage payload. A JMAP Chat server that also participates in a MIMI federation domain MAY include `"application/mimi-content"` in its `supportedBodyTypes` capability and accept MIMI-formatted message bodies in `Message/set` and `Peer/deliver`. This document does not require or preclude such interoperability.

The federation protocol defined in {{JMAP-CHAT-FED}} uses a mailbox-per-user architecture distinct from MIMI's hub-and-spoke room ownership model; see {{JMAP-CHAT-FED}} for discussion of that distinction.

# Conventions and Definitions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 {{RFC2119}} {{RFC8174}} when, and only when, they appear in all capitals, as shown here.

Terminology from {{RFC8620}} is used throughout.

The following terms are specific to this document:

Mailbox:
: A JMAP server instance serving exactly one user.

Owner:
: The user whose data a mailbox stores and serves.

Peer:
: Another mailbox server communicating with this mailbox.

id / userId:
: A ChatContact's `id` is the stable, opaque identity string provided by the authentication layer for that user. These two terms are intentionally equivalent in this protocol: ChatContact.id IS the userId. There is no separate identity namespace. Servers MUST set ChatContact.id to the userId string obtained from the authentication layer.

# The urn:ietf:params:jmap:chat Capability {#capability}

The `urn:ietf:params:jmap:chat` capability is advertised in the JMAP Session object at both the top-level `capabilities` key and within each account's `accountCapabilities` map.

## Session-Level Capability Object

The value of `capabilities["urn:ietf:params:jmap:chat"]` at the session level is an empty object `{}`.

## Account-Level Capability Object

The value of `accountCapabilities["urn:ietf:params:jmap:chat"]` is a JSON object with the following fields:

`maxBodyBytes` (UnsignedInt):
: Maximum UTF-8 byte length of a Message `body`. Servers MUST reject messages exceeding this limit with `invalidArguments`.

`maxAttachmentBytes` (UnsignedInt):
: Maximum size in bytes of a single attachment blob.

`maxAttachmentsPerMessage` (UnsignedInt):
: Maximum number of attachments per message.

`maxGroupMembers` (UnsignedInt):
: Maximum number of members in a group Chat, including the owner.

`maxSpaceMembers` (UnsignedInt):
: Maximum number of members in a Space, including the owner.

`maxRolesPerSpace` (UnsignedInt):
: Maximum number of named roles per Space.

`maxChannelsPerSpace` (UnsignedInt):
: Maximum number of channel Chats per Space.

`maxCategoriesPerSpace` (UnsignedInt):
: Maximum number of categories per Space.

`supportedBodyTypes` (String[]):
: MIME types accepted in `bodyType`. MUST include `"text/plain"`. End-to-end encrypted deployments SHOULD also include an appropriate encrypted-content type such as `"application/mls-ciphertext"`. Servers SHOULD support `"text/markdown"` (CommonMark profile, as specified in {{RFC7763}}) and `"application/jmap-chat-rich"` (defined in {{rich-body}}). Servers participating in MIMI federation domains MAY also include `"application/mimi-content"` {{MIMI-CONTENT}}.

`supportsThreads` (Boolean):
: Whether this server supports the optional thread model defined in {{threads}}.

The `role` field used to distinguish owner and peer accounts in federation deployments is defined in {{JMAP-CHAT-FED}}.

## Session Object Extensions

When the `urn:ietf:params:jmap:chat` capability is present, servers MAY include the following additional fields in the Session object:

`ownerUserId` (String):
: The id of the mailbox owner (equals the owner's ChatContact.id on any peer server that has recorded this mailbox as a contact).

`ownerLogin` (String):
: A human-readable login name for the mailbox owner.

`ownerEndpoints` (Endpoint[], optional):
: The owner's advertised out-of-band capability endpoints (see {{endpoint}}). Peers that probe `/.well-known/jmap` SHOULD merge these into the `endpoints` field of the corresponding ChatContact record.

The `ownerDirectAddress` field used in federation deployments is defined in {{JMAP-CHAT-FED}}.

# Data Types

Data types are defined in dependency order: embedded sub-types precede the types that contain them.

## Attachment {#attachment}

An Attachment carries metadata for a file blob associated with a Message.

`blobId` (String):
: Opaque server-assigned blob identifier.

`filename` (String):
: Original filename. MUST NOT contain `/`, `\`, or null bytes.

`contentType` (String):
: Valid MIME type string.

`size` (UnsignedInt):
: Blob size in bytes. Servers MUST verify against actual content.

`sha256` (String):
: Lowercase hex SHA-256 of blob content. Servers SHOULD verify.

## Endpoint {#endpoint}

An Endpoint advertises an out-of-band capability reachable at a URI. Endpoints appear on ChatContact records and in Session objects as persistent capability advertisements. The `type` field uses an extensible URI namespace; clients MUST silently ignore Endpoint records whose `type` they do not recognize.

`type` (String):
: A URI identifying the capability type. Well-known values defined by this document:

  - `"urn:jmap:chat:cap:vtc"` — video/voice teleconference. `uri` is a signaling or room URL (e.g., a WebRTC signaling endpoint, a Jitsi room URL, a SIP URI).
  - `"urn:jmap:chat:cap:payment"` — payment receiving endpoint. `uri` is a payment URI (e.g., `lightning:...`, `zcash:...`, `monero:...`, `bitcoin:...`).
  - `"urn:jmap:chat:cap:blob"` — out-of-band file transfer endpoint. `uri` is a base URL for fetching or uploading blobs outside the JMAP blob mechanism.

  Other type URIs MAY be defined by deployments or future documents. The `urn:jmap:chat:cap:` prefix is reserved for types defined in JMAP Chat specifications.

`uri` (String):
: The endpoint URI. Format is type-specific. Peer-supplied; MUST be treated as untrusted.

`label` (String, optional):
: Human-readable label for this endpoint (e.g., `"Personal Jitsi"`, `"Zcash address"`).

`metadata` (Object, optional):
: Type-specific key-value pairs. Clients MUST ignore unknown keys. Examples by type:

  - `vtc`: `{"protocol": "webrtc", "roomName": "...", "password": "..."}`
  - `payment`: `{"network": "lightning", "currency": "BTC"}`
  - `blob`: `{"maxBytes": 10485760}`

## MessageAction {#message-action}

A MessageAction is a per-message out-of-band action invitation carried in a Message and in `Peer/deliver`. It signals that a message is associated with an out-of-band interaction — a video call invitation, a payment request, a file available outside the blob channel, etc. Servers MUST store and forward MessageAction records without inspection or transformation. Clients MUST NOT act on a MessageAction automatically; all OOB actions require explicit user initiation.

`type` (String):
: Same URI namespace as Endpoint `type` (see {{endpoint}}). Clients MUST ignore actions whose `type` they do not recognize.

`uri` (String):
: The action URI. Peer-supplied; MUST be treated as untrusted.

`label` (String, optional):
: Human-readable label for the action (e.g., `"Join call"`, `"Pay $5"`, `"Download file"`).

`expiresAt` (UTCDate, optional):
: Time after which the action is no longer valid. Clients SHOULD visually indicate expired actions. Servers MUST NOT enforce expiry on stored actions; enforcement is the OOB system's responsibility.

`metadata` (Object, optional):
: Type-specific key-value pairs. Clients MUST ignore unknown keys.

## Mention {#mention}

A Mention identifies a user referenced within a message body.

`id` (String):
: The ChatContact.id (userId) of the mentioned participant.

`offset` (UnsignedInt):
: Byte offset into `body` where the mention text begins.

`length` (UnsignedInt):
: Byte length of the mention text. Servers MUST reject a mention where `offset + length` exceeds the byte length of `body`.

## MessageRevision {#message-revision}

A MessageRevision records one historical version of a Message body.

`body` (String):
: The prior body text.

`bodyType` (String):
: The prior MIME type.

`editedAt` (UTCDate):
: The time this version was superseded by an edit.

## Reaction {#reaction}

A Reaction is an emoji response to a Message, stored as a value in the `reactions` map on a Message object. The map key is the `senderReactionId`.

`emoji` (String):
: A non-empty string identifying the reaction. Typically a Unicode emoji sequence or a deployment-defined token.

`customEmojiId` (String, optional):
: The id of a Space-scoped custom emoji. When present, `emoji` SHOULD contain a fallback representation (e.g., the emoji name) for clients that do not support custom emoji.

`senderId` (String):
: `"self"` for the owner's reaction, or a ChatContact.id.

`sentAt` (UTCDate):
: Time the reaction was added.

## ChatContact {#chat-contact}

A ChatContact represents a remote user known to this mailbox. A ChatContact's `id` is exactly the userId provided by the authentication layer: it is the single, global identity key for that user within this deployment.

`id` (String, immutable, server-set):
: The userId provided by the authentication layer. Servers MUST set this to the verified identity string and MUST NOT assign a different value.

`login` (String, server-set):
: A non-empty human-readable identifier for this contact, suitable for display when `displayName` is absent. The format is deployment-specific but MUST be a valid UTF-8 string of at least one non-whitespace character. Clients MUST fall back to `login` when `displayName` is absent or empty, and MAY fall back to `id` when `login` is unavailable.

`displayName` (String, optional):
: Human-readable display name. MAY be absent or empty. Clients SHOULD fall back to `login`, then `id`.

`firstSeenAt` (UTCDate, server-set):
: Time this ChatContact was first recorded.

`lastSeenAt` (UTCDate, server-set):
: Time of most recent interaction with this ChatContact's mailbox.

`presence` (String, server-set):
: Last known presence state: `"online"`, `"away"`, `"offline"`, or `"unknown"`. Updated on a best-effort basis.

`lastActiveAt` (UTCDate, optional, server-set):
: Time the ChatContact was last observed to be active.

`endpoints` (Endpoint[], optional):
: Out-of-band capability endpoints advertised by this ChatContact. Servers populate this field from the ChatContact's `ownerEndpoints` at `/.well-known/jmap`. Clients MAY use these to initiate video calls, send payments, or fetch files outside the JMAP message channel. Clients MUST NOT act on these values automatically without explicit user intent.

`blocked` (Boolean):
: When `true`, messages from this ChatContact are silently dropped by this mailbox, including messages arriving in group chats. Default: `false`.

Additional ChatContact fields used in federation deployments (`serverUrl`, `directAddress`) are defined in {{JMAP-CHAT-FED}}.

Note: {{RFC9610}} defines `ContactCard` (a JSContact record) for storing rich contact information in a JMAP address book. `ChatContact` and `ContactCard` serve different purposes and have distinct identity models: `ChatContact.id` is the authenticated userId assigned by the transport layer, while `ContactCard.id` is an opaque JMAP-assigned identifier. Implementations MAY surface `ChatContact` records as `ContactCard` objects in a user's address book as a display-layer mapping; this is not a protocol requirement.

## ChatMember {#chat-member}

A ChatMember describes one participant in a group Chat. The `id` field is the participant's ChatContact.id.

`id` (String):
: The participant's ChatContact.id / userId.

`role` (String):
: Either `"admin"` or `"member"`. Admins may add and remove members and update group chat metadata. The creator is automatically assigned the `"admin"` role.

`joinedAt` (UTCDate):
: Time this participant joined the chat.

`invitedBy` (String, optional):
: The ChatContact.id of the member who added this participant.

## Chat {#chat}

A Chat is a conversation between two or more participants. Three kinds are defined: `"direct"` (one-to-one), `"group"` (multi-party), and `"channel"` (a channel within a Space). Fields whose applicability is restricted to one or two kinds are labeled accordingly; unlabeled fields apply to all kinds.

### Chat ID Assignment {#chat-id}

All Chat IDs are ULIDs {{ULID}} assigned by the creating server at the moment the chat is created. IDs are opaque and stable for the lifetime of the chat.

For a **direct chat**, the creating server is the one whose owner sends the first message. Before assigning a new chatId, the server MUST check whether a direct chat with the relevant contactId already exists locally. If one exists, the server MUST use the existing chatId rather than creating a new one. When a `Peer/deliver` arrives for a direct chat with an unknown chatId, the receiving server creates a new Chat record with that chatId and sets `contactId` to the sender.

For a **group chat**, the creating server assigns the chatId and distributes it to all initial members via `Peer/groupUpdate` ({{peer-groupupdate}}) before any messages are sent.

**Channel** Chats are created as part of a Space via the `addChannels` patch key in `Space/set` ({{space-set}}). Their chatId is assigned by the server at that time.

### Chat Object Fields

`id` (String, immutable, server-set):
: A ULID assigned per {{chat-id}}.

`kind` (String, immutable):
: `"direct"`, `"group"`, or `"channel"`. Channel Chats do not carry a `members` field; membership and access control are determined by the containing Space.

`contactId` (String, immutable):
: **Direct chats only.** The ChatContact.id of the other participant.

`name` (String):
: **Group and channel Chats.** Display name. Required at creation for group chats; mutable by admins. For channel chats, mutable by members with `"manage_channels"` permission.

`description` (String, optional):
: **Group chats only.** Short description. Mutable by admins.

`avatarBlobId` (String, optional):
: **Group chats only.** blobId of the group avatar image. Mutable by admins.

`members` (ChatMember[]):
: **Group chats only.** Full membership list including the owner.

`spaceId` (String, immutable):
: **Channel Chats only.** The id of the containing Space.

`categoryId` (String, optional):
: **Channel Chats only.** The Category id within the Space. Absent if this channel is uncategorized.

`position` (UnsignedInt, optional):
: **Channel Chats only.** Sort order within the category (or among uncategorized channels). Lower values appear first.

`topic` (String, optional):
: **Channel Chats only.** Short description shown in the channel header. Mutable by members with `"manage_channels"` permission.

`slowModeSeconds` (UnsignedInt, optional):
: **Channel Chats only.** When non-zero, each member MUST wait at least this many seconds between messages in this channel. Members with `"manage_channels"` permission are exempt. Enforced server-side; clients SHOULD display a countdown. Default: `0`.

`permissionOverrides` (ChannelPermission[]):
: **Channel Chats only.** Per-channel permission overrides for specific roles or members. Evaluated after Space-level role permissions per {{space-permissions}}. Only members with `"manage_channels"` permission may modify this list. Empty by default.

`createdAt` (UTCDate, immutable, server-set):
: Time this chat was first recorded on this mailbox.

`lastMessageAt` (UTCDate, optional, server-set):
: Received time of the most recent message.

`unreadCount` (UnsignedInt, server-set):
: Count of Messages received after `ReadPosition.lastReadMessageId` for this Chat. Derived server-side from the owner's ReadPosition record.

`pinnedMessageIds` (String[]):
: Ordered list of pinned Message ids, most-recently-pinned first. For group chats, only admins may modify this list. For direct chats, the owner may modify it freely. Empty by default.

`muted` (Boolean):
: When `true`, push notifications for this chat are suppressed. Owner-side preference; not shared with peers. Default: `false`.

`muteUntil` (UTCDate, optional):
: Muting expires at this time. Servers SHOULD clear `muted` and `muteUntil` automatically when the time passes.

`messageExpirySeconds` (UnsignedInt, optional):
: A local expiry policy. When set and non-zero, messages in this chat older than this many seconds are deleted by this mailbox. Each mailbox enforces its own policy independently. This is a local setting, not a bilateral negotiated commitment.

## Message {#message}

A Message is a single transmission within a Chat.

### Message IDs {#message-ids}

Message IDs are ULIDs {{ULID}}, assigned by the **receiving** mailbox at storage time. ULIDs are lexicographically ordered by time, enabling ordered retrieval without a separate sort field.

The **sender-assigned ULID** (`senderMsgId`) is set by the originating mailbox and carried in `Peer/deliver`. The receiving mailbox stores both its own `id` and the `senderMsgId`. Servers MUST maintain a durable index of `senderMsgId` values per chat to support idempotent delivery, `Peer/retract` lookup, and resolution of `replyTo` / `threadRootId` references. If a `senderMsgId` is seen again for the same chat, the server MAY silently discard the duplicate.

### Message Object Fields

`id` (String, immutable, server-set):
: Receiver-assigned ULID. Used in all client-facing references.

`senderMsgId` (String, immutable, server-set):
: The sender-assigned ULID carried in `Peer/deliver`. Equals `id` for messages composed by the owner.

`chatId` (String, immutable):
: ID of the containing Chat.

`senderId` (String, immutable, server-set):
: `"self"` for owner-composed messages; the sender's ChatContact.id for inbound messages, as verified by the authentication layer.

`body` (String):
: Message content. When `bodyType` is `"text/plain"` or another plaintext type, `body` MUST be valid UTF-8 text. When `bodyType` indicates an end-to-end encrypted payload (e.g., `"application/mls-ciphertext"`), `body` contains ciphertext encoded as a base64url string; servers MUST store and forward it without inspection or transformation. Cleared to empty string when the message is deleted.

`bodyType` (String):
: MIME type of `body`. MUST be in `supportedBodyTypes`.

`attachments` (Attachment[]):
: File attachments. Cleared to empty array when deleted.

`mentions` (Mention[]):
: Structured @mention annotations. Empty by default.

`actions` (MessageAction[]):
: Out-of-band action invitations associated with this message. Empty by default. Servers MUST store and forward these without inspection.

`reactions` (Id[Reaction]):
: Emoji reactions, keyed by `senderReactionId`. The `senderReactionId` is a client-assigned ULID for owner reactions, or a peer-supplied ULID for received reactions. Empty object by default.

`replyTo` (String, optional):
: The receiver-assigned `id` of the Message this replies to. Servers MUST validate that this ID exists in the same Chat before storing.

`threadRootId` (String, optional):
: The receiver-assigned `id` of the thread root message. Only meaningful when `supportsThreads` is `true`. See {{threads}}.

`replyCount` (UnsignedInt, server-set):
: Count of messages in this chat with `replyTo` equal to this message's `id`. Present only when `supportsThreads` is `true`.

`unreadReplyCount` (UnsignedInt, server-set, optional):
: Count of replies to this message received after the owner's `ReadPosition.lastReadMessageId` in this Chat. Present only when `supportsThreads` is `true` and this message is a thread root (`threadRootId` is absent). A value of zero means all replies have been read.

`sentAt` (UTCDate):
: Sender's claimed composition time. Peer-supplied; MUST be treated as untrusted and MUST NOT be used for ordering.

`receivedAt` (UTCDate, immutable, server-set):
: Time this mailbox stored the message. Authoritative for ordering and expiry calculations.

`senderExpiresAt` (UTCDate, optional, immutable):
: Sender-set hard-deletion deadline. When present, servers MUST permanently delete this message — removing the row entirely, not leaving a tombstone — at or before this time. A hard-deleted message appears in the `destroyed` list of subsequent `Message/changes` responses, not `updated`. Receiving servers MUST honor this field regardless of local `messageExpirySeconds` policy; whichever deadline arrives first takes effect. Servers MUST NOT use this field for ordering. Servers MUST reject a `senderExpiresAt` value that is already in the past at delivery time with `invalidArguments`. After hard deletion, stored attachment blobs referenced by the message SHOULD also be purged.

`burnOnRead` (Boolean, optional, immutable):
: When `true`, the receiving server MUST permanently hard-delete this message (row removal, not tombstone) immediately after setting `readAt`. Applies only to the receiving mailbox; the sender's own copy is not affected. In E2EE relay deployments, the relay cannot observe read events; the bridge or client layer MUST enforce this rule after receiving the read acknowledgement from the owner.

`deliveryState` (String, server-set):
: `"pending"`, `"delivered"`, `"failed"`, or `"received"`. For group chats, reflects aggregate state across all recipients; see `deliveryReceipts` for per-recipient detail.

`deliveryReceipts` (Object, optional, server-set):
: For group chats, a JSON object mapping each non-owner participant's ChatContact.id to `{"deliveredAt": <UTCDate-or-null>, "readAt": <UTCDate-or-null>}`. Present only when `senderId` is `"self"`.

`deliveredAt` (UTCDate, optional, server-set):
: Time the first outbound delivery was acknowledged.

`readAt` (UTCDate, optional, server-set):
: Time the owner acknowledged reading this message.

`editedAt` (UTCDate, optional, server-set):
: Time of the most recent edit.

`editHistory` (MessageRevision[], optional, server-set):
: Prior versions, oldest first. Servers MAY limit the number of retained revisions.

Note: {{JMAP-OBJ-HISTORY}} defines a general JMAP mechanism for retrieving historical object versions via `Foo/get` with `includeReplaced`. This specification instead embeds edit history inline within the Message object, trading on-demand retrieval for always-available history at the cost of response size.

`deletedAt` (UTCDate, optional, server-set):
: Time the message was deleted. When set, `body` is empty and `attachments` is empty. The record is retained as a tombstone unless a hard-delete rule applies.

`deletedForAll` (Boolean, optional, server-set):
: `true` when deletion was propagated to all participants via `Peer/retract`.

## SpaceRole {#space-role}

A SpaceRole is a named set of permissions within a Space. Roles are ordered by `position`; higher position values outrank lower ones. The implicit `@everyone` role (position 0) is held by all Space members and is not included in the `roles` array.

`id` (String, immutable, server-set):
: A ULID assigned by the server.

`name` (String):
: Display name of the role.

`color` (String, optional):
: Hex color string (e.g., `"#5865f2"`). Clients MAY use this to visually distinguish role holders.

`permissions` (String[]):
: Named permissions this role grants. Defined permission names:

  - `"view"` — see the channel
  - `"send"` — send messages
  - `"pin"` — pin messages
  - `"manage_channels"` — create, edit, delete, and reorder channels
  - `"manage_members"` — kick members, edit nicknames
  - `"manage_roles"` — create and edit roles below own highest role
  - `"manage_space"` — edit Space name, description, and icon
  - `"ban"` — ban and unban members
  - `"manage_emoji"` — create, edit, and delete Space-scoped custom emoji
  - `"mention_all"` — use Space-wide @mentions

  Servers MUST ignore unrecognized permission names.

`position` (UnsignedInt):
: Role hierarchy position. No two roles in a Space SHOULD share the same value.

## SpaceMember {#space-member}

A SpaceMember describes one participant in a Space.

`id` (String):
: The participant's ChatContact.id.

`roleIds` (String[]):
: SpaceRole ids held by this member. Order is not significant. An empty list means the member holds only the `@everyone` role.

`nick` (String, optional):
: Space-specific display name override. MAY be absent; clients SHOULD fall back to ChatContact `displayName`, then `login`.

`joinedAt` (UTCDate):
: Time this member joined the Space.

## Category {#category}

A Category is a named grouping of channels within a Space.

`id` (String, immutable, server-set):
: A ULID assigned by the server.

`name` (String):
: Display name of the category.

`position` (UnsignedInt):
: Sort order among categories. Lower values appear first.

`channelIds` (String[]):
: Ordered list of channel Chat ids in this category.

## ChannelPermission {#channel-permission}

A ChannelPermission record overrides Space-level role permissions for a specific channel, for a specific role or member. The evaluation order for these overrides is defined in {{space-permissions}}.

`targetId` (String):
: A SpaceRole id or a SpaceMember ChatContact.id.

`targetType` (String):
: `"role"` or `"member"`.

`allow` (String[]):
: Permissions explicitly granted in this channel, overriding the Space-level role defaults.

`deny` (String[]):
: Permissions explicitly denied in this channel, overriding the Space-level role defaults.

## Space {#space}

A Space is a named container for channel Chats, members, roles, and categories. It corresponds to what other systems call a server, workspace, or team.

`id` (String, immutable, server-set):
: A ULID assigned by the server.

`name` (String):
: Display name of the Space.

`description` (String, optional):
: Short description. Mutable by members with `"manage_space"` permission.

`iconBlobId` (String, optional):
: blobId of the Space icon. Mutable by members with `"manage_space"` permission.

`roles` (SpaceRole[]):
: Named roles defined for this Space, ordered by `position` descending. Does not include the implicit `@everyone` role.

`members` (SpaceMember[]):
: Full membership list including the owner.

`categories` (Category[]):
: Categories, ordered by `position`.

`uncategorizedChannelIds` (String[]):
: Ordered list of channel Chat ids not assigned to any category.

`createdAt` (UTCDate, immutable, server-set):
: Time this Space was created.

`isPublic` (Boolean):
: If `true`, any user may join this Space via `Space/join` without an invite code. Default is `false`. Mutable by members with `"manage_space"` permission.

`isPubliclyPreviewable` (Boolean):
: If `true`, users who are not members of this Space may query it via `Space/query` and receive the fields `id`, `name`, `description`, `iconBlobId`, `memberCount`, `createdAt`, `isPublic`, and `isPubliclyPreviewable`. Default is `false`. Mutable by members with `"manage_space"` permission.

`memberCount` (UnsignedInt, server-set):
: Current number of members in this Space.

Note: {{RFC9670}} defines a general JMAP sharing framework (`shareWith`, `myRights`) for simple read/write/admin access control. The Space permission model (role hierarchy, named permissions, per-channel overrides) is intentionally richer than that framework and is not expressed using it. In deployments that also implement {{RFC9670}}, server implementations may choose to align `Principal.id` values with `ChatContact.id` values (both ultimately derived from the same authentication identity), but this alignment is implementation-defined and not required by either specification.

Note: {{JMAP-FILENODE}} defines a hierarchical file-storage extension for JMAP. A future companion specification could define Space-scoped file storage by associating a Filenode namespace with each Space, analogous to how server-to-server federation methods are defined in a separate companion draft.

## CustomEmoji {#custom-emoji}

A CustomEmoji is a server- or Space-scoped custom emoji image available for use in Reactions.

`id` (String, immutable, server-set):
: A ULID assigned by the server.

`name` (String):
: The shortcode name for this emoji, without colons (e.g., `catjam`). MUST be unique within its scope (Space or server-global). MUST contain only lowercase alphanumeric characters, hyphens, and underscores.

`blobId` (String):
: blobId of the emoji image. MUST be a valid image type (e.g., PNG, GIF, WebP).

`spaceId` (String, optional):
: The id of the Space this emoji belongs to. If absent, the emoji is server-global and available in all chats on this server.

`createdBy` (String, immutable, server-set):
: ChatContact.id of the user who created this emoji.

`createdAt` (UTCDate, immutable, server-set):
: Time this emoji was created.

## SpaceInvite {#space-invite}

A SpaceInvite grants a new member access to a Space via a shared invite code.

`id` (String, immutable, server-set):
: Opaque server-assigned JMAP identifier for this invite.

`code` (String, immutable, server-set):
: The user-shareable invite code. This is the value passed as `inviteCode` to `Space/join`. Servers SHOULD generate short, URL-safe strings suitable for sharing.

`spaceId` (String, immutable):
: The Space this invite grants access to.

`defaultChannelId` (String, optional):
: Chat id of the channel to highlight when a new member arrives.

`createdBy` (String, immutable, server-set):
: ChatContact.id of the member who created this invite.

`expiresAt` (UTCDate, optional):
: Expiry time. Servers MUST reject redemption after this time.

`maxUses` (UnsignedInt, optional):
: Maximum redemption count. Servers MUST reject redemption when `uses` equals `maxUses`.

`uses` (UnsignedInt, server-set):
: Number of times this invite has been redeemed.

`createdAt` (UTCDate, immutable, server-set):
: Time this invite was created.

## SpaceBan {#space-ban}

A SpaceBan prevents a user from participating in a Space.

`id` (String, immutable, server-set):
: A ULID assigned by the server.

`spaceId` (String, immutable):
: The id of the Space this ban applies to.

`userId` (String, immutable):
: The ChatContact.id of the banned user.

`bannedBy` (String, immutable, server-set):
: The ChatContact.id of the Space member who issued this ban.

`reason` (String, optional):
: Human-readable reason for the ban. Visible to the banned user.

`createdAt` (UTCDate, immutable, server-set):
: Time this ban was created.

`expiresAt` (UTCDate, optional):
: If present, the ban expires at this time and the server MUST restore the user's access. If absent, the ban is permanent until explicitly destroyed.

## ReadPosition {#read-position}

A ReadPosition tracks the owner's read state within a Chat. The server creates a ReadPosition record automatically when a Chat first becomes visible to the owner, and destroys it when the Chat is destroyed.

`id` (String, immutable, server-set):
: A ULID assigned by the server.

`chatId` (String, immutable):
: The id of the Chat this position tracks.

`lastReadMessageId` (String, optional):
: The `id` of the most recent Message the owner has read in this Chat. The server uses this to compute `Chat.unreadCount`. If absent, the owner has not read any messages in this Chat.

`lastReadAt` (UTCDate, server-set):
: Time the `lastReadMessageId` was last updated.

## PresenceStatus {#presence-status}

A PresenceStatus represents the owner's self-reported availability and custom status. There is exactly one PresenceStatus record per account; the server creates it automatically.

`id` (String, immutable, server-set):
: A ULID assigned by the server.

`presence` (String):
: The owner's self-reported availability. One of `"online"`, `"away"`, `"busy"`, `"invisible"`, or `"offline"`. Default is `"online"`.

`statusText` (String, optional):
: A short custom status message. If absent, no custom text is displayed.

`statusEmoji` (String, optional):
: A single emoji or emoji shortcode representing the owner's status. If absent, no status emoji is displayed.

`expiresAt` (UTCDate, optional):
: If set, the server SHOULD clear `statusText` and `statusEmoji` (setting them to null) at this time, and SHOULD reset `presence` to `"online"`.

`updatedAt` (UTCDate, server-set):
: Time the owner last updated this record.

# Methods

## ChatContact Methods

ChatContacts are created automatically when a peer delivers a message or a group update names a new participant. Owner clients may not create or destroy ChatContacts directly.

### ChatContact/get

Standard JMAP `/get` ({{RFC8620}} Section 5.1).

### ChatContact/changes

Standard JMAP `/changes` ({{RFC8620}} Section 5.2).

### ChatContact/set

Standard JMAP `/set` ({{RFC8620}} Section 5.3).

`create` and `destroy` are not supported; both MUST return `forbidden`.

`update` supports: `blocked`, `displayName`.

### ChatContact/query

Standard JMAP `/query` ({{RFC8620}} Section 5.5).

Filter properties: `blocked` (Boolean), `presence` (String).
Sort properties: `lastSeenAt`, `login`, `lastActiveAt`. When sorting by `lastActiveAt`, ChatContacts for which the field is absent sort last.

### ChatContact/queryChanges

Standard JMAP `/queryChanges` ({{RFC8620}} Section 5.6).

## Chat Methods

### Chat/get

Standard JMAP `/get`.

### Chat/changes

Standard JMAP `/changes`.

### Chat/set

Standard JMAP `/set`.

#### Creating a Direct Chat

`create` with `kind: "direct"` accepts:

`contactId` (String, required):
: ChatContact.id of the other participant. If a direct Chat with this contactId already exists, the server MUST return it in `updated` rather than creating a duplicate. Otherwise the server assigns a new ULID per {{chat-id}}.

#### Creating a Group Chat

`create` with `kind: "group"` accepts:

`name` (String, required):
: Display name of the group.

`memberIds` (String[], required):
: ChatContact.ids of initial non-owner members. Total membership including the owner MUST NOT exceed `maxGroupMembers`.

Optional at creation: `description` (String), `avatarBlobId` (String), `messageExpirySeconds` (UnsignedInt).

The server assigns the chatId (a ULID), sets the owner as an admin member, and MUST send `Peer/groupUpdate` to each initial member before any messages are sent.

#### Updating a Chat

`update` supports the following patch keys for all chat kinds: `muted`, `muteUntil`, `pinnedMessageIds`, `messageExpirySeconds`.

For group chats, admins additionally may update: `name`, `description`, `avatarBlobId`.

Member list changes use the following patch keys (all require admin role):

`addMembers` (Object[]):
: Each entry: `id` (String, ChatContact.id) and optional `role` (String, default `"member"`). Total membership after addition MUST NOT exceed `maxGroupMembers`. The server MUST send `Peer/groupUpdate` to all current members.

`removeMembers` (String[]):
: ChatContact.ids to remove. The server MUST send `Peer/groupUpdate` to all remaining members and to the removed members.

`updateMemberRoles` (Object[]):
: Each entry: `id` (String) and `role` (String). The server MUST send `Peer/groupUpdate` to all members.

### Chat/query

Standard JMAP `/query`.

Filter properties: `kind` (String), `muted` (Boolean).
Default sort: `lastMessageAt` descending; chats without messages sort last.

### Chat/queryChanges

Standard JMAP `/queryChanges` ({{RFC8620}} Section 5.6).

## Message Methods

### Message/get

Standard JMAP `/get`.

### Message/changes

Standard JMAP `/changes`.

### Message/set

Standard JMAP `/set`.

#### Creating a Message

`create` accepts:

`chatId` (String, required), `body` (String, required), `bodyType` (String, required), `sentAt` (UTCDate, required).

Optional: `attachments` (Attachment[]), `mentions` (Mention[]), `actions` (MessageAction[]), `replyTo` (String), `threadRootId` (String), `senderExpiresAt` (UTCDate), `burnOnRead` (Boolean).

The server sets `id`, `senderMsgId`, `senderId`, `receivedAt`, `deliveryState`, and delivery timestamp fields, then enqueues the message for outbound delivery.

#### Editing a Message

`update` with changed `body`, `bodyType`, and/or `mentions`, on a message where `senderId` is `"self"` and `deletedAt` is absent.

The server MUST:

1. Push a MessageRevision onto `editHistory` with the current `body`, `bodyType`, and current server time as `editedAt`.
2. Replace `body` and `bodyType` with the submitted values.
3. Set `editedAt` to the current server time.
4. Send `Peer/deliver` carrying an `edit` payload to all recipients (see {{peer-deliver}}).

#### Adding and Removing Reactions

Reactions are mutated via standard JSON Pointer patch keys on the `reactions` map.

To **add** a reaction, the client MUST supply a ULID as the `senderReactionId` and patch:

~~~
"reactions/<senderReactionId>": {
  "emoji": "<value>",
  "sentAt": "<UTCDate>"
}
~~~

The server MUST set `senderId` to `"self"` and MUST propagate the reaction to all recipients via `Peer/deliver` `reactionUpdate` payload, carrying the `senderReactionId` as the map key.

To **remove** a reaction, the client patches:

~~~
"reactions/<senderReactionId>": null
~~~

The server MUST remove the entry and MUST propagate the removal via `Peer/deliver` `reactionUpdate` payload using the same `senderReactionId`.

Clients MUST only add or remove reactions where `senderId` is `"self"`. Servers MUST reject attempts to add or remove reactions for other senders with `forbidden`.

#### Deleting a Message

`update` with `deletedAt: <timestamp>`.

- If `deletedForAll: true` is also set, the server MUST send `Peer/retract` to all participants before marking the local record. Servers MUST reject `deletedForAll: true` for messages where `senderId` is not `"self"`.
- Otherwise, deletion is local only: `body` and `attachments` are cleared on this mailbox with no peer notification.

#### Marking as Read

`update` with `readAt: <UTCDate>`.

### Message/query

Standard JMAP `/query`.

All requests MUST include a `chatId` filter, unless the request includes a `hasMention: true` filter condition. Servers MUST return an `unsupportedFilter` error for any request that omits `chatId` without also including `hasMention: true`.

When `chatId` is absent and `hasMention: true` is present, the query spans all Chats of which the owner is a member. Servers that do not support cross-chat mention queries MUST return `unsupportedFilter`; clients MUST handle this gracefully.

Filter properties:

`chatId` (String, optional):
: Restrict results to a single Chat. Required unless `hasMention: true` is also present.

`text` (String, optional):
: Full-text search over `body`. Servers that do not support full-text search MUST return `unsupportedFilter`.

`threadRootId` (String, optional):
: Return only messages in this thread. Valid only when `supportsThreads` is `true`; otherwise servers MUST return `unsupportedFilter`.

`hasAttachment` (Boolean, optional):
: Filter to messages with or without attachments.

`hasMention` (Boolean, optional):
: Filter to messages that mention the owner (owner's ChatContact.id appears in `mentions`).

Default sort: `receivedAt` ascending when `chatId` is present; `receivedAt` descending when `chatId` is absent.

### Message/queryChanges

Standard JMAP `/queryChanges` ({{RFC8620}} Section 5.6).

## Space Methods

### Space/get

Standard JMAP `/get`.

### Space/changes

Standard JMAP `/changes`. The Space object is returned in full on each change; the sub-arrays `roles`, `members`, and `categories` are included in every Space object and reflect the current state.

### Space/set {#space-set}

Standard JMAP `/set`.

The `update` operation for Space uses semantic mutation keys (`addRoles`, `removeRoles`, `addMembers`, etc.) rather than JSON Pointer paths. This departure from the standard RFC 8620 PatchObject model is intentional: Space membership and role lists are ordered, server-enforced collections subject to permission checks and cascading side effects (e.g., peer notifications, role hierarchy enforcement) that cannot be expressed as simple pointer-path assignments. Each key names a discrete, permission-checked mutation operation rather than a direct property assignment.

#### Creating a Space

`create` accepts:

`name` (String, required).

Optional: `description` (String), `iconBlobId` (String).

The server assigns a ULID, sets the caller as the owner with all permissions, and returns the new Space.

#### Updating a Space

`update` supports the following patch keys:

`name`, `description`, `iconBlobId`:
: Metadata fields. Require `"manage_space"` permission.

`addRoles` (Object[]):
: Each entry: `name` (String), `permissions` (String[]), `position` (UnsignedInt), and optionally `color` (String). Server assigns ULIDs. Total MUST NOT exceed `maxRolesPerSpace`. Requires `"manage_roles"`. Members may only add roles whose `position` is strictly less than their own highest-position role; servers MUST enforce this.

`removeRoles` (String[]):
: SpaceRole ids to remove. Members holding only removed roles are demoted to `@everyone`. Requires `"manage_roles"`.

`updateRoles` (Object[]):
: Each entry: `id` (String) and any of `name`, `color`, `permissions`, `position`. Requires `"manage_roles"`. Members may only modify roles whose `position` is strictly less than their own highest-position role; servers MUST enforce this.

`addMembers` (Object[]):
: Each entry: `id` (ChatContact.id, String) and optional `roleIds` (String[]). Total MUST NOT exceed `maxSpaceMembers`. Requires `"manage_members"`.

`removeMembers` (String[]):
: ChatContact.ids to remove. Requires `"manage_members"`. The owner cannot be removed.

`updateMembers` (Object[]):
: Each entry: `id` (String) and any of `roleIds`, `nick`. Role changes require `"manage_roles"`.

`addChannels` (Object[]):
: Each entry: `name` (String, required), optional `categoryId` (String), `position` (UnsignedInt), and `topic` (String). The server creates a Chat record of `kind: "channel"` with `spaceId` set to this Space's id and assigns a ULID as the chatId. Total channel count MUST NOT exceed `maxChannelsPerSpace`. Requires `"manage_channels"`.

`removeChannels` (String[]):
: Channel Chat ids to remove. Cascades to all Messages in those channels. Requires `"manage_channels"`.

`updateChannels` (Object[]):
: Each entry: `id` (String, channel Chat id) and any of `name`, `topic`, `categoryId`, `position`, `slowModeSeconds`, `permissionOverrides`. Requires `"manage_channels"`.

`addCategories` (Object[]):
: Each entry: `name` (String), optional `position` (UnsignedInt) and `channelIds` (String[]). Server assigns ULIDs. Total MUST NOT exceed `maxCategoriesPerSpace`. Requires `"manage_channels"`.

`removeCategories` (String[]):
: Category ids to remove. Channels in removed categories move to `uncategorizedChannelIds`. Requires `"manage_channels"`.

`updateCategories` (Object[]):
: Each entry: `id` (String) and any of `name`, `position`, `channelIds`. Requires `"manage_channels"`.

#### Destroying a Space

Cascades to all channel Chats and their Messages. Hard-deletes all records; no tombstones are retained. Requires owner identity.

### Space/query

Standard JMAP `/query`.

Filter properties: `name` (String, substring match), `isPublic` (Boolean).
Default sort: `name` ascending.

When the request includes an `isPublic: true` filter condition, servers MUST include Spaces for which the requesting account is not a member but which have `isPubliclyPreviewable: true`. For such non-member results, the server MUST return only the fields `id`, `name`, `description`, `iconBlobId`, `memberCount`, `createdAt`, `isPublic`, and `isPubliclyPreviewable`; all other fields MUST be omitted. For Spaces where the requesting account is a member, full Space objects are returned regardless of the filter.

### Space/queryChanges

Standard JMAP `/queryChanges` ({{RFC8620}} Section 5.6).

### Space/join

Allows a caller to join a Space either by redeeming an invite code or by directly requesting membership in a public Space.

Method name: `Space/join`

Request: `accountId` (String), and exactly one of `inviteCode` (String) or `spaceId` (String). Supplying both or neither MUST cause the server to return an `invalidArguments` method error.

**Joining via invite code:** The server MUST resolve `inviteCode` to a SpaceInvite record by matching against the `code` field (not the `id` field), verify the invite has not expired and has not reached `maxUses`, and then atomically increment `uses` and add the caller to the Space's member list. The `uses` increment and membership insertion MUST be performed within a single atomic operation so that concurrent redemptions cannot exceed `maxUses`. The caller is assigned no roles beyond `@everyone` unless the invite specifies otherwise. If the invite has expired (`expiresAt` is in the past) or has reached its redemption limit (`uses` equals `maxUses`), the server MUST return an `invalidArguments` method error. If the `inviteCode` does not correspond to any SpaceInvite record, the server MUST return an `invalidArguments` method error.

**Joining a public Space:** When `spaceId` is supplied, the server MUST verify that the identified Space has `isPublic: true`. If the Space does not exist or has `isPublic: false`, the server MUST return a `notPermitted` method error. On success, the server adds the caller to the Space's member list with no roles beyond `@everyone`.

Response: `accountId` (String), `spaceId` (String).

## ReadPosition Methods

### ReadPosition/get

Standard JMAP `/get` ({{RFC8620}} Section 5.1).

### ReadPosition/changes

Standard JMAP `/changes` ({{RFC8620}} Section 5.2).

### ReadPosition/set

Standard JMAP `/set` ({{RFC8620}} Section 5.3).

`update` supports: `lastReadMessageId`. The server sets `lastReadAt` to the current time and recomputes `Chat.unreadCount` for the affected Chat.

`create` and `destroy` are not supported; both MUST return a SetError of type `forbidden`. ReadPosition records are managed by the server.

## PresenceStatus Methods

### PresenceStatus/get

Standard JMAP `/get` ({{RFC8620}} Section 5.1). Returns the singleton PresenceStatus record for the account.

### PresenceStatus/changes

Standard JMAP `/changes` ({{RFC8620}} Section 5.2).

### PresenceStatus/set

Standard JMAP `/set` ({{RFC8620}} Section 5.3).

`update` supports: `presence`, `statusText`, `statusEmoji`, `expiresAt`.

`create` and `destroy` are not supported; both MUST return a SetError of type `forbidden`. The PresenceStatus record is managed by the server.

## CustomEmoji Methods

### CustomEmoji/get

Standard JMAP `/get` ({{RFC8620}} Section 5.1).

### CustomEmoji/changes

Standard JMAP `/changes` ({{RFC8620}} Section 5.2).

### CustomEmoji/set

Standard JMAP `/set` ({{RFC8620}} Section 5.3).

`create` accepts: `name` (String, required), `blobId` (String, required), `spaceId` (String, optional). For Space-scoped emoji, requires `"manage_emoji"` permission in the Space. For server-global emoji, requires server admin authorization (server policy).

`update` supports: `name`, `blobId`.

`destroy` removes the emoji. Existing Reaction records referencing this emoji retain their `customEmojiId` value; clients SHOULD fall back to the `emoji` field when the referenced emoji cannot be resolved.

### CustomEmoji/query

Standard JMAP `/query` ({{RFC8620}} Section 5.5).

Filter properties: `spaceId` (String, optional — if absent returns all emoji accessible to the account including server-global).
Default sort: `name` ascending.

## SpaceInvite Methods

### SpaceInvite/get

Standard JMAP `/get`. Only members of the Space may retrieve its invites. Members with `"manage_members"` permission see all invites; other members see only invites they created.

### SpaceInvite/set

Standard JMAP `/set`.

`create` accepts: `spaceId` (String, required), `defaultChannelId` (String, optional), `expiresAt` (UTCDate, optional), `maxUses` (UnsignedInt, optional). Requires `"manage_members"` permission. The server assigns both `id` and `code`; clients MUST NOT supply either.

`destroy` revokes an invite. Requires `"manage_members"` permission or ownership of the invite.

`update` is not supported; any attempt MUST return a SetError of type `forbidden`.

## SpaceBan Methods

### SpaceBan/get

Standard JMAP `/get`. Members with `"ban"` permission in the Space see all SpaceBan records for that Space. A banned user fetching their own account sees only SpaceBan records where `userId` matches their own identity.

### SpaceBan/changes

Standard JMAP `/changes`.

### SpaceBan/set

Standard JMAP `/set`.

`create` accepts: `spaceId` (String, required), `userId` (String, required), `reason` (String, optional), `expiresAt` (UTCDate, optional). Requires `"ban"` permission in the Space.

`update` supports: `reason`, `expiresAt`.

`destroy` lifts the ban. Requires `"ban"` permission in the Space.

# Rich Body Format {#rich-body}

This section defines the `application/jmap-chat-rich` body type for structured inline formatting.

When `bodyType` is `"application/jmap-chat-rich"`, `body` MUST be a valid JSON object conforming to this section. The `mentions` array on the Message MUST be empty; mention information is carried inline within the spans.

## Span Object

The body object contains a single field:

`spans` (Span[]):
: An ordered array of Span objects representing the message content from left to right.

Each Span has the following fields:

`type` (String):
: One of the defined span types below.

`text` (String):
: The plaintext content of this span. MUST be present on all span types. Clients that do not support a given span type MUST render this field as plaintext.

Additional fields are type-specific:

| Type | Additional fields | Meaning |
|---|---|---|
| `"text"` | none | Plain text |
| `"bold"` | none | Bold text |
| `"italic"` | none | Italic text |
| `"bold-italic"` | none | Bold and italic text |
| `"code"` | none | Inline code |
| `"codeblock"` | `lang` (String, optional) | Fenced code block; `lang` is a language hint for syntax highlighting |
| `"blockquote"` | none | Quoted text |
| `"mention"` | `userId` (String) | @mention; `userId` is the ChatContact.id of the mentioned user |
| `"link"` | `uri` (String) | Hyperlink; `uri` MUST be treated as untrusted |

Servers MUST reject messages containing unknown span types with `invalidArguments`. Clients MUST silently ignore unknown span types and render the `text` field as plaintext, preserving forward compatibility.

## Example

~~~json
{
  "spans": [
    {"type": "text", "text": "Hello "},
    {"type": "mention", "text": "@alice", "userId": "user:alice@example.com"},
    {"type": "text", "text": ", see this code: "},
    {"type": "code", "text": "foo()"},
    {"type": "text", "text": ". Full example:"},
    {"type": "codeblock", "text": "fn main() {}", "lang": "rust"}
  ]
}
~~~

# Thread Model {#threads}

Servers advertising `supportsThreads: true` support structured conversation threads.

A thread is the set of Messages sharing a common `threadRootId`. The root message has `threadRootId` absent.

Thread root assignment rules:

- A message with no `replyTo` is a potential thread root. `threadRootId` MUST be absent.
- A message replying to a thread root (the referenced message has no `threadRootId`): set `threadRootId` to the value of `replyTo`.
- A message replying to a non-root message: set `threadRootId` to the referenced message's `threadRootId`.

Clients MUST follow these rules. Servers SHOULD validate them and MAY correct `threadRootId` if the client supplies an incorrect value.

`Message/query` with a `threadRootId` filter returns all messages in that thread. The `replyCount` field on each message gives the count of direct replies.

# Server-to-Server Methods

Server-to-server federation methods (`Peer/deliver`, `Peer/receipt`, `Peer/typing`, `Peer/retract`, `Peer/groupUpdate`) are defined in {{JMAP-CHAT-FED}}.

# Push Notifications {#push}

Servers MUST support the EventSource mechanism defined in {{RFC8620}} Section 7.3.

Servers SHOULD also support the push subscription mechanism defined in {{RFC8620}} Section 7.2 for deployments requiring offline and mobile push delivery.

## State-Change Events

~~~
event: state
data: {"@type":"StateChange","changed":{"<accountId>":{"Message":"<s>","Chat":"<s>","ChatContact":"<s>"}}}
~~~

Clients SHOULD call the corresponding `/changes` method upon receipt. On `cannotCalculateChanges`, fall back to `/get`.

## Typing Events

~~~
event: typing
data: {"chatId":"<id>","senderId":"<contact-id>","typing":<bool>}
~~~

Not stored; carries no state token.

## Presence Events

~~~
event: presence
data: {"contactId":"<id>","presence":"<state>","lastActiveAt":"<ts>","statusText":"<string>|null","statusEmoji":"<string>|null"}
~~~

The `statusText` and `statusEmoji` fields reflect the contact's current PresenceStatus values; they are `null` when the contact has no active custom status. Clients SHOULD update their displayed status immediately upon receiving this event without waiting for a `/changes` poll.

# Blob Storage {#blobs}

Standard JMAP blob upload and download per {{RFC8620}} Section 6, using the `uploadUrl` and `downloadUrl` Session templates.

Implementations requiring additional blob operations (Blob/get, Blob/copy) SHOULD refer to {{RFC9404}} (JMAP Blob Management Extensions). The upload response defined here extends the RFC 8620 response with a `sha256` field not defined in RFC 9404; this extension is specific to this specification.

## Upload

`POST <uploadUrl>` with the blob as the request body.

Response (HTTP 200) — this document extends the standard RFC 8620 upload response with the `sha256` field:

~~~json
{
  "blobId":  "<id>",
  "type":    "<mime-type>",
  "size":    <bytes>,
  "sha256":  "<lowercase-hex>"
}
~~~

## Download

`GET <downloadUrl>` with placeholders percent-encoded.

# Outbox and Delivery {#outbox}

Outbound delivery mechanics for the mailbox-per-user federation topology are defined in {{JMAP-CHAT-FED}}.

# Authorization {#authorization}

Authentication is handled at the transport layer as described in {{introduction}}. The protocol derives access control from the stable, opaque id provided per connection:

- **Owner** (identity equals the mailbox owner's id): all methods.
- **Other**: HTTP 401.

Authorization for peer server access in federation deployments is defined in {{JMAP-CHAT-FED}}.

# Space Permission Resolution {#space-permissions}

When determining whether a member may perform an action in a channel Chat, servers MUST evaluate permissions in this order:

1. The Space owner always has all permissions; skip remaining steps.
2. Compute the union of `permissions` across all SpaceRoles held by the member, including the implicit `@everyone` role.
3. Apply `deny` entries from any ChannelPermission records matching the member's roles (`targetType: "role"`), in ascending position order.
4. Apply `allow` entries from the same role-targeted records, in ascending position order.
5. Apply `deny` entries from any ChannelPermission record matching the member directly (`targetType: "member"`).
6. Apply `allow` entries from the same member-targeted record.

Servers MUST perform this resolution server-side. Clients MUST NOT be trusted to assert their own permissions.

Role hierarchy enforcement: members may only create or modify SpaceRoles whose `position` is strictly less than their own highest-position role. Servers MUST reject role management operations that violate this constraint.

# Security Considerations {#security}

## Identity Verification

`senderUserId` in all Peer/* methods is caller-supplied and MUST be treated as untrusted. The server MUST obtain the verified identity from its own authentication layer independently and MUST compare before any storage or action. Verification MUST precede all effects.

## Input Validation

All peer-supplied fields are attacker-controlled. Servers MUST validate:

- `body`: validate UTF-8 for plaintext body types; enforce `maxBodyBytes`.
- `bodyType`: validate against `supportedBodyTypes`.
- `filename`: reject values containing `/`, `\`, or null bytes.
- `contentType`: reject syntactically invalid MIME values.
- `size`: verify against actual blob byte count after fetch.
- `sha256`: verify against actual blob content after fetch.
- `sentAt`, `editedAt`: store as-is; never use for ordering or expiry.
- `chatId`: verify per {{chat-id}} — confirm match with stored id or verify membership; reject mismatches.
- `emoji`: validate as a non-empty string; enforce an implementation-defined maximum byte length to prevent denial of service.
- `mentions`: reject any entry where `offset + length` exceeds body byte length.

## Denial of Service

Enforce `maxBodyBytes` and `maxAttachmentBytes` at parse time, before any fetch or storage. Enforce `maxAttachmentsPerMessage`, `maxGroupMembers`, and `maxSpaceMembers` at creation and update time. Rate-limit `Peer/typing` per peer.

## Chat ID Integrity

Chat IDs are server-assigned ULIDs. Security against cross-conversation injection relies on sender authentication and chat membership verification, not on ID derivation.

## Blocked Contacts

Messages from a ChatContact whose `blocked` field is `true` are silently dropped regardless of whether they arrive in a direct chat or a group chat context.

## Out-of-Band Endpoints and Actions

`ChatContact.endpoints`, `Session.ownerEndpoints`, and `Message.actions` carry peer-supplied URIs and MUST be treated as untrusted at every level:

- Clients MUST NOT fetch or connect to any OOB URI automatically. All OOB interactions require explicit user initiation.
- Payment URIs (`urn:jmap:chat:cap:payment`) MUST be validated by the client wallet before any funds are transferred. Servers MUST NOT inspect or act on payment URI values.
- VTC URIs (`urn:jmap:chat:cap:vtc`) MUST NOT be opened without user consent; auto-joining a call is a privacy violation.
- Blob/file URIs (`urn:jmap:chat:cap:blob`) used for OOB fetch are an SSRF vector; servers that fetch from peer-supplied blob endpoints MUST restrict connections to the known peer address space.
- `metadata` values are peer-supplied and MUST be ignored if they do not conform to the expected shape for the known `type`.
- Unknown `type` URIs MUST be silently ignored by both clients and servers.

## End-to-End Encrypted Deployments {#e2ee}

In relay deployments, the relay routes Peer/* messages but MUST NOT have access to plaintext message content. Implementations MUST ensure:

- The `body` field carries ciphertext only; plaintext MUST never be transmitted to the relay in an encrypted deployment.
- The relay is architecturally excluded from the encryption key schedule (e.g., by using MLS {{RFC9420}} or a similar protocol that does not involve the relay in key agreement).
- Servers MUST NOT reject or transform `body` based on content when `bodyType` indicates an encrypted type.
- Metadata visible to the relay — sender id, recipient id, timestamp, and body size — remains an information-leakage surface. Deployments requiring metadata privacy SHOULD apply message padding and cover traffic at the transport layer; those techniques are outside the scope of this document.

## Federation Security

Security considerations for server-to-server federation are defined in {{JMAP-CHAT-FED}}.

# IANA Considerations

## JMAP Capability Registration

IANA is requested to register the following entry in the "JMAP Capabilities" registry:

Capability Name:
: `urn:ietf:params:jmap:chat`

Intended Use:
: common

Change Controller:
: IETF

Reference:
: This document.

Security and Privacy Considerations:
: See {{security}} of this document.


## JMAP Data Types Registration

IANA is requested to register the following entries in the "JMAP Data Types" registry:

| Type Name | Can Use for State Change | Is Per-User | Server-set | Supports /changes | Supports /queryChanges | Reference |
|---|---|---|---|---|---|---|
| ChatContact | Yes | No | Yes | Yes | Yes | This document |
| Chat | Yes | No | No | Yes | Yes | This document |
| Message | Yes | No | No | Yes | Yes | This document |
| Space | Yes | No | No | Yes | Yes | This document |
| SpaceInvite | Yes | No | No | Yes | No | This document |
| CustomEmoji | Yes | No | No | Yes | Yes | This document |
| SpaceBan | Yes | No | No | Yes | No | This document |
| ReadPosition | Yes | Yes | No | Yes | No | This document |
| PresenceStatus | Yes | Yes | Yes | Yes | No | This document |

--- back

# Acknowledgements

The author thanks the JMAP working group for {{RFC8620}}.
