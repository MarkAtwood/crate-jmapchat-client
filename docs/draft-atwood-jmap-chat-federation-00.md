---
title: JMAP Chat Federation
abbrev: JMAP Chat Federation
docname: draft-atwood-jmap-chat-federation-00
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
  RFC8615:
  JMAP-CHAT:
    title: JMAP for Chat
    author:
      fullname: Mark Atwood
    seriesinfo:
      Internet-Draft: draft-atwood-jmap-chat-00
    date: 2026

informative:
  RFC9420:
  MIMI-PROTOCOL:
    title: More Instant Messaging Interoperability (MIMI) Protocol
    target: https://datatracker.ietf.org/doc/draft-ietf-mimi-protocol/

--- abstract

This document defines the server-to-server federation protocol for JMAP Chat ({{JMAP-CHAT}}). It specifies peer discovery via `/.well-known/jmap`, the peer authentication model, the `role` field in the JMAP Chat account capability, ChatContact and Session object extensions required for federation, and the Peer/* methods used to exchange messages and events between mailbox servers. Five methods are REQUIRED: `Peer/deliver`, `Peer/receipt`, `Peer/typing`, `Peer/retract`, and `Peer/groupUpdate`. Three additional methods for federated presence are OPTIONAL: `Peer/subscribePresence`, `Peer/unsubscribePresence`, and `Peer/presence`. This document is a companion to {{JMAP-CHAT}} and is intended to be read alongside it.

--- middle

# Introduction

{{JMAP-CHAT}} defines the JMAP Chat capability, which supports a mailbox-per-user deployment topology in which each participant operates their own JMAP server. In this topology, mailboxes must exchange messages and events directly with one another over the open Internet. This document formalizes the mechanisms by which that exchange occurs.

Specifically, this document defines:

- How a peer server discovers another server's JMAP session URL ({{peer-discovery}}).
- How a peer server authenticates to another server and obtains peer-role access ({{peer-authentication}}).
- Extensions to the ChatContact data type and the JMAP Session object that support federation ({{chat-contact-extensions}} and {{session-extensions}}).
- The `role` field in the JMAP Chat account-level capability ({{account-capability}}).
- The five mandatory server-to-server Peer/* methods ({{peer-methods}}).
- Three optional Peer/* methods for federated presence: `Peer/subscribePresence`, `Peer/unsubscribePresence`, and `Peer/presence` ({{peer-subscribepresence}}, {{peer-unsubscribepresence}}, {{peer-presence}}).
- Outbox and delivery semantics, including retry behavior ({{outbox}}).
- Security considerations specific to the federation protocol ({{security}}).

## Deployment Context

In the mailbox-per-user model, each participant runs their own JMAP server (a "mailbox") that stores only their own messages. There is no central server, no central message store, and no central operator. Mailboxes exchange messages directly with each other over a secure transport using the Peer/* server-to-server methods defined in this document.

This document does not redefine the relay topology described in {{JMAP-CHAT}}. In a relay deployment, the relay itself implements the Peer/* methods internally; this federation protocol applies only when distinct, independently operated mailbox servers communicate with one another.

## Relationship to JMAP Chat

This document extracts and formalizes the server-to-server elements of {{JMAP-CHAT}}. Implementations MUST support {{JMAP-CHAT}} as a prerequisite to implementing this document. Terminology, data types, and method semantics from {{JMAP-CHAT}} are used throughout without re-definition.

## Relationship to MIMI

The IETF MIMI working group {{MIMI-PROTOCOL}} is developing a provider-to-provider federation protocol for messaging interoperability. MIMI adopts a hub-and-spoke architecture in which each room is owned by one provider (the "hub") and other providers connect as "followers." The hub is authoritative for room state and distributes messages to follower providers.

This document takes a different approach: each participant operates an independent mailbox server with no central room owner. Message delivery is point-to-point between mailboxes; room state is derived from the union of messages each mailbox has received. This design prioritizes decentralization and avoids any single provider holding authority over a conversation, at the cost of additional complexity in race condition handling (see {{direct-chat-race}}).

The two protocols are not interoperable at the federation layer and serve different deployment contexts. They are not in conflict; a deployment could implement both, using this protocol for JMAP-native mailbox-to-mailbox federation and MIMI for cross-provider interoperability with non-JMAP messaging platforms.

# Conventions and Definitions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 {{RFC2119}} {{RFC8174}} when, and only when, they appear in all capitals, as shown here.

Terminology from {{RFC8620}} and {{JMAP-CHAT}} is used throughout.

The following terms are specific to this document:

Local server:
: The JMAP Chat mailbox server receiving a federation connection from a remote peer.

Remote server / peer server:
: A JMAP Chat mailbox server initiating a federation connection to the local server.

Peer identity:
: The stable, opaque identity string assigned by the local server's authentication layer to a successfully authenticated remote server. This string MUST equal the `ownerUserId` advertised in the remote server's own JMAP Session object.

# Peer Discovery {#peer-discovery}

Peer servers discover each other's JMAP session URL via the Well-Known URI mechanism defined in {{RFC8615}}.

## Discovery Procedure

To discover a peer's JMAP session URL, a server issues:

~~~
GET /.well-known/jmap HTTP/1.1
Host: <peer-hostname>
~~~

The response is the peer's JMAP Session object as defined in {{RFC8620}} Section 2. The Session object MUST include an `ownerUserId` field identifying the mailbox owner (see {{session-extensions}}).

Servers SHOULD cache discovered session data, including the session URL and `ownerUserId`, to avoid redundant discovery on each request. Servers SHOULD re-run discovery when delivery fails, as the peer's session URL or capabilities may have changed.

## Direct Address Hints

The Session object MAY include an `ownerDirectAddress` field (see {{session-extensions}}). This field is an optional hint for out-of-band direct delivery and is deployment-defined in format and semantics. Servers MUST fall back to the standard JMAP API path on any failure when using a direct address. Servers MUST NOT use `ownerDirectAddress` as a blob fetch target (see {{ssrf}}).

# Peer Authentication Model {#peer-authentication}

## Overview

Before a remote server can invoke any Peer/* method on a local server, it MUST authenticate and obtain a JMAP session at the local server. The local server MUST provide a JMAP account in the resulting session that carries the `urn:ietf:params:jmap:chat` capability with `role: "peer"` (see {{account-capability}}).

The local server MUST verify that the remote server's authenticated identity — the stable identity string assigned by the local server's authentication layer — equals the `ownerUserId` advertised in the remote server's own `/.well-known/jmap` Session object. The local server MUST perform this correspondence check before granting peer-role access. If the authenticated identity does not match the discovered `ownerUserId`, the local server MUST reject the connection.

## Authentication Mechanism

The specific authentication mechanism is deployment-defined. Acceptable mechanisms include, but are not limited to:

- Mutual TLS, where the peer's certificate identity corresponds to its `ownerUserId`.
- OAuth 2.0 client credentials grant, where the `client_id` or an associated claim corresponds to the peer's `ownerUserId`.
- Bearer tokens issued through a prior out-of-band enrollment process.
- Overlay network membership credentials (e.g., a Tailscale node identity).

Regardless of mechanism, the local server MUST obtain a stable, opaque identity string for the authenticated peer from its own authentication layer. This string becomes the peer's ChatContact.id on the local server (see {{JMAP-CHAT}} Section on ChatContact) and is the identity against which all Peer/* method arguments are validated.

## Identity Binding

The peer's stable identity string on the local server MUST equal the `ownerUserId` in the peer's own JMAP Session object, as discovered via `/.well-known/jmap`. This binding ensures that:

1. The ChatContact record created or updated for a peer on the local server uses the same id that the peer itself advertises.
2. The `senderUserId` argument in Peer/* methods can be validated against the peer's authenticated identity.
3. Cross-server identity forgery is not possible through mismatched identity namespaces.

The local server MUST discover the remote server's `ownerUserId` (via {{peer-discovery}}) and verify it matches the authenticated identity before processing any Peer/* request from that server.

# Account-Level Capability {#account-capability}

The value of `accountCapabilities["urn:ietf:params:jmap:chat"]` is a JSON object. This document defines the following field within that object, which is relevant to federation:

`role` (String):
: Either `"owner"` or `"peer"`. An account with `role: "owner"` has full access to all JMAP Chat methods. An account with `role: "peer"` is granted access only to the Peer/* methods defined in this document. Callers authenticated as a peer and operating against an account where `role` is `"peer"` MUST receive `forbiddenMethod` for any method not listed in {{peer-methods}}.

The local server MUST include this field in the account capability for any account it provisions for peer use.

# ChatContact Type Extensions for Federation {#chat-contact-extensions}

The ChatContact data type defined in {{JMAP-CHAT}} includes the following fields that are specific to federation. They are defined here as the normative reference for their federation semantics.

`serverUrl` (String):
: Base HTTPS URL of this contact's mailbox. Used for outbound delivery and for probing `/.well-known/jmap`. Servers MUST treat this value as the base URL for discovery and delivery to this contact's mailbox. Servers populate this field at contact creation time and SHOULD update it when a more recent value is discovered via `/.well-known/jmap`.

`directAddress` (String, optional):
: A deployment-specific address at which this contact's client may be reachable directly, without routing through their mailbox. The format and semantics are deployment-defined (examples: a Tailscale node name, a WebRTC signaling URI, an IP:port tuple). This field is a hint only: senders MAY attempt delivery to this address when both parties are online and the deployment supports it, but MUST fall back to the standard mailbox path on any failure. This field has no effect on message storage or multi-device sync. Servers populate this field from the `ownerDirectAddress` advertised in the contact's `/.well-known/jmap` Session object (see {{session-extensions}}). This value is peer-supplied and MUST be treated as untrusted (see {{direct-address-security}}).

# Session Object Extensions for Federation {#session-extensions}

Servers implementing this specification MUST include the following fields in their JMAP Session object (advertised at `/.well-known/jmap`):

`ownerUserId` (String):
: The id of the mailbox owner. This value equals the owner's ChatContact.id on any peer server that has recorded this mailbox as a contact. Servers MUST advertise this field. Peer servers use this value as the authoritative identity for the mailbox owner when performing the identity binding check described in {{peer-authentication}}.

Servers MAY include the following additional field:

`ownerDirectAddress` (String, optional):
: A deployment-specific address at which the owner's client may be reachable directly. Peers that probe `/.well-known/jmap` SHOULD store this value as the `directAddress` on the corresponding ChatContact record (see {{chat-contact-extensions}}). The format and semantics are deployment-defined. This value is advisory only; receiving servers MUST NOT treat it as authoritative or use it as a blob fetch target.

# Server-to-Server Methods {#peer-methods}

The following methods are used between mailbox servers only. An authenticated peer server accesses these methods using the JMAP account in which `role: "peer"` is set in the `urn:ietf:params:jmap:chat` accountCapabilities (see {{account-capability}}). Callers without the `"peer"` role MUST receive `forbiddenMethod` for all methods in this section.

## Peer/deliver {#peer-deliver}

Delivers a new message, an edit, or a reaction update from a remote mailbox. Exactly one of `message`, `edit`, or `reactionUpdate` MUST be present in a given request.

### Request Arguments

`accountId` (String):
: Account ID on the receiving server.

`chatId` (String):
: The Chat ID. For direct chats, if the receiving server already has a chat with the sender's contactId, it MUST verify that `chatId` matches the stored chatId. For group and channel chats, the receiver MUST verify this value matches the chatId of a known chat of which the sender is a current member.

`senderUserId` (String):
: The sender's id (ChatContact.id / userId). MUST match the identity provided by the authentication layer. The receiver MUST compare these and MUST reject the request if they differ.

`chatKind` (String):
: `"direct"`, `"group"`, or `"channel"`. Informs which chatId verification procedure applies.

`message` (Object, optional):
: A new message to deliver. Fields:

  - `senderMsgId` (String) — Sender-assigned ULID. Functions as an idempotency key; if this value is already known for the given chat, the receiver MAY silently discard the duplicate.
  - `body` (String) — Message content. Validated per step 5 of {{new-message-validation}}.
  - `bodyType` (String) — MIME type of `body`. Validated against `supportedBodyTypes`.
  - `sentAt` (UTCDate) — Sender's claimed composition time. Stored as-is; MUST NOT be used for ordering.
  - `attachments` (Object[]) — Each entry carries Attachment fields (as defined in {{JMAP-CHAT}}) plus `fetchUrl` (String): the URL from which the receiver fetches the blob.
  - `mentions` (Mention[], optional) — Structured @mention annotations.
  - `actions` (MessageAction[], optional) — Out-of-band action invitations. Servers MUST store and forward without inspection.
  - `replyTo` (String, optional) — Sender's own `senderMsgId` of the message being replied to. The receiver resolves this to a local message `id` via the `senderMsgId` index.
  - `threadRootId` (String, optional) — Sender's own `senderMsgId` of the thread root message. The receiver resolves this similarly.
  - `senderExpiresAt` (UTCDate, optional) — Hard-deletion deadline. Servers MUST reject a value that is already in the past at delivery time with `invalidArguments`.
  - `burnOnRead` (Boolean, optional) — When `true`, the receiver MUST permanently hard-delete the message immediately after setting `readAt`.

`edit` (Object, optional):
: An edit to an existing message. Fields:

  - `senderMsgId` (String) — Identifies the message to edit via the `senderMsgId` index.
  - `body` (String) — New body content. Validated against `maxBodyBytes`.
  - `bodyType` (String) — MIME type of the new body. Validated against `supportedBodyTypes`.
  - `editedAt` (UTCDate) — Claimed edit time. Stored as-is.
  - `mentions` (Mention[], optional) — Structured @mention annotations for the new body.

  The receiver MUST verify the sender is the original sender of the identified message before applying the edit.

`reactionUpdate` (Object, optional):
: A reaction change on an existing message. Fields:

  - `senderMsgId` (String) — Identifies the target message via the `senderMsgId` index.
  - `senderReactionId` (String) — Sender-assigned ULID for this reaction. For `"add"`, the receiver stores the reaction in the `reactions` map using `senderReactionId` as the map key. For `"remove"`, identifies which reaction to delete.
  - `emoji` (String) — Non-empty string identifying the reaction.
  - `customEmojiId` (String, optional) — Space-scoped custom emoji id, if applicable.
  - `action` (String) — `"add"` or `"remove"`.
  - `sentAt` (UTCDate) — Time of the reaction event.

### New Message Validation {#new-message-validation}

Before storing a new message, the server MUST perform the following steps in order:

1. Verify caller identity via the authentication layer.
2. Confirm `senderUserId` matches the verified identity. Reject immediately if they differ.
3. For direct chats: if a chat with this sender already exists, confirm `chatId` matches its stored id; otherwise create a new Chat record with this chatId and set `contactId` to the sender (see {{direct-chat-race}} for the simultaneous-initiation race condition). For group and channel chats: confirm `chatId` matches a known chat and the sender is a current member of that chat.
4. Confirm the sender is not blocked by the owner. If `blocked` is `true` on the sender's ChatContact record, silently drop the message and return success (to avoid disclosing block status to the sender).
5. Validate `body` byte length against `maxBodyBytes`; reject with `invalidArguments` if exceeded. For plaintext `bodyType` values, also validate UTF-8 encoding. For encrypted `bodyType` values (e.g., `"application/mls-ciphertext"`), `body` is opaque ciphertext; servers MUST NOT parse or transform it beyond byte-length checking.
6. Validate `bodyType` against the server's `supportedBodyTypes`; reject with `invalidArguments` if not present. Servers implementing {{JMAP-CHAT}} that support the `application/jmap-chat-rich` body type SHOULD include it in `supportedBodyTypes` and MUST be prepared to receive it in `Peer/deliver`.
7. Validate each attachment `filename` (MUST NOT contain `/`, `\`, or null bytes), `contentType` (MUST be a syntactically valid MIME type string), and `size`.
8. Fetch each attachment blob from its `fetchUrl`; verify the byte count against `size` and the content against `sha256`. Reject with `invalidArguments` if either check fails. See {{ssrf}} for restrictions on `fetchUrl` targets.
9. Validate each mention `offset + length` against the body byte length. Reject with `invalidArguments` if any mention exceeds the body bounds.
10. If `senderExpiresAt` is present, confirm it is strictly in the future; reject with `invalidArguments` if it is in the past or equal to the current time. Schedule hard deletion at that time. If `burnOnRead` is also `true`, register a trigger to hard-delete the message when `readAt` is set.

Failure at any step MUST result in rejection with no data stored and no side effects.

### Edit and Reaction Validation

For `edit` payloads: perform steps 1 through 4 of {{new-message-validation}}, then validate `body` and `bodyType` per steps 5 and 6. Verify that the identified message exists in the given chat and that `senderUserId` matches the recorded sender of that message. Reject if the sender does not match.

For `reactionUpdate` payloads: perform steps 1 through 4. Validate `emoji` as a non-empty string. For `"add"`, verify the target message exists. For `"remove"`, verify the target reaction exists and was originally added by `senderUserId`. Reject if any check fails.

### Response

`accountId` (String):
: The account ID from the request.

`receivedMsgId` (String):
: For a new message: the receiver's assigned ULID for the stored message. For an edit or reaction update: the local `id` of the affected message.

`receivedAt` (UTCDate):
: The time at which the receiving server stored the message or applied the edit or reaction.

## Peer/receipt

Notifies the sending mailbox that a message has been stored and/or read by a recipient.

Method name: `Peer/receipt`

### Request Arguments

`accountId` (String):
: Account ID on the receiving server.

`senderMsgId` (String):
: The sender-assigned ULID identifying the message for which this receipt is being sent.

`deliveredAt` (UTCDate, optional):
: Time the message was stored by the recipient's mailbox. SHOULD be present on first delivery acknowledgement.

`readAt` (UTCDate, optional):
: Time the message was read by the recipient. SHOULD be present when the recipient's owner has read the message.

`readerUserId` (String):
: The ChatContact.id of the acknowledging user. For group chats, used to update the per-recipient delivery receipt in `deliveryReceipts`.

### Response

`accountId` (String):
: The account ID from the request.

## Peer/typing

Notifies a remote mailbox that the owner of the calling server is or is not currently typing in a shared chat.

Method name: `Peer/typing`

### Request Arguments

`accountId` (String):
: Account ID on the receiving server.

`chatId` (String):
: The Chat ID of the conversation in which typing is occurring.

`senderUserId` (String):
: The ChatContact.id of the typing user. MUST match the authenticated identity; the receiver MUST reject the request if they differ.

`typing` (Boolean):
: `true` if the user is typing; `false` if typing has stopped.

### Response

`accountId` (String):
: The account ID from the request.

### Receiver Behavior

The receiving server MUST NOT persist this event. It MUST forward a typing push event (as defined in {{JMAP-CHAT}}) to the owner's connected clients. Servers MUST rate-limit inbound `Peer/typing` calls per peer to prevent abuse. Servers SHOULD accept no more than one `Peer/typing` call per peer per chat per 3 seconds; calls received above this rate MAY be silently discarded without error.

## Peer/retract

Requests that a remote mailbox tombstone a specific message on behalf of the original sender.

Method name: `Peer/retract`

### Request Arguments

`accountId` (String):
: Account ID on the receiving server.

`chatId` (String):
: The Chat ID containing the message to be retracted.

`senderUserId` (String):
: The ChatContact.id of the user requesting retraction. MUST match the authenticated identity.

`senderMsgId` (String):
: The sender-assigned ULID of the message to retract.

### Receiver Behavior

The receiving server MUST look up the message by `senderMsgId` within `chatId`. It MUST verify that the stored message's `senderId` matches `senderUserId` before applying any changes. If the sender does not match, the server MUST reject the request.

On success, the server MUST: clear `body` to an empty string, clear `attachments` to an empty array, set `deletedAt` to the current time, and set `deletedForAll` to `true`. The message record is retained as a tombstone.

### Response

`accountId` (String):
: The account ID from the request.

`retractedAt` (UTCDate):
: The time at which the retraction was applied.

## Peer/groupUpdate {#peer-groupupdate}

Notifies participant mailboxes of a new group chat or of a membership or metadata change to an existing one.

Method name: `Peer/groupUpdate`

### Request Arguments

`accountId` (String):
: Account ID on the receiving server.

`chatId` (String):
: The group chat ULID.

`senderUserId` (String):
: The ChatContact.id of the user making the change. MUST match the authenticated identity and MUST be an admin of the identified group on the sending server.

`action` (String):
: One of the following values:

  - `"create"` — Initial group creation notification. The receiver MUST create a new Chat record of `kind: "group"` with this chatId.
  - `"addMembers"` — One or more members were added to the group.
  - `"removeMembers"` — One or more members were removed from the group.
  - `"updateRoles"` — One or more members' roles within the group were changed.
  - `"updateMetadata"` — The group's `name`, `description`, or `avatarBlobId` was changed.

`members` (ChatMember[], required for `"create"`):
: Full membership list at the time of this update, including the calling server's owner.

`addedMembers` (ChatMember[], required for `"addMembers"`):
: The newly added members.

`removedMemberIds` (String[], required for `"removeMembers"`):
: ChatContact.ids of the members who were removed.

`updatedRoles` (Object[], required for `"updateRoles"`):
: Each entry MUST contain `id` (String, the ChatContact.id of the member) and `role` (String, the new role value: `"admin"` or `"member"`).

`name` (String, optional):
: Updated group display name. MAY be present for `"create"` or `"updateMetadata"` actions.

`description` (String, optional):
: Updated group description. MAY be present for `"create"` or `"updateMetadata"` actions.

`avatarBlobId` (String, optional):
: Updated group avatar blob ID. MAY be present for `"create"` or `"updateMetadata"` actions. For `"create"`, the receiver MAY fetch this blob from the sending server.

### Receiver Behavior

The receiving server MUST verify that `senderUserId` is authenticated. For all actions other than `"create"`, it MUST also verify that `senderUserId` holds an admin role in the named group on the receiving server's local Chat record before applying any changes. For `"create"`, it MUST verify that `senderUserId` is one of the members listed in `members`.

On success, the receiving server updates its local Chat record to reflect the change indicated by `action`.

### Response

`accountId` (String):
: The account ID from the request.

## Peer/subscribePresence {#peer-subscribepresence}

Requests that the remote server push presence updates for its owner to the calling server. Servers MAY implement this method; callers MUST handle `unknownMethod` gracefully.

Method name: `Peer/subscribePresence`

### Request Arguments

`accountId` (String):
: Account ID on the remote server whose owner's presence is being subscribed to.

`subscriberId` (String):
: The `ownerUserId` of the calling server. MUST match the authenticated identity; the remote server MUST reject the request if they differ.

`ttl` (UnsignedInt, optional):
: Requested subscription lifetime in seconds. Servers MAY cap or adjust this value. If absent, the server assigns an implementation-defined default TTL. Subscriptions that are not renewed before expiry SHOULD be silently dropped by the server.

### Response

`accountId` (String):
: The account ID from the request.

`presence` (String):
: The owner's current presence state at the time of subscription.

`lastActiveAt` (UTCDate, optional):
: Current value of the owner's `lastActiveAt`, if known.

`statusText` (String, optional):
: Current custom status text, if set.

`statusEmoji` (String, optional):
: Current status emoji, if set.

`ttl` (UnsignedInt):
: The actual subscription lifetime in seconds as granted by the server.

### Behavior

On success, the remote server MAY push subsequent `Peer/presence` calls to the subscriber whenever the owner's presence, `statusText`, or `statusEmoji` changes. Servers SHOULD rate-limit outbound `Peer/presence` calls per subscriber to no more than one call per 30 seconds. Servers MAY drop presence push deliveries that fail; presence delivery is best-effort.

Calling `Peer/subscribePresence` again before the TTL expires renews the subscription and SHOULD reset the TTL to the newly granted value.

## Peer/unsubscribePresence {#peer-unsubscribepresence}

Cancels a presence subscription before its TTL expires. Servers MAY implement this method; callers MUST handle `unknownMethod` gracefully. Servers MUST silently succeed if no active subscription exists for the given `subscriberId`.

Method name: `Peer/unsubscribePresence`

### Request Arguments

`accountId` (String):
: Account ID on the remote server.

`subscriberId` (String):
: The `ownerUserId` of the calling server. MUST match the authenticated identity.

### Response

`accountId` (String):
: The account ID from the request.

## Peer/presence {#peer-presence}

Delivers a presence update for the calling server's owner to a subscriber. Servers MAY implement this method; callers MUST handle `unknownMethod` gracefully.

Method name: `Peer/presence`

### Request Arguments

`accountId` (String):
: Account ID on the receiving server.

`contactId` (String):
: The `ownerUserId` of the pushing server. MUST match the authenticated identity; the receiver MUST reject the request if they differ.

`presence` (String):
: The updated presence state of the remote owner. One of `"online"`, `"away"`, `"busy"`, `"invisible"`, or `"offline"`.

`lastActiveAt` (UTCDate, optional):
: Updated `lastActiveAt` timestamp, if available.

`statusText` (String, optional):
: Updated custom status text. If absent, the receiver SHOULD treat the current value as unchanged. To explicitly clear the field, the sender MUST send `null`.

`statusEmoji` (String, optional):
: Updated status emoji. If absent, the receiver SHOULD treat the current value as unchanged. To explicitly clear the field, the sender MUST send `null`.

### Response

`accountId` (String):
: The account ID from the request.

### Receiver Behavior

The receiving server MUST update the `presence` and `lastActiveAt` fields on the ChatContact record identified by `contactId`. It MUST fire a local presence push event (as defined in {{JMAP-CHAT}}) to the owner's connected clients.

# Outbox and Delivery {#outbox}

## Persistent Queue

Outbound messages MUST be queued in a persistent outbox before the first delivery attempt. This ensures that messages are not lost if the remote server is temporarily unreachable or if the local server restarts between the time a message is created and the time delivery is confirmed.

## Retry and Backoff

Servers MUST retry failed delivery attempts with exponential backoff. The minimum initial retry interval, maximum retry interval, and total retry duration are implementation-defined, but implementations SHOULD apply a jitter factor to avoid synchronized retry storms from multiple servers.

A delivery attempt is considered failed when the remote server returns an error response, when the connection cannot be established, or when no response is received within an implementation-defined timeout.

## Group Chat Delivery

For group chats, the sender delivers independently to each participant's mailbox and tracks per-recipient state in the `deliveryReceipts` field of the outgoing Message record. The aggregate `deliveryState` advances to `"delivered"` when all participants have acknowledged receipt.

A message whose `senderMsgId` is already known for the given chat at the receiving server MAY be silently discarded by that server. Sending servers SHOULD treat a successful response (even for a discarded duplicate) as confirmation that delivery is complete for that recipient.

## Delivery State Transitions

The `deliveryState` field on an outbound Message transitions as follows:

- `"pending"` — Initial state. The message is queued but no successful delivery has occurred.
- `"delivered"` — All recipients (for group chats: all participants) have acknowledged receipt.
- `"failed"` — Delivery has been abandoned after exhausting retries for one or more recipients.
- `"received"` — The message has been explicitly read by the recipient (indicated via `Peer/receipt`).

# Security Considerations {#security}

## Identity Verification {#identity-verification}

The `senderUserId` argument in all Peer/* methods is caller-supplied and MUST be treated as untrusted. The local server MUST obtain the verified identity from its own authentication layer independently and MUST compare it to `senderUserId` before any storage, state change, or side effect. This comparison MUST precede all effects of the method. If the values differ, the request MUST be rejected immediately.

## Blob Fetch and SSRF {#ssrf}

When fetching attachment blobs from peer-supplied `fetchUrl` values, servers MUST restrict outbound HTTP connections to the known peer address space. Specifically, servers MUST NOT follow redirects to addresses outside the peer's known IP range or hostname, and MUST NOT connect to link-local, loopback, or private-network addresses in response to a peer-supplied URL. Unrestricted fetches from peer-supplied URLs are a Server-Side Request Forgery (SSRF) vector that could allow a malicious peer to probe internal services.

The same restriction applies to any other context in which a server fetches from a URL supplied by a peer, including blob URIs carried in `Message.actions` or `ChatContact.endpoints`.

## Chat ID Integrity {#chat-id-integrity}

Chat IDs are server-assigned ULIDs. Security against cross-conversation injection relies on sender authentication and chat membership verification, not on ID unpredictability.

For direct chats, the receiving server MUST confirm that the incoming `chatId` matches the chatId already associated with the sending contact, if one exists. This prevents a sender from injecting messages into a chatId that belongs to a different conversation.

For group and channel chats, servers MUST confirm the sender is a current member of the identified chat before accepting any `Peer/deliver` or `Peer/groupUpdate` call.

## Direct Chat Simultaneous Initiation {#direct-chat-race}

A race condition exists when two servers simultaneously initiate a direct chat with each other. Each server assigns its own ULID as the chatId before any delivery occurs, resulting in two distinct chatIds for the same logical conversation. This is a known limitation of the stateless ULID assignment model.

Implementations SHOULD resolve such conflicts using lexicographic ULID ordering: when a `Peer/deliver` arrives for a direct chat with an unknown chatId X, and the receiving server already has a direct chat chatId Y with the same sender, both servers SHOULD adopt `min(X, Y)` (lexicographically smaller) as the canonical chatId. The receiving server SHOULD migrate any messages stored under the non-canonical chatId to the canonical one, and SHOULD return the canonical chatId in the `Peer/deliver` response so that the sending server can update its records.

Servers that do not implement this resolution will produce duplicate direct-chat records for the affected pair. Client applications SHOULD detect and surface duplicate direct chats with the same contact to the user.

## Retract Authorization

`Peer/retract` MUST verify, via the `senderMsgId` index, that `senderUserId` matches the original sender of the identified message before applying any tombstone. Servers MUST NOT allow retraction of messages sent by other users.

## Group Admin Verification

`Peer/groupUpdate` MUST verify that `senderUserId` holds an admin role in the named group on the receiving server's local Chat record before applying any membership or metadata changes. For `"create"` actions, the sender MUST be listed as a member in the provided `members` array. Servers MUST reject requests where these conditions are not met.

## Direct Address Hints {#direct-address-security}

The `directAddress` field on ChatContact records and the `ownerDirectAddress` field in Session objects are peer-supplied values and MUST be treated as untrusted. Implementations that use these values for direct delivery MUST apply the same authentication requirements to the direct path as to the standard mailbox path. Servers MUST NOT use `directAddress` or `ownerDirectAddress` as `fetchUrl` targets for blob fetches or in any other context subject to the SSRF restrictions described in {{ssrf}}.

Senders MUST NOT treat successful delivery to a `directAddress` as a substitute for mailbox delivery. The standard mailbox path MUST always be used to ensure message persistence and multi-device visibility. Direct address delivery, when attempted, is an optimization only.

## Blocked Contacts

Messages from a contact whose `blocked` field is `true` MUST be silently dropped by the receiving server, regardless of whether they arrive in a direct chat or a group chat context. The server SHOULD return a success response to avoid disclosing the block status to the sender.

## Cross-Server Message Injection

A malicious peer could attempt to inject messages into chats it is not a party to by forging `chatId` or `senderUserId` values. The validation steps in {{new-message-validation}} — particularly the authenticated identity check in step 2, the chat membership check in step 3, and the chatId correspondence check for direct chats — collectively prevent this attack. Implementations MUST perform all of these steps and MUST NOT skip any of them.

## Presence Subscription Abuse

Servers implementing `Peer/subscribePresence` MUST authenticate the subscriber before recording the subscription (see {{peer-subscribepresence}}). Unauthenticated or mismatched `subscriberId` values MUST be rejected. Servers SHOULD limit the number of active presence subscriptions per peer to bound fan-out costs on presence changes. TTL-based expiry ensures that subscriptions from peers that have gone away do not accumulate indefinitely.

## End-to-End Encryption Relay Considerations

When the federation protocol is used in conjunction with end-to-end encrypted deployments (as described in {{JMAP-CHAT}}), the relay or forwarding server routes Peer/* messages but MUST NOT have access to plaintext message content. Servers operating in an encrypted deployment MUST ensure that `body` carries only ciphertext when `bodyType` indicates an encrypted payload type (e.g., `"application/mls-ciphertext"`). The encryption key schedule MUST exclude the relay (e.g., by using MLS {{RFC9420}} or a similar protocol that does not involve the relay in key agreement).

Metadata visible to relay servers — including sender id, recipient id, timestamp, and body size — remains an information-leakage surface even in encrypted deployments. Deployments with strong metadata-privacy requirements SHOULD apply message padding and cover traffic at the transport layer; those techniques are outside the scope of this document.

# IANA Considerations

This document does not request any new IANA registrations. The `urn:ietf:params:jmap:chat` capability URI is registered by {{JMAP-CHAT}}. This document extends the semantics of that capability by defining the `role: "peer"` account capability value and the Peer/* server-to-server methods; no additional capability URI is required.

Servers that implement this document MUST advertise the `urn:ietf:params:jmap:chat` capability in peer-role accounts as specified in {{account-capability}}.

--- back

# Acknowledgements

The author thanks the JMAP working group for {{RFC8620}} and the authors of {{RFC8615}} for the Well-Known URI mechanism that peer discovery relies upon.
