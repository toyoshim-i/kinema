---
id: leaf-incomplete
title: "Rule: leaf-incomplete"
summary: Leaf module definition is missing required attributes or valid pad mapping.
category: rule
---

# Rule: `leaf-incomplete`

## Problem / Symptom
Static lint detected a leaf module missing mandatory `footprint` or `prefix` attributes, or containing ports without `pad` attributes.

## Root Cause
Every primitive leaf module representing a physical hardware component must specify `footprint` and `prefix` attributes, and every port must declare its physical pad number.

## Remediation

1. Add `footprint` and `prefix` attributes to the module header:
   ```verilog
   (* footprint = "Resistor_SMD:R_0603_1608Metric", prefix = "R" *)
   module Resistor (
       (* pad = "1", etype = "passive" *) inout A,
       (* pad = "2", etype = "passive" *) inout B
   );
   endmodule
   ```
2. Use `kinema gen-leaf <NAME>` to scaffold a canonical leaf module template.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
