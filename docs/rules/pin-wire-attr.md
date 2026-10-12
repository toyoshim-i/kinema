---
id: pin-wire-attr
title: "Rule: pin-wire-attr"
summary: Pin-wire declaration is missing electrical type or required attribute.
category: rule
---

# Rule: `pin-wire-attr`

## Problem / Symptom
Static lint detected a pin-wire lacking the required `(* etype = "..." *)` attribute or other structural annotations.

## Root Cause
In kinema connection architecture, pin-wires must annotate electrical type matching the connected leaf port.

## Remediation

1. Annotate the pin-wire with electrical type matching the port definition:
   ```verilog-fragment
   (* etype = "input" *)
   wire U1_IN;
   ```
2. Read the [Connection Model & Decoupling Guide](../guides/connection.md).

See also:
- [Connection Model & Decoupling](../guides/connection.md)
