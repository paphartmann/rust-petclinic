/**
 * @name Axum mutation route without handler authentication
 * @description Finds registered write routes whose handler does not use the Claims extractor.
 * @kind problem
 * @problem.severity warning
 * @security-severity 6.5
 * @precision high
 * @id rust/axum-unprotected-mutation
 * @tags security
 *       external/cwe/cwe-862
 */

import codeql.rust.elements
import codeql.rust.elements.Call

from MethodCallExpr route, Call method, Function handler
where
  route.getIdentifier().getText() = "route" and
  method = route.getPositionalArgument(1) and
  (
    method.getTargetName() = "post" or
    method.getTargetName() = "put" or
    method.getTargetName() = "patch" or
    method.getTargetName() = "delete"
  ) and
  handler.getName() = method.getPositionalArgument(0).toString() and
  not handler.getText().matches("%Claims%") and
  route.getPositionalArgument(0).toString() != "\"/token\""
select route,
  "Write route " + route.getPositionalArgument(0).toString() +
    " uses handler " + handler.getName() + " without the Claims extractor."
