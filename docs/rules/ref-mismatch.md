---
id: ref-mismatch
title: "Rule: ref-mismatch"
summary: Fixed reference designator does not match PCB footprint reference.
category: rule
---

# Rule: `ref-mismatch`

## Problem / Symptom
During equivalence checking, a component with a fixed reference designator (`(* ref = "..." *)`) does not match the reference designator of the matched footprint on the PCB.

## Root Cause
The reference designator was changed on the PCB layout without updating the explicit `ref` attribute in Verilog.

## Remediation

1. Update the reference designator in Verilog:
   ```verilog-fragment
   (* ref = "U2" *)
   NE555 U2 (...);
   ```
2. Or update the footprint reference in KiCad PCB Editor to match the Verilog description.

See also:
- [Component Identity & Refactoring](../guides/identity.md)
