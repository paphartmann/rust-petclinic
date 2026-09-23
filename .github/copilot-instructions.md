# Copilot instructions for rust-petclinic

## Build, run, test, and lint

This repository has separate Cargo projects. The server workspace is rooted at
`server/`; `client/` and `dto/` each have their own manifest and lockfile.

### Server

Run these commands from `server/`:

```sh
cargo run
cargo test
cargo test get_owners_returns_owner_list
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
```

The server listens on `http://localhost:3000`. Useful smoke checks are:

```sh
curl -v http://localhost:3000/owners
curl -X POST http://localhost:3000/token \
  -H "Content-Type: application/json" \
  -d '{"user":"foo","password":"bar"}'
```

Server tests are inline in `server/src/main.rs`, use an in-memory SQLite
connection, run the migrations before each test, and exercise the Axum router
with `tower::ServiceExt::oneshot`. Pass a test-name substring to `cargo test`
to run one test.

### Client

The client is a Sycamore/WASM application. From `client/`:

```sh
trunk serve
cargo check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
```

`trunk serve` serves the client at `http://localhost:8080`; the running server
must be available because the client calls the API directly.

### Repository checks

The pre-commit configuration runs the `gitleaks` hook. CI also runs Rust
Semgrep, CodeQL, and an OWASP ZAP baseline scan against the `/owners`
endpoint.

## Architecture

- `server/` contains the Axum HTTP binary and is the Cargo workspace root for
  the `rust-petclinic`, `entity`, and `migration` crates.
- `server/src/main.rs` wires routes, CORS, the database extension, JWT
  authentication, request/response DTO mapping, and the inline integration
  tests. Current routes are `GET /owners`, `POST /owners/:owner_id/pets/new`,
  `POST /token`, and authenticated `GET /vets`.
- `server/entity/` contains SeaORM models and relations for owners, pets, pet
  types, vets, specialties, and the vet-specialty join table. The server maps
  these persistence models to API DTOs rather than exposing entity models
  directly.
- `server/migration/` owns the ordered SeaORM migrations and seed data. New
  migration modules must be declared and appended in
  `server/migration/src/lib.rs` in execution order.
- `dto/` is a separate shared Rust library used by both server and client.
  Changes to its serialized structs affect both the API payloads and the
  client, so update both sides together.
- `client/` is a Sycamore/WASM binary. `client/src/main.rs` defines the
  browser routes and page shell; `owners.rs` renders the owner list and
  `pet.rs` submits the new-pet form. HTTP requests use `reqwasm` and currently
  target `http://localhost:3000` directly.

The server connects to `sqlite::memory:` and runs `Migrator::fresh` followed by
`Migrator::up` at startup. Every process starts with a fresh schema and the
migration seed data; there is no persistent database configuration in the
repository. The same fresh-migration setup is required by the server tests.
The CORS policy currently allows the local Trunk origin
`http://localhost:8080`.

## Repository-specific conventions

- Keep server changes within the `server/` workspace and client changes within
  the independent `client/` Cargo project; there is no root-level
  `Cargo.toml`.
- Prefer the existing SeaORM relation/query patterns and explicit mapping into
  `dto` types at the HTTP boundary. Add persistence fields to the entity,
  migration, and DTO layers as appropriate rather than coupling the client to
  database models.
- Follow the migration naming/order pattern
  `mYYYYMMDD_NNNNNN_<description>.rs`, and register every migration in
  `Migrator::migrations()`.
- Preserve the current API paths, JSON field names, and date representation
  used by `dto::NewPet` (`NaiveDate` serialized as `YYYY-MM-DD`) when changing
  client or server behavior.
- Keep authentication enforcement in the Axum `Claims` request extractor for
  protected routes. The token endpoint currently accepts any non-empty
  username/password pair, and the signing secret is defined in the server
  source; do not treat this sample behavior as production authentication.
- Keep browser-facing URL, CORS, and route changes synchronized across
  `server/src/main.rs`, the Sycamore route definitions, and the Trunk client.
