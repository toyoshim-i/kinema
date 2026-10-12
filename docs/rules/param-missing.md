---
id: param-missing
title: "Rule: param-missing"
summary: Required module parameter was not provided at instantiation.
category: rule
---

# Rule: `param-missing`

## Problem / Symptom
Static lint found a submodule instance where a mandatory parameter has no value.

## Root Cause
The module definition declares a parameter without a default value, but the instantiation did not provide a value.

## Remediation

1. Supply the parameter value at instantiation:
   ```verilog-fragment
   MyModule #(.PARAM(value)) u1 (...);
   ```
2. Or provide a default parameter value in the module declaration.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
