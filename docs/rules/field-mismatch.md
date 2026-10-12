---
id: field-mismatch
title: "Rule: field-mismatch"
summary: Component property, value, MPN, or DNP attribute does not match PCB layout.
category: rule
---

# Rule: `field-mismatch`

## Problem / Symptom
During equivalence checking, a component property (such as `value`, `mpn`, `dnp`, or custom field) differs between the Verilog description and the PCB footprint.

## Root Cause
Component properties were edited on the PCB without updating Verilog, or vice versa.

## Remediation

1. Update the component attributes in Verilog (e.g. `(* mpn = "...", value = "..." *)`) to match the intended values.
2. Re-export the netlist and re-import into KiCad to synchronize component fields.

See also:
- [Component Identity & Refactoring](../guides/identity.md)
