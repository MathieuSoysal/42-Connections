# Repository Copilot Instructions

These instructions give Copilot (and coding agents) enough context to extend the project without exploratory guesswork. Keep answers generic (not task-specific). Do not exceed patterns described here unless justified.

## 1. Project Purpose

Ingest public (and authenticated) data from the 42 API (https://api.intra.42.fr/apidoc) into MongoDB for later analysis (profiles, locations, events, event participations, etc.). Pattern: fetch JSON via OAuth2 (client credentials) → transform → insert/update MongoDB collections using idempotent operations and pagination/index tracking.

## 2. High-Level Architecture

Modules (src/):
- ft_api.rs: Thin HTTP layer. One function per endpoint family: request_profil, request_location, request_event_participations, request_event, generate_access_token. Central helper send_http_request().
- fetching_*.rs (fetching_profile, fetching_locations, fetching_event, fetching_event_participation): Orchestrate repeated calls, concurrency (futures::future::try_join), pacing (sleep_until), pagination loops.
- ft_mongodb_* (shown examples: ft_mongodb_profile_indexer, ft_mongodb_app_indexor): Persist indices and documents. Collections used as *work queues* (dequeue with find_one_and_delete; requeue via replace/upsert).
- Indexor (ft_mongodb_app_indexor.rs): Generic helpers to pop and reinsert user/event/page index.
- *_index collections:
  - profiles_index (holds profile ids to fetch)
  - location_index / events_participation_index (track (user_id, page_number))
  - events_ids (event ids)
- TIME_BETWEEN_REQUESTS (rate pacing) and NB_FETCH (batch loop size) drive ingestion loops.

Common flow for paginated resources:
1. Pop an index doc (user_id [+ page_number]).
2. Call ft_api::*.
3. If network/API failure → reinsert same doc (retry later).
4. If empty page → stop (do not reinsert).
5. If page size == 100 (full page) → increment page_number and reinsert.
6. Insert transformed documents (one per JSON array element or whole object).

## 3. Build / Test / Bench / Run

Prerequisites:
- Rust stable for normal build; nightly needed for benches (feature(test)).
- Docker required for tests using testcontainers (spins up temporary Mongo).
- MongoDB URI (tests use container; runtime may use local/remote).

Commands:
- Build: cargo build
- Tests (unit + async + testcontainers): cargo test
- Benches (nightly): RUSTC_BOOTSTRAP=1 cargo +nightly bench (lib uses #![feature(test)])
- Run (if a binary is later added): cargo run --bin <name> (currently library-focused)
- Lints (add clippy if configured later): cargo clippy --all-targets -- -D warnings

Always run cargo test before committing. If benches fail on stable, switch to nightly or disable the bench feature locally.

## 4. Environment / Configuration (expected)

Set (if/when main runtime binary added):
- FT_API_UID / FT_API_SECRET: 42 API OAuth client credentials (used by generate_access_token()).
- MONGODB_URI (default could be mongodb://localhost:27017).
Currently tests construct a dynamic URI from testcontainers; avoid hard-coding URIs in new code.

## 5. API Interaction Conventions

- All request_* functions:
  - Accept &AccessToken plus ids/page refs.
  - Return serde_json::Value (no custom struct decoding yet).
  - Treat 404 as non-fatal (resource absent) – only propagate other HTTP errors.
  - Log at debug on success, error on failure.
- Add new endpoints by cloning pattern: build URL, send_http_request, tolerant handling of 404 vs 200.

Rate Limiting / Throttling:
- Pacing controlled via TIME_BETWEEN_REQUESTS (seconds) using sleep_until for drift-avoiding scheduling.
- When adding new concurrent flows, reuse same pacing variable—do not hardcode sleeps.

## 6. MongoDB Patterns

Insertion:
- Use insert_many or replace_one with upsert(true) for idempotency.
- _id always derived from the 42 id (or composite (user_id, page_number) logic if needed).
Index Popping:
- find_one_and_delete(doc! {}) acts as a simple FIFO/LIFO queue surrogate (unordered pick). If the collection is empty → treat ingestion as complete (propagate error upward unless explicitly handled).
Error Handling:
- On transient/API error: requeue (reinsert same document).
- On completion (empty page or null body): do not requeue.

When adding a new paginated ingestion:
collection name: <resource>_index
document schema:
{ "_id": <primary_id>, "page_number": <i32> }   (only if pagination needed)

## 7. Logging & Error Conventions

- info!: high-level progress (fetched X, inserted Y).
- debug!: per-request fine-grained traces.
- warn!: (profiles) non-fatal absence (e.g. profile not found).
- error!: network/parsing failures or unexpected BSON mismatch.
- Never panic! in production-path functions; return Result and let orchestrator decide requeue. (Current convert_json_profile_to_bson panics—if modifying, prefer graceful fallback.)

## 8. Concurrency

- For dual-token parallelism use futures::future::try_join(...)
- Ensure both futures are independent (no shared mutable state aside from Mongo).
- Avoid spawning unbounded tasks—stay deterministic for rate control.

## 9. Adding Support For A New 42 Resource (Template)

1. Define request_<resource>() in ft_api.rs following existing pattern.
2. Create ft_mongodb_<resource>.rs with:
   - Transformation (serde_json::Value → Vec<Document> or Document)
   - insert_<resource>_in_mongodb()
3. (If paginated per user) Add an index collection <resource>_index mirroring existing design.
4. Create fetching_<resource>.rs with:
   - fetch_<resource>_from_42_to_mongo(...)
   - Optional double_fetch_<resource>_... if parallel tokens beneficial.
5. Integrate into orchestration loop(s) or create a new driver.
6. Respect TIME_BETWEEN_REQUESTS and NB_FETCH.
7. Add unit tests:
   - Pure transformation tests (no network)
   - Integration tests using testcontainers for Mongo (follow existing style).
8. Ensure all new request_* functions propagate only meaningful errors; treat 404 as "resource absent".

## 10. Testing Strategy

- Unit tests isolate pure helpers & parsing (fast).
- Async integration tests spin ephemeral Mongo container (may be slower—keep minimal).
- Avoid external HTTP in tests (mock or skip). If adding HTTP mocking later, use a feature flag.
- Benchmarks: Only for pure logic (e.g., fizz_buzz_fibonacci) – do not benchmark network or Mongo I/O.

## 11. Naming & Style

- snake_case for modules/functions; _id fields consistent with Mongo.
- request_* for raw API calls; fetch_* for orchestration; insert_* for Mongo persistence.
- Avoid abbreviations beyond established ones (uid, id, nb for count is already used; prefer count/total in new code).
- Return Result<T, Box<dyn Error>> for fallible async functions to maintain uniform signature.

## 12. Safety / Robustness Checklist (Apply When Modifying)

- Before inserting, validate expected JSON shape (array vs object).
- Guard unwraps; prefer if let / matches.
- Do not assume pagination size constant—logic already relies on threshold (< 100) meaning "last page".
- When changing index schema, update both insertion and popping logic atomically.

## 13. Benchmarks (Current)

- lib.rs contains example computational benches (fizz/buzz/fibonacci). Keep side-effect-free.
- If adding ingestion performance benches, isolate behind a cargo feature and avoid live network.

## 14. When To Search vs Trust These Instructions

Agents should trust these conventions first. Only perform repository-wide search if:
- A referenced module/file is missing.
- A new pattern (e.g., compression, caching) is required.
- Build or tests fail due to outdated assumptions.

## 15. Common Pitfalls (Avoid)

- Forgetting to requeue on transient failure → data gaps.
- Panicking inside async fetch loops → aborts ingestion batch.
- Hardcoding secrets or URIs.
- Busy-wait sleeping instead of sleep_until.
- Mixing i32/i64 for ids without normalizing (normalize to i64 internally; store as int in Mongo).

## 16. Future Improvements (Non-Binding)

(Do not implement unless explicitly requested.)
- Structured models (serde derive) instead of raw Value.
- Central retry/backoff strategy.
- Metrics or Prometheus instrumentation.
- Batching index pops to improve throughput.

## 17. Rust Conventions (Supplement from shared org guidelines)

Follow idiomatic, safe Rust (see rust-lang book & API guidelines). Key points for suggestions inside this repo:

### Core Principles
- Prefer readability + safety over micro-optimizations.
- Minimize cloning; prefer borrowing (&T / &mut T). Only clone when ownership transfer or lifetime mismatch requires it.
- Keep functions small and cohesive; extract pure helpers for complex logic.
- Avoid panics (`unwrap`, `expect`) in library/ingestion paths; propagate errors with `?`.
- Use Result<T, E> for recoverable errors; Option<T> for absence; no sentinel values.
- Document non-obvious reasoning with short comments (why, not what).

### Error Handling
- Wrap external failures early (network, BSON, JSON) with context in log messages.
- When adding custom error types, prefer thiserror or anyhow (not yet introduced—only add if requested).
- Never swallow errors silently; log at appropriate level before returning Ok(()).

### API & Types
- Use explicit integer widths (i64 for 42 ids; i32 for page_number) consistently.
- Prefer &str parameters unless ownership or mutation required.
- Avoid boolean parameters that hide intent—introduce enums if future branching grows.

### Concurrency & Async
- Use async/await; do not block threads (no std::thread::sleep).
- Keep parallelism bounded (currently pairs via try_join). If expanding, justify concurrency fan-out.
- Avoid interior mutability unless necessary; prefer passing immutable references.

### Collections & Iteration
- Use iterators instead of index loops; chain lazily and collect only when needed.
- Reserve capacity when building large Vec<Document> (e.g., with_capacity(array_len)) if you add transformations.

### Formatting & Linting
- Always cargo fmt and cargo clippy --all-targets -- -D warnings before commit.
- Keep lines ~100 chars max.
- Add rustdoc (///) only for public APIs or complex helpers when crate gains public surface.

### Testing
- For pure data transforms: unit tests inside same module.
- For integration with Mongo: keep minimal and deterministic (already using testcontainers).
- Avoid network in tests; if future mocking is added, gate behind feature flags.

### Benchmarks
- Benchmark only pure, deterministic code (no I/O, no randomness). Do not add benches for HTTP/Mongo.

### Patterns to Avoid
- unwrap/expect in production paths.
- Global mutable state or singletons.
- Deep nesting; prefer early return (guard style).
- Premature allocation or collect on iterators that can stay lazy.
- Introducing unsafe without strong justification and comments (avoid for now).

### Quality Checklist Before Adding Code
1. Compiles without warnings (clippy + fmt clean).
2. No unwrap/expect in new ingestion logic.
3. Errors logged with context (ids, page_number).
4. Tests cover edge cases (empty page, 404, type mismatch).
5. No magic numbers (reuse TIME_BETWEEN_REQUESTS, NB_FETCH, page size constant 100 consider const PAGE_SIZE if reused widely).

Agents: trust these conventions; search only if something conflicts or a referenced item is missing.

---
End of instructions.