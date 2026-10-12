---
id: pad-count
title: "Rule: pad-count"
summary: Leaf module declared pad count does not match footprint pad count.
category: rule
---

# Rule: `pad-count`

## Problem / Symptom
Static lint detected a mismatch between the count of distinct physical pads declared in a leaf module and the pad count of the resolved KiCad footprint:
```text
Leaf pad count (<actual>) does not match footprint pad count (<expected>)
```

## Root Cause
Kinema counts the total number of distinct physical pad numbers declared across all ports of a leaf module via `(* pad = "..." *)` attributes. If this count differs from the total number of pads defined in the referenced `.kicad_mod` footprint file, kinema reports a pad count mismatch.

Common causes:
1. An incorrect footprint was assigned (e.g. assigning a 5-pin SOT-23-5 footprint to a 3-pin transistor leaf).
2. The leaf module omitted physical pins present on the package (such as thermal exposed pads, ground tabs, or shield pins).

## Remediation

1. Verify that the assigned footprint matches the component package:
   ```verilog
   // SOT-23 (3 pads)
   (* footprint = "Package_TO_SOT_SMD:SOT-23", prefix = "Q" *)
   module NPN_BJT (
       (* pad = "1", etype = "input" *)   inout B,
       (* pad = "2", etype = "passive" *) inout E,
       (* pad = "3", etype = "output" *)  inout C
   );
   endmodule
   ```
2. If the package contains extra physical pins (such as thermal pad `9` on an SOIC-8-1EP), declare the additional pad in the leaf port list so all physical pads are accounted for.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
- [Rule: leaf-incomplete](leaf-incomplete.md)
