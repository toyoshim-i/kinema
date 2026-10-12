---
id: elab-error
title: "Rule: elab-error"
summary: Elaboration failed during module hierarchy instantiation or parameter resolution.
category: rule
---

# Rule: `elab-error`

## Problem / Symptom
Kinema failed to elaborate the Verilog AST into a flattened netlist IR.

## Root Cause
Unresolved module references, invalid parameter expressions, or hierarchy cycle prevented elaboration.

## Remediation

1. Check the elaboration error message for the specific submodule or parameter causing the failure.
2. Ensure all instantiated submodules are defined and included in `kinema.toml` source or library paths.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
