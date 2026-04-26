# jmap-chat-ui

A terminal UI (TUI) client for the [JMAP Chat](docs/draft-atwood-jmap-chat-00.md) protocol.

## What

`jmap-chat-ui` is a generic, server-agnostic chat client for the terminal. It speaks
JMAP Chat over HTTPS and works with any conforming JMAP Chat server regardless of
deployment topology (mailbox-per-user or relay) or authentication mechanism.

The project is a two-crate Cargo workspace:

```
jmap-chat-ui/
├── Cargo.toml
└── crates/
    ├── jmap-chat/        (lib) — protocol types + HTTP client; no ratatui dependency
    └── jmap-chat-tui/    (bin) — ratatui TUI; depends on jmap-chat
```

**`jmap-chat`** is a reusable protocol library. It can be vendored by other clients
(bots, GUI clients, test harnesses) without pulling in any TUI dependency.

**`jmap-chat-tui`** is the terminal binary. Three-panel layout: chat list on the left,
message view in the center, compose bar at the bottom.

```
+--------------------------------------------------------------+
| jmap-chat-tui  [alice@example.com]              [Connected]  |
+--------------+-----------------------------------------------+
| Chats        | # general                                     |
|              |                                               |
| DMs          | bob  10:14  hello world                       |
|  > bob       | alice 10:15 hi bob                            |
|    carol     | bob  10:16  how's it going                    |
|              |                                               |
| Groups       |                                               |
|    team      |                                               |
|              |                                               |
| Spaces       |                                               |
|  > acme      |                                               |
|    # general |                                               |
|    # ops     |                                               |
|              +-----------------------------------------------+
|              | > _                                           |
+--------------+-----------------------------------------------+
```

Keyboard shortcuts:

| Key | Action |
|-----|--------|
| `Tab` / `Shift-Tab` | Cycle focus between panels |
| `j` / `k` or arrow keys | Navigate chat list / scroll messages |
| `Enter` on chat list | Open chat |
| `Enter` in compose bar | Send message |
| `PgUp` / `PgDn` | Scroll message history |
| `r` | Mark current chat as read |
| `q` or `Ctrl-C` | Quit |

## Why

JMAP Chat is a JMAP capability for direct and group text messaging. Multiple servers
implement or are building toward it (e.g., kith, a Tailscale-native mailbox daemon).
Each historically shipped its own tightly coupled client. This project provides a single
TUI binary that any JMAP Chat server operator can hand to their users, and a protocol
library that other client authors can build on.

Design goals:

1. **Server-agnostic.** No assumptions about network topology, authentication mechanism,
   or server implementation.
2. **Auth-pluggable.** Auth configured at startup via flags or config file. Built-in:
   custom CA cert (DER), Bearer token, TLS client cert (mTLS), HTTP Basic, and
   unauthenticated (local dev).
3. **Spec-faithful.** Full JMAP Chat object model as defined in the draft. No shortcuts
   that would break interop with future servers.
4. **Real-time.** Live updates via the JMAP EventSource (SSE) mechanism per RFC 8620
   §7.3: `StateChange` event -> `/changes` -> reconcile; fallback to full `/get` on
   `cannotCalculateChanges`; exponential backoff on disconnect.

Out of scope: server-side federation (Peer/* methods), E2EE key management (encrypted
bodies are displayed as `[encrypted]`), attachment transfer (metadata displayed only in
MVP), and anything Tailscale- or deployment-specific.

## How

### Build

```bash
cargo build --release
```

The binary is at `target/release/jmap-chat-tui`.

### Run

```bash
# Custom trust root only (e.g., kith with self-signed DER cert)
jmap-chat-tui --url https://100.64.x.x:8008 --ca-cert /path/to/server.der

# Bearer token
jmap-chat-tui --url https://chat.example.com --bearer-token <token>

# HTTP Basic
jmap-chat-tui --url https://chat.example.com --basic-user alice --basic-pass s3cr3t

# Mutual TLS
jmap-chat-tui --url https://chat.example.com \
              --client-cert cert.der --client-key key.der

# Unauthenticated local dev server
jmap-chat-tui --url http://localhost:8080
```

Optional config file at `~/.config/jmap-chat-tui/config.toml` (Phase 5).

### Auth providers

| Flag(s) | Auth mechanism | Use case |
|---------|---------------|----------|
| _(none)_ | `NoneAuth` — no auth header | Local dev / test stub |
| `--bearer-token` | `BearerAuth` | Standard token-gated servers |
| `--basic-user` + `--basic-pass` | `BasicAuth` | HTTP Basic servers |
| `--ca-cert` | `CustomCaAuth` | Self-signed CA (e.g., kith) |
| `--client-cert` + `--client-key` | `ClientCertAuth` | Mutual TLS |

The `AuthProvider` trait is public in `jmap-chat`; downstream crates can add their own
implementations without forking.

### Protocol library

`jmap-chat` exposes:

- `types::` — all twenty JMAP Chat data types from the spec (`Chat`, `Message`,
  `Space`, `ChatContact`, `ReadPosition`, `PresenceStatus`, etc.)
- `jmap::` — JMAP core wire types (`JmapRequest`, `JmapResponse`, `Invocation`,
  `Id`, `UTCDate`)
- `auth::` — `AuthProvider` trait and the five built-in implementations
- `client::` — `JmapChatClient`: `fetch_session`, `call` (POST), `subscribe_events` (SSE)
- `methods::` — typed wrappers for each JMAP Chat method (`Chat/get`, `Message/set`, etc.)
- `error::` — `ClientError` enum

## References

- [`docs/draft-atwood-jmap-chat-00.md`](docs/draft-atwood-jmap-chat-00.md) — JMAP Chat
  specification (primary reference): data types, methods, SSE events, blob transfer,
  authorization model.
- [`docs/draft-atwood-jmap-chat-federation-00.md`](docs/draft-atwood-jmap-chat-federation-00.md)
  — JMAP Chat Federation (Peer/* server-to-server methods). Informational for this
  client — federation is implemented server-side.
- [RFC 8620](https://www.rfc-editor.org/rfc/rfc8620) — JMAP base protocol: session
  discovery, API POST, EventSource/SSE, blob upload/download.
- [RFC 8621](https://www.rfc-editor.org/rfc/rfc8621) — JMAP for Mail. Structural
  analogue; useful for understanding the JMAP object model.
- [ULID spec](https://github.com/ulid/spec) — identifier format used for `Id` values.

## License

MIT License

Copyright (c) 2024 Mark Atwood

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
