# Axum route CodeQL queries

The `axum-routes.ql` diagnostic query lists method calls shaped like Axum route
registrations: `.route(path, method(handler))`. It reports the path, HTTP
method, and handler function in a diagnostic message.

The `axum-unprotected-mutations.ql` security query reports registered `POST`,
`PUT`, `PATCH`, and `DELETE` routes whose handler does not use the `Claims`
authentication extractor. The `/token` route is excluded because it is the
public credential-exchange endpoint. This check follows this application's
handler-level authentication convention; it does not infer authentication
provided by route or router middleware.

From the repository root, create a local Rust database and run the query:

```sh
codeql database create /tmp/rust-petclinic-codeql-db \
  --language=rust \
  --source-root=server \
  --command='cargo check --workspace'

codeql pack install .github/codeql/axum-route-inventory

codeql query run \
  --database=/tmp/rust-petclinic-codeql-db \
  --output=/tmp/axum-routes.bqrs \
  .github/codeql/axum-route-inventory/axum-routes.ql

codeql query run \
  --database=/tmp/rust-petclinic-codeql-db \
  --output=/tmp/axum-unprotected-mutations.bqrs \
  .github/codeql/axum-route-inventory/axum-unprotected-mutations.ql

codeql bqrs decode \
  --format=csv \
  --output=/tmp/axum-routes.csv \
  /tmp/axum-routes.bqrs
```

The CodeQL workflow runs both the standard `security-extended` suite and the
queries in this pack, so the route-authentication check appears in code
scanning results.
