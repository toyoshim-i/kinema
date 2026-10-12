---
id: pin-wire-name
title: "Rule: pin-wire-name"
summary: Pin-wire name does not follow canonical <REF>_<PORT> naming convention.
category: rule
---

# Rule: `pin-wire-name`

## Problem / Symptom
Static lint detected a pin-wire that does not follow the required `<REF>_<PORT>` naming pattern.

## Root Cause
Pin-wires directly connected to component ports must use deterministic names matching the instance reference and port name to preserve structural traceability.

## Remediation

1. Name the pin-wire following the `<REF>_<PORT>` convention (e.g. `U1_VCC` for instance `U1` port `VCC`):
   ```verilog-fragment
   wire U1_VCC;
   U1 (.VCC(U1_VCC));
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
