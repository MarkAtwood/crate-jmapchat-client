# jmapchat-client

An auth-agnostic Rust client library for the [JMAP Chat](https://github.com/MarkAtwood/jmap-chat-spec) protocol.

## What

`jmapchat-client` is a protocol library. It handles session discovery, JMAP method calls, SSE event streaming, WebSocket ephemeral events, and blob upload/download. It has no UI dependency and is intended to be embedded in chat clients, bots, test harnesses, or any other JMAP Chat consumer.

## Why

JMAP Chat is a JMAP capability for direct and group text messaging. Multiple servers implement or are building toward it. Each historically shipped its own tightly coupled client. This library provides a reusable protocol foundation that any client author can build on without reimplementing wire types, auth handling, or SSE parsing.

Design goals:

1. **Server-agnostic.** No assumptions about network topology, authentication mechanism, or server implementation.
2. **Auth-pluggable.** `AuthProvider` and `TransportConfig` traits compose independently — any credential scheme with any TLS configuration.
3. **Spec-faithful.** Full JMAP Chat object model from the draft. `#[non_exhaustive]` on spec-mirroring enums so future spec additions do not silently break match arms.
4. **Real-time.** SSE stream via RFC 8620 §7.3 and WebSocket ephemeral push via draft-atwood-jmap-chat-wss-00.

## API

```rust
use jmapchat_client::{DefaultTransport, BearerAuth, JmapChatClient};

let client = JmapChatClient::new(DefaultTransport, BearerAuth::new("tok")?, "https://chat.example.com")?;
let session = client.fetch_session().await?;
let sc = client.with_session(&session);
let chats = sc.chat_get(None, None).await?;
```

### Auth providers

| Type | Mechanism |
|------|-----------|
| `NoneAuth` | No `Authorization` header (local dev / test stub) |
| `BearerAuth` | `Authorization: Bearer <token>` |
| `BasicAuth` | `Authorization: Basic <base64(user:pass)>` |
| `DefaultTransport` | Publicly-trusted TLS, 10-second connect timeout |
| `CustomCaTransport` | Custom DER trust root (e.g., self-signed server cert) |

`AuthProvider` and `TransportConfig` are public traits; downstream crates can implement their own without forking.

### Modules

- `auth` — `AuthProvider` + `TransportConfig` traits and built-in implementations
- `client` — `JmapChatClient`: `fetch_session`, `call`, `call_batch`, `subscribe_events`, `connect_ws`, `upload_blob`, `download_blob`
- `jmap` — JMAP core wire types: `JmapRequest`, `JmapResponse`, `Invocation`, `Id`, `UTCDate`, `Session`
- `types` — all JMAP Chat data types from the spec
- `methods` — typed wrappers for every JMAP Chat method (`chat_get`, `message_set`, `space_join`, etc.) via `SessionClient`
- `sse` — `SseFrame` / `SseEvent`: parsed SSE frames from the event source
- `ws` — `WsSession` / `WsFrame`: WebSocket session for ephemeral push events
- `error` — `ClientError` enum

### Build and test

```bash
cargo test
cargo clippy --all-features -- -D warnings
cargo fmt --check
cargo doc --no-deps --all-features
```

## Spec references

- [draft-atwood-jmap-chat-00](https://github.com/MarkAtwood/jmap-chat-spec/blob/main/draft-atwood-jmap-chat-00.md) — primary spec: data types, methods, SSE events, blob transfer, authorization model
- [draft-atwood-jmap-chat-wss-00](https://github.com/MarkAtwood/jmap-chat-spec/blob/main/draft-atwood-jmap-chat-wss-00.md) — WebSocket ephemeral push (`ChatStreamEnable`, `ChatTypingEvent`, `ChatPresenceEvent`)
- [draft-atwood-jmap-chat-push-00](https://github.com/MarkAtwood/jmap-chat-spec/blob/main/draft-atwood-jmap-chat-push-00.md) — rich push notifications (`ChatMessagePush`, `PushSubscription` extension)
- [draft-atwood-jmap-cid-00](https://github.com/MarkAtwood/jmap-chat-spec/blob/main/draft-atwood-jmap-cid-00.md) — blob content identifiers (SHA-256 verification)
- [RFC 8620](https://www.rfc-editor.org/rfc/rfc8620) — JMAP base protocol: session discovery, API POST, SSE, blob upload/download
- [RFC 8887](https://www.rfc-editor.org/rfc/rfc8887) — JMAP over WebSocket

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
