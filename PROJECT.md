# jmap-chat-ui — Project Plan

## What This Is

A generic terminal UI (TUI) user agent for the JMAP Chat protocol, as specified
in `docs/draft-atwood-jmap-chat-00.md` and the federation companion
`docs/draft-atwood-jmap-chat-federation-00.md`.

"Generic" means: works with any conforming JMAP Chat server regardless of
deployment topology (mailbox-per-user or relay) or authentication mechanism.
The TUI is a client only — it implements no server-side Peer/* federation
methods, no outbox, and no relay logic.

## Why This Exists

Multiple projects are building or using JMAP Chat servers:

- **kith** (`~/PROJECT/kith`) — Tailscale-native mailbox-per-user daemon
  (`kithd`), AGPL-licensed. Has a working but kith-specific TUI (`kith-tui`)
  that is tightly coupled to Tailscale auth (custom DER cert as trust root,
  hardcoded for `https://100.64.x.x:8008`).
- **nie** (`~/PROJECT/Agora`) — E2E encrypted relay; uses its own WebSocket
  protocol, not JMAP Chat. Not a consumer of this TUI (yet), but the
  architecture is a reference for relay-topology considerations.
- Future JMAP Chat servers, test stubs, CI harnesses.

The goal is a single TUI binary that any JMAP Chat server operator can point
their users at. kith users should eventually be able to replace `kith-tui`
with this.

## Design Goals

1. **Server-agnostic.** Speaks JMAP Chat over HTTPS. Does not assume any
   particular auth mechanism, network topology, or server implementation.
2. **Auth-pluggable.** Auth is configured at startup via flags or a config file.
   Built-in mechanisms: custom CA cert (for self-signed kithd), Bearer token,
   TLS client cert, HTTP Basic. OAuth 2.0 is a stretch goal.
3. **Spec-faithful.** Implements the full JMAP Chat object model as defined in
   the draft. Does not cut corners that would break interop with future servers.
4. **Maintainable.** Two crates: a protocol library and the TUI binary. The
   library can be vendored by other clients (e.g., a future bot or GUI client)
   without pulling in ratatui.
5. **Real-time.** Uses the JMAP EventSource (SSE) mechanism for live updates,
   per RFC 8620 §7.3.

## Non-Goals / Explicit Exclusions

- **No server-side code.** No Peer/* federation methods, no outbox, no relay.
- **No kith-specific coupling.** No Tailscale API calls, no `WhoIs`, no
  `is_tailnet_ip`, no hardcoded kithd port. Auth for kith is just HTTPS with
  a custom trust root — that is one of the pluggable auth modes.
- **No E2EE key management.** The TUI sends and displays whatever the server
  delivers. Encrypted body types (e.g., `application/mls-ciphertext`) are
  displayed as `[encrypted]` with an indicator; the TUI does not participate
  in the MLS key schedule.
- **No federation.** The TUI connects to exactly one server (the user's
  mailbox or relay). Federation happens server-side.
- **No mobile/web.** Terminal UI only. A future companion crate could share
  the protocol library.
- **No attachment transfer in MVP.** Attachment metadata is displayed;
  open/save deferred to a later phase.

## Architecture

### Crate Structure

```
jmap-chat-ui/
├── Cargo.toml              (workspace)
├── crates/
│   ├── jmap-chat/          (lib) — protocol types + HTTP client
│   └── jmap-chat-tui/      (bin) — ratatui TUI
```

#### `jmap-chat` (library)

Auth-agnostic JMAP Chat client library. Depends on: `reqwest`, `serde`,
`serde_json`, `tokio`, `thiserror`, `futures`, `chrono`, `ulid`.

Modules:
- `types::` — all JMAP Chat data types from the spec:
  `Session`, `Chat`, `Message`, `ChatContact`, `ChatMember`, `Space`,
  `SpaceRole`, `SpaceMember`, `Category`, `SpaceInvite`, `SpaceBan`,
  `ReadPosition`, `PresenceStatus`, `CustomEmoji`, `Attachment`, `Mention`,
  `MessageAction`, `Reaction`, `MessageRevision`, `Endpoint`.
- `jmap::` — JMAP core wire types: `JmapRequest`, `JmapResponse`,
  `Invocation`, `Id`, `UTCDate`.
- `client::` — `JmapChatClient`:
  - `fetch_session()` — GET `/.well-known/jmap`, parse Session
  - `call()` — POST to `apiUrl`, parse response
  - `subscribe_events()` — GET `eventSourceUrl`, return SSE stream
- `auth::` — `AuthProvider` trait + built-in implementations:
  - `NoneAuth` — no auth header (for local dev/test)
  - `BearerAuth` — `Authorization: Bearer <token>`
  - `BasicAuth` — `Authorization: Basic <base64>`
  - `CustomCaAuth` — custom trust root (DER), used by kith
  - `ClientCertAuth` — mTLS with client cert + key (DER)
- `methods::` — typed wrappers for each JMAP Chat method call:
  `chat_get`, `message_get`, `message_set`, `chat_query`, `message_query`,
  `read_position_set`, `presence_status_set`, etc.
- `error::` — `ClientError` enum.

#### `jmap-chat-tui` (binary)

TUI binary. Depends on `jmap-chat`, `ratatui`, `crossterm`, `clap`, `tokio`,
`tracing`, `tracing-subscriber`.

Modules:
- `config::` — parse CLI args and optional config file into `Config`:
  server URL, auth method + credentials, optional `ownerUserId` override.
- `app::` — `AppState` — all mutable TUI state (chat list, messages, input,
  focus, connection status, scroll offset, presence map, typing indicators).
- `client_task::` — background async task: fetch session, run JMAP polling
  loop and SSE stream, send `AppEvent`s to TUI via mpsc channel.
- `event::` — input event handling (keyboard, mouse, terminal resize).
- `ui::` — ratatui render pass: three panels (chat list, message view,
  compose bar) + status bar.
- `main::` — entry point: parse config, build auth provider, spawn
  client_task, run terminal event loop.

### Auth Configuration

Auth is configured via CLI flags. The `AuthProvider` trait abstracts
the reqwest `Client` construction (different trust roots, client certs)
and per-request header injection (Bearer, Basic).

```
# kith (custom trust root only, no auth header)
jmap-chat-tui --url https://100.64.x.x:8008 --ca-cert /path/to/kithd.der

# Bearer token server
jmap-chat-tui --url https://chat.example.com --bearer-token <token>

# HTTP Basic
jmap-chat-tui --url https://chat.example.com --basic-user alice --basic-pass s3cr3t

# mTLS
jmap-chat-tui --url https://chat.example.com \
              --client-cert cert.der --client-key key.der

# Unauthenticated local dev server
jmap-chat-tui --url http://localhost:8080
```

### Real-Time Updates

1. On connect, fetch session and snapshot (chats, messages, contacts).
2. Open SSE connection to `eventSourceUrl`.
3. On `StateChange` event naming a changed type, call the corresponding
   `/changes` method and reconcile into `AppState`.
4. On `cannotCalculateChanges`, fall back to full `/get`.
5. SSE reconnect with exponential backoff on connection loss.

## TUI Layout

```
┌──────────────────────────────────────────────────────────────┐
│ jmap-chat-tui  [alice@example.com]              [Connected]  │
├──────────────┬───────────────────────────────────────────────┤
│ Chats        │ # general                                     │
│              │                                               │
│ DMs          │ bob  10:14  hello world                       │
│  > bob       │ alice 10:15 hi bob                            │
│    carol     │ bob  10:16  how's it going                    │
│              │                                               │
│ Groups       │                                               │
│    team      │                                               │
│              │                                               │
│ Spaces       │                                               │
│  > acme      │                                               │
│    # general │                                               │
│    # ops     │                                               │
│              ├───────────────────────────────────────────────┤
│              │ > _                                           │
└──────────────┴───────────────────────────────────────────────┘
```

Keyboard shortcuts:
- `Tab` / `Shift-Tab` — cycle focus between panels
- `j/k` or arrow keys — navigate chat list / scroll messages
- `Enter` on chat list — open chat
- `Enter` in compose bar — send message
- `PgUp/PgDn` — scroll message history
- `r` — mark current chat as read
- `q` or `Ctrl-C` — quit

## Work Plan

### Phase 1 — Protocol Library (`jmap-chat` crate)

1. Cargo workspace scaffold (`Cargo.toml`, `crates/jmap-chat/Cargo.toml`).
2. JMAP core wire types: `JmapRequest`, `JmapResponse`, `Invocation`, `Id`,
   `UTCDate`. Round-trip serde tests against hand-written JSON.
3. JMAP Chat data types: all types from spec, with serde derives. Unit tests
   for optional fields, enum variants, and nested objects.
4. `AuthProvider` trait + `NoneAuth`, `BearerAuth`, `BasicAuth`, `CustomCaAuth`
   implementations.
5. `JmapChatClient::fetch_session()` — with integration test against a
   hand-crafted JSON fixture.
6. `JmapChatClient::call()` — POST + response parse.
7. `JmapChatClient::subscribe_events()` — SSE stream returning
   `StateChange` events.
8. Typed method wrappers for: `Chat/get`, `Chat/query`, `Chat/changes`,
   `Message/get`, `Message/query`, `Message/changes`, `Message/set` (create
   only), `ChatContact/get`, `ReadPosition/get`, `ReadPosition/set`,
   `PresenceStatus/get`.

### Phase 2 — TUI Scaffold (`jmap-chat-tui` crate)

9. Binary crate scaffold, clap arg parse, config struct.
10. Terminal lifecycle management (raw mode, alternate screen, panic hook
    that restores terminal).
11. Static layout render with placeholder data (no network yet).
12. Keyboard event loop: focus cycling, quit.

### Phase 3 — Connect and Display

13. `client_task`: fetch session, fetch initial chat list + contacts.
14. Wire `AppState` to real chat list; render DMs / groups / spaces in
    left panel, sorted by `lastMessageAt` desc.
15. On chat select, fetch messages for that chat (`Message/query` +
    `Message/get`). Display in message panel with sender name and timestamp.
16. Scroll: `PgUp/PgDn`, `j/k`, auto-scroll to bottom on new message.

### Phase 4 — Send and Update

17. Compose bar: text input with UTF-8 cursor, `Enter` to send
    (`Message/set` create).
18. SSE integration: `StateChange` → `/changes` → update displayed messages
    and chat list in real time.
19. `ReadPosition/set` on chat open and on new message visible.
20. Typing indicator: send `PresenceStatus` updates on keystroke (rate-limited);
    display peer typing events from SSE.

### Phase 5 — Polish and Extended Features

21. Unread count badges on chat list entries.
22. Presence status display (online/away/offline dot next to contact name).
23. Reaction display inline on messages (emoji count summary).
24. Edit indicator (`[edited]` tag) and deletion tombstone (`[deleted]`).
25. Reply-to threading: show quoted snippet for `replyTo` messages.
26. `ClientCertAuth` (mTLS) auth provider.
27. Config file support (`~/.config/jmap-chat-tui/config.toml`).
28. Space/channel browsing: expand space to show channels in chat list,
    navigate channels.

### Phase 6 — Deferred / Stretch

- Attachment open/save (launch OS viewer for downloaded blob).
- Full-text search via `Message/query` with `text` filter.
- Thread view (indented replies, `supportsThreads` capability check).
- Space admin actions (invite, kick, role management).
- OAuth 2.0 auth provider.
- Mouse support (click to focus, click chat to open).
- Custom emoji rendering (fallback to shortcode name in terminal).

## Development Process

### Phases map to epics

Before starting any phase, file a beads epic and one issue per numbered step:

```bash
bd epic create --title "Phase N: <name>" --description "<goal>"
bd create --title "Step N: <summary>" --epic <epic-id> --type=task --priority=2
# one issue per numbered step; file all before starting any
```

Do not begin a phase without all its issues filed. Do not begin a step without claiming
its issue (`bd update <id> --claim`).

### Agent teams

Use `TeamCreate` to parallelize independent steps within a phase. Example split for Phase 1:
- Agent A: `types::` module (step 3)
- Agent B: `jmap::` wire types (step 2)
- Agent C: `auth::` + `client::` (steps 4–6)

Each agent owns its scope: claims its beads issue, reads only what it needs, closes the
issue when done. Agents do not touch files outside their scope.

### Test-forward

Every implementation step has tests written before or alongside the code. No issue is
closeable without passing tests. Test writing is never deferred to a later phase.

### Test oracle discipline

Tests must have an oracle independent of the code under test.

**For serde / wire-type tests:**
- Hand-write the expected JSON directly from spec examples. Commit it as a fixture in
  `crates/jmap-chat/tests/fixtures/`.
- Assert that deserialization produces the expected struct, and that re-serialization
  produces the same JSON (byte-for-byte after normalization).
- Do NOT serialize then deserialize and assert equality — this proves only internal
  consistency, not spec conformance.

**For HTTP client / method tests:**
- Use committed `.json` response fixtures, not live network calls.
- A local mock HTTP server (`wiremock` crate) is acceptable for integration tests.

**Generating fixtures:**
- Python `json.dumps(...)` or `jq` to normalize hand-crafted objects is acceptable.
- Commit the script that produced the fixture alongside the fixture file.

### Defensive coding

- No `unwrap()` or `expect()` in production code paths.
- Never log credentials (tokens, passwords, key bytes).
- Treat every server response as untrusted: validate structure, handle missing/extra fields.
- Use `#[non_exhaustive]` on spec-mirroring enums so future spec additions don't silently break match arms.

## Key Reference Material

- `docs/draft-atwood-jmap-chat-00.md` — primary JMAP Chat spec (data types,
  methods, SSE events, blob upload/download, authorization model)
- `docs/draft-atwood-jmap-chat-federation-00.md` — federation companion
  (Peer/* methods, peer discovery, `/.well-known/jmap` session fields);
  informational only for this client — we are not implementing federation
- RFC 8620 — JMAP base protocol (GET session, POST API, SSE, blob upload)
- `~/GIT/ideas/draft-atwood-jmap-chat-00.md` — older draft, superseded by
  the version in `docs/`
- `~/PROJECT/kith/crates/kith-tui/` — working reference TUI (kith-specific,
  ratatui + crossterm, reqwest); study for TUI structure and SSE client pattern
- `~/PROJECT/kith/crates/kith-core/` — working JMAP Chat type definitions
  and wire types; study for serde patterns, do NOT copy kith-specific types
  (`is_tailnet_ip`, `Role::Peer`, etc.)

## What NOT To Do

- Do not implement Peer/* server-to-server methods. This is a client.
- Do not add Tailscale, WhoIs, or overlay-network-specific code. Auth for
  kith is just HTTPS with a custom trust root.
- Do not couple to `kith-core`, `kith-jmap`, or any kith crate. Shared types
  would create a dependency on kith's license (AGPL) and release cadence.
  Reimplement the narrow set of types we need.
- Do not implement blob fetch/upload in Phase 1. Attachment display is
  metadata-only until Phase 5.
- Do not add features not listed above without updating this plan first.
- Do not use TodoWrite, TaskCreate, or markdown task lists for tracking.
  Use `bd create` for all work items.
