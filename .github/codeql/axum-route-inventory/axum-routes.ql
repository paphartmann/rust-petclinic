/**
 * @name Axum-style route registrations
 * @description Lists route paths, HTTP methods, and handler functions passed to route calls.
 * @kind table
 * @id local/axum-route-inventory
 */

import codeql.rust.elements
import codeql.rust.elements.Call

from MethodCallExpr route, Call method
where
  route.getIdentifier().getText() = "route" and
  method = route.getPositionalArgument(1)
select
  route,
  route.getPositionalArgument(0).toString(),
  method.getTargetName(),
  method.getPositionalArgument(0).toString()
