# jmap-chat-ui

user agents for jmap chat

Read PROJECT.md before starting any work.

## Build & Test

```bash
cargo test
cargo clippy --all-features -- -D warnings
cargo fmt --check
cargo doc --no-deps --all-features
```

**Pre-commit gate:**
```bash
cargo fmt --all
typos src/
cargo clippy --all-features -- -D warnings
cargo test
```

## Architecture

Two-crate Cargo workspace (neither crate exists yet — see PROJECT.md §Work Plan for phases):

```
jmap-chat-ui/
├── Cargo.toml              (workspace)
└── crates/
    ├── jmap-chat/          (lib) — protocol types + HTTP client, no ratatui dep
    └── jmap-chat-tui/      (bin) — ratatui TUI, depends on jmap-chat
```

### `jmap-chat` (library crate)

Auth-agnostic JMAP Chat client. Key modules:
- `types::` — all JMAP Chat data types from the spec (Chat, Message, Space, etc.)
- `jmap::` — JMAP core wire types (JmapRequest, JmapResponse, Invocation, Id, UTCDate)
- `auth::` — `AuthProvider` trait + NoneAuth / BearerAuth / BasicAuth / CustomCaAuth / ClientCertAuth
- `client::` — `JmapChatClient`: fetch_session, call (POST), subscribe_events (SSE)
- `methods::` — typed wrappers for each JMAP Chat method (Chat/get, Message/set, etc.)
- `error::` — `ClientError` enum

### `jmap-chat-tui` (binary crate)

ratatui TUI. Key modules:
- `config::` — CLI args + optional config file → `Config`
- `app::` — `AppState`: all mutable TUI state (chat list, messages, input, focus, scroll)
- `client_task::` — background async task: session fetch, JMAP polling, SSE stream, sends `AppEvent` to TUI
- `event::` — keyboard/mouse/resize input handling
- `ui::` — ratatui render pass: chat list panel, message panel, compose bar, status bar
- `main::` — entry point: parse config, build auth, spawn client_task, run event loop

### Real-time updates
SSE (EventSource) via RFC 8620 §7.3: `StateChange` event → call `/changes` → reconcile
`AppState`. On `cannotCalculateChanges`, fall back to full `/get`. Exponential backoff on
disconnect.

### Auth
Configured via CLI flags. `AuthProvider` abstracts reqwest `Client` construction (trust
root, client cert) and per-request header injection (Bearer, Basic). kith uses
`CustomCaAuth` (custom DER trust root) with no auth header.

## Coding Rules

- Do not commit or push without explicit user approval
- No dead code, no commented-out code, no TODO comments in committed code
- No `unwrap()` or `expect()` in production code paths; use `?` and `ClientError`
- Never log credentials: Bearer tokens, Basic passwords, client key bytes
- Treat every server response as untrusted input: validate shapes, handle unexpected/missing fields gracefully
- Use `#[non_exhaustive]` on enums that mirror spec-defined sets so future spec additions do not silently break match arms

## Testing Philosophy

**Test-forward.** Tests are written before or alongside the implementation. An issue is not closeable until its tests pass. Test writing is never deferred to a later phase.

**Independent oracle.** Every test must verify against an oracle that does not use the code under test.
- Serializing then deserializing with the same function and asserting equality proves nothing — it tests only internal consistency, not spec conformance.
- Acceptable oracles: hand-written JSON derived directly from spec examples; Python `json.dumps(...)` run once and committed; `jq` to reformat spec text.
- Fixture files live in `crates/jmap-chat/tests/fixtures/` as committed `.json` files.
- If a fixture was produced by a script, commit the script alongside the fixture.

**Never weaken tests.** Do not skip, `#[ignore]`, delete, or soften assertions to make tests pass. Fix the code. If the fix is out of scope, file a new beads issue and escalate.

**No fabricated results.** Never report a test suite as passing unless you have run it and seen zero failures.


<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
