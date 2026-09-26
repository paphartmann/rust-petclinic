# Axum route inventory query

This diagnostic query lists method calls shaped like Axum route registrations:
`.route(path, method(handler))`. It reports the path, HTTP method, and handler
function, but does not model Axum authentication, middleware, or request data
flow.

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

codeql bqrs decode \
  --format=csv \
  --output=/tmp/axum-routes.csv \
  /tmp/axum-routes.bqrs
```
