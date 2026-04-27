# JMAP Chat Spec Improvements — Implementation Impact

Changes from OLD/ to current spec, from the perspective of implementing them in this user agent.

---

## 1. New Companion Specs (entirely new surface area)

### 1.1 WebSocket Transport (`draft-atwood-jmap-chat-wss-00`)

Capability: `urn:ietf:params:jmap:chat:websocket`

New capability on top of RFC 8887 WebSocket. Adds:
- `ChatStreamEnable` — client sends to subscribe to ephemeral events for a set of chatIds
- `ChatStreamDisable` — client sends to stop ephemeral events
- `ChatTypingEvent` — server pushes typing indicator over WebSocket
- `ChatPresenceEvent` — server pushes presence update over WebSocket

**Implementation work:**
- Current client_task uses SSE (EventSource). Detect this capability and optionally upgrade to WebSocket for lower latency.
- On reconnect, must re-send `ChatStreamEnable` to restore subscriptions (session-scoped, not persisted).
- `chatIds` in `ChatStreamEnable` is a view-management filter — keep in sync with which chats the UI has open.
- `ChatStreamEnable`/`ChatStreamDisable` is independent of `Chat.receiveTypingIndicators` (both can suppress typing events, evaluated independently).

### 1.2 Rich Push Notifications (`draft-atwood-jmap-chat-push-00`)

Capability: `urn:ietf:params:jmap:chat:push`

Extends RFC 8620 `PushSubscription` with `chatPush` property. Server delivers `ChatMessagePush` payloads — message metadata plus optional body snippet — directly to the push endpoint without a follow-up fetch.

**Implementation work:**
- When registering a `PushSubscription`, set `chatPush` with per-chat filters and urgency preferences.
- Handle `ChatMessagePush` payloads on the push receive path before attempting `Message/changes`.
- For mentions, request `high` urgency; normal messages can use `normal`.
- In E2EE mode, server delivers metadata-only (no body snippet); adjust notification rendering accordingly.
- Account-level capability exposes `maxSnippetBytes`, `supportedUrgencyValues`, `maxMessagesPerPush`.

### 1.3 Blob Content Identifiers (`draft-atwood-jmap-cid-00`)

The `sha256` field in upload responses is now normatively defined by a companion spec rather than inline. Behavior is unchanged, but the client verification guidance and security considerations now live there.

**Implementation work:** No code change needed; the upload response shape is the same. Just source the verification logic from JMAP-CID rather than the old inline text.

---

## 2. ChatContact: New Fields

### 2.1 `presence` field — expanded value set and now optional

OLD: `"online"`, `"away"`, `"offline"`, `"unknown"` — always present.
NEW: `"online"`, `"away"`, `"busy"`, `"invisible"`, `"offline"` — now optional (absent = unknown to this server).

**Implementation work:**
- Add `Busy` and `Invisible` variants to the presence enum.
- Handle absent `presence` field (treat same as unknown — do not assume offline).
- UI: add distinct display for `busy` (typically a red/DND indicator) and `invisible` (show as offline to others but not yourself).

### 2.2 `statusText` and `statusEmoji` — new fields on ChatContact

Server mirrors the contact's `PresenceStatus.statusText` and `PresenceStatus.statusEmoji`, updated when a presence event arrives from that server.

**Implementation work:**
- Add `status_text: Option<String>` and `status_emoji: Option<String>` to the `ChatContact` type.
- Display alongside presence indicator in the contact list and chat headers.
- These arrive via `ChatContact/changes` and also via real-time presence push events (see §3.2).

---

## 3. Chat: New Fields and Behavior Changes

### 3.1 `receiveTypingIndicators` (Boolean, default `true`) — new field

Per-chat preference. When `false`, server silently drops typing push events for this chat before delivering to the owner. Sender is not informed.

**Implementation work:**
- Add field to `Chat` type.
- Expose a toggle in chat settings UI ("Show typing indicators").
- Call `Chat/set` with `receiveTypingIndicators: false` to disable.
- Client must use a decay timer regardless: if no typing event for a `(chatId, senderId)` pair within 10 seconds, hide the indicator. This handles the suppression-to-enabled transition without stale state.

### 3.2 `receiptSharing` (Boolean, optional) — new per-chat field

Overrides the account-level `PresenceStatus.receiptSharing` for a single chat. When absent, account-level applies.

**Implementation work:**
- Add `receipt_sharing: Option<bool>` to `Chat` type.
- Expose per-chat override in settings ("Share read receipts for this chat").
- Call `Chat/set` with `receiptSharing` to override.

### 3.3 `unreadCount` — clarified definition

Now explicitly: count of Messages whose `id` ULID is lexicographically greater than `ReadPosition.lastReadMessageId`. If `lastReadMessageId` absent, all messages are unread.

**Implementation work:** No API change. Clarifies the semantics for local display logic.

### 3.4 `pinnedMessageIds` — channel chat rule added

For channel chats: only members with the `"pin"` Space permission may modify the list. (Direct: owner freely; group: admins only.)

**Implementation work:** Check `"pin"` permission before showing pin/unpin UI in channel chats.

### 3.5 `slowModeSeconds` — error response specified

When a message is rejected for violating slow mode, server returns `rateLimited` (specific error code, not generic).

**Implementation work:** Handle `rateLimited` SetError in `Message/set` response. Show countdown to next permitted send time in the compose bar.

---

## 4. Message: New Fields and Clarified Behavior

### 4.1 `deliveryReceipts` — new `deviceDeliveredAt` per-recipient field

OLD: per-recipient object was `{"deliveredAt": ..., "readAt": ...}`.
NEW: `{"deliveredAt": ..., "deviceDeliveredAt": ..., "readAt": ...}`.

`deviceDeliveredAt` is optional (absent if the recipient's platform cannot confirm device-level delivery). No top-level parallel field.

**Implementation work:**
- Update `DeliveryReceipt` struct to add `device_delivered_at: Option<UTCDate>`.
- UI: optionally show a distinct "delivered to device" indicator (double-tick style) separate from "delivered to server".

### 4.2 `burnOnRead` — interaction with `receiptSharing: false` clarified

When recipient has `receiptSharing: false`, server still sets `readAt` locally and fires the hard-delete. The sending server receives no `Peer/receipt` confirmation. Senders MUST NOT rely on receipt confirmation to verify burn-on-read occurred.

**Implementation work:** No client behavior change. Clarifies that burn-on-read fires silently even when receipts are suppressed — do not retry or warn user if no receipt comes back for a burnOnRead message.

---

## 5. PresenceStatus: New Field

### 5.1 `receiptSharing` (Boolean, default `true`) — new account-level field

When `false`, server suppresses all outbound `Peer/receipt` calls. Owner does not broadcast read times. The opt-out is bidirectional: an account that does not broadcast read times also does not receive others' read times (within affected scope).

**Implementation work:**
- Add `receipt_sharing: bool` to `PresenceStatus` type.
- `PresenceStatus/set` now accepts `receiptSharing` in the update.
- Expose in account settings ("Share read receipts").
- When `false`, suppress display of `readAt` timestamps from others (server will not deliver them anyway, but guard against stale cached values).
- UI: coarsen displayed `readAt`/`lastReadAt`/`deviceDeliveredAt` timestamps to hour or day granularity; store full precision internally.

---

## 6. New Method: `Chat/typing`

OLD: typing was an implicit push-only event with no defined client→server method.
NEW: formally defined JMAP method.

Request: `accountId`, `chatId`, `typing` (Boolean).
Response: `accountId`.

Server behavior:
- Does NOT persist the event.
- Checks `receiveTypingIndicators` per recipient before delivering.
- Rate-limits: max 1 call per account per chat per 3 seconds; excess silently discarded.

**Implementation work:**
- Add `Chat/typing` to the methods module.
- Call on keypress (debounce to ~1s, send `typing: true`); send `typing: false` on send or idle timeout.
- Respect the 3-second rate limit in the client to avoid hammering the server.
- Handle silently (no error expected; server may discard excess calls without error).

---

## 7. SpaceInvite: New `SpaceInvite/changes` Method

OLD: `SpaceInvite` had no `/changes` method.
NEW: `SpaceInvite/changes` added, with the same visibility rules as `SpaceInvite/get`.

**Implementation work:**
- Add `SpaceInvite/changes` to the methods module.
- Include `SpaceInvite` state in `StateChange` subscription and poll loop.

---

## 8. CustomEmoji: New `CustomEmoji/queryChanges` Method

OLD: `CustomEmoji` had no `/queryChanges` method.
NEW: `CustomEmoji/queryChanges` added.

**Implementation work:**
- Add `CustomEmoji/queryChanges` to the methods module.
- Can now efficiently maintain a local emoji cache via query + queryChanges pattern.

---

## 9. ReadPosition: Clarified Lifecycle

Now explicit: ReadPosition is created when the first message is delivered (direct/group) or when the owner joins the Space (channel). Destroyed when the Chat is destroyed.

Also: the receipt-sharing opt-out is bidirectional — when `receiptSharing` is `false`, server suppresses inbound `Peer/receipt` events too, so `ReadPosition.lastReadAt` from others is not delivered.

**Implementation work:** No API change. Confirms that channel chats always have a ReadPosition once joined; safe to assume its existence without defensive null checks.

---

## 10. Blob Storage: New Capabilities

### 10.1 `JMAP-BLOBEXT` (`urn:ietf:params:jmap:blob2`) supersedes RFC 9404

Server SHOULD advertise `blob2`. New capabilities:
- `Blob/lookup` — reverse lookup: given a blobId, find all Messages referencing it as an attachment.
- `Blob/convert` — server-side image conversion (thumbnail generation).

**Implementation work:**
- Detect `blob2` capability in the session.
- Use `Blob/lookup` to implement "find all messages with this attachment" search.
- Use `Blob/convert` to request thumbnail previews for image attachments instead of downloading full blobs.

### 10.2 VAPID for Web Push

Server SHOULD advertise `urn:ietf:params:jmap:webpush-vapid` and sign push messages with a VAPID JWT.

**Implementation work:** When subscribing to push, extract and store the server's VAPID public key from the capability object. Pass to the browser/platform push service when registering the subscription endpoint.

---

## 11. Endpoint Types: Expanded

OLD: 3 types (`vtc`, `payment`, `blob`).
NEW: 4 additional types: `calendar-event`, `availability`, `task`, `filenode`.

**Implementation work:**
- Add enum variants for the new types (or keep as extensible string with recognized-type handling).
- UI: render calendar-event actions as "View event", availability as "Check free/busy", task as "View task", filenode as "Open file".
- All OOB URIs remain untrusted; require explicit user initiation.
- Unknown types: silently ignore (changed from MUST to SHOULD — same practical behavior).

---

## 12. JMAP Protocol Integrations (informative, but affects client ergonomics)

### 12.1 JMAP RefPlus (`urn:ietf:params:jmap:refplus`)

Server SHOULD support RefPlus. Allows result references inside `/set` create/update bodies and inside `/query` filter conditions within a single request array.

**Implementation work:** When server advertises RefPlus, use it to chain Message creation → Message/query in one round-trip instead of two. Reduces latency for send-and-scroll-to-message patterns.

### 12.2 JMAP Quotas (`urn:ietf:params:jmap:quotas`)

Server SHOULD implement `Quota` extension and register `Message`, `Chat`, and `Space` as data types.

**Implementation work:** Poll `Quota/get` to display storage usage. Warn user when approaching limits.

### 12.3 JMAP Metadata (`urn:ietf:params:jmap:metadata`)

Server MAY support per-object annotations on `Chat`, `Message`, and `Space` objects.

**Implementation work:** Optional. Can use for per-chat color tags, per-message bookmarks, per-space labels — all local to this client without modifying shared state.

---

## 13. Rich Body Format: Encoding Clarification

OLD: `body` for `application/jmap-chat-rich` "MUST be a valid JSON object".
NEW: `body` "MUST contain a JSON-encoded string whose top-level parsed value is an object".

This means the `body` field itself is a string containing escaped JSON, not a raw JSON object embedded in the JMAP response.

**Implementation work:** When rendering rich messages, `serde_json::from_str::<RichBody>(&message.body)` — parse the body string as JSON, not treat it as a pre-parsed object. If the current implementation treats it as a raw object, this is a bug to fix.

---

## 14. Thread Model: Softened Client Obligation

OLD: "Clients MUST follow these rules" (for `threadRootId` assignment).
NEW: "Clients SHOULD follow these rules."

Consequence of incorrect assignment: message doesn't appear in thread query results and `replyCount` is wrong. Not a delivery failure.

**Implementation work:** No change needed. Already following the rules. The softening just means a non-conformant client won't cause a server error.

---

## 15. Security/Privacy: New Guidance for the UI

### 15.1 Read receipt timestamp coarsening

NEW: "Clients SHOULD NOT expose sub-minute precision for `readAt`, `lastReadAt`, or `deviceDeliveredAt` in the UI. Displaying relative representations (today, yesterday, hour-granularity) reduces behavioral pattern exposure."

**Implementation work:** Format delivery/read timestamps as relative or hour-granularity strings in the UI. Store full-precision values internally for sorting.

### 15.2 Typing indicator suppression is private

When `receiveTypingIndicators` is `false`, the sender's `Chat/typing` call succeeds normally — they cannot detect suppression.

**Implementation work:** No behavior change. Confirms the server design; the client just sends normally and doesn't need to handle any suppression signal.

---

## Summary: What Needs to Be Built or Changed

| Area | Effort | Notes |
|---|---|---|
| WebSocket transport + `ChatStreamEnable`/`ChatStreamDisable` | High | New connection mode alongside SSE |
| Rich push notifications (`ChatMessagePush`) | Medium | New push payload type |
| `Chat/typing` method | Low | Formalized; replaces any ad-hoc approach |
| `receiveTypingIndicators` field + UI toggle | Low | New Chat field |
| `receiptSharing` fields (account + per-chat) | Medium | Two new fields, new UI controls |
| `PresenceStatus.receiptSharing` | Low | New field on existing singleton |
| ChatContact `statusText`/`statusEmoji` | Low | New display fields |
| `presence` expanded enum + optional | Low | Two new variants, absent handling |
| `deliveryReceipts.deviceDeliveredAt` | Low | New optional field |
| `SpaceInvite/changes` | Low | Missing method |
| `CustomEmoji/queryChanges` | Low | Missing method |
| Blob2 / `Blob/lookup` / `Blob/convert` | Medium | New blob capabilities |
| VAPID push key handling | Low | Extract from capability, pass to push service |
| Endpoint new types (calendar, task, etc.) | Low | Enum extension |
| JMAP Quota display | Low | Optional but useful |
| JMAP RefPlus chaining | Low | Optimization, not correctness |
| Rich body encoding fix (string-of-JSON) | Low | May be a current bug — verify |
| Timestamp coarsening in UI | Low | Display-only change |
| `rateLimited` error handling in Message/set | Low | New error variant |
