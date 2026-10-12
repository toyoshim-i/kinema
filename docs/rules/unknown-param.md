---
id: unknown-param
title: "Rule: unknown-param"
summary: Unknown parameter passed to module instance.
category: rule
---

# Rule: `unknown-param`

## Problem / Symptom
Static lint detected a parameter passed to a module instance that is not declared in the target module.

## Root Cause
Parameter name in `#(.PARAM(val))` is misspelled or does not exist in the module definition.

## Remediation

1. Verify parameter names declared in the target module definition.
2. Correct the parameter name at instantiation.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
