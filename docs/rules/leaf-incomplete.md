---
id: leaf-incomplete
title: "Rule: leaf-incomplete"
summary: Leaf module is missing prefix attribute or port is missing pad/etype attribute.
category: rule
---

# Rule: `leaf-incomplete`

## Problem / Symptom
Static lint detected a primitive leaf module or leaf port that lacks mandatory hardware attributes:
```text
Leaf module '<module>' does not specify a prefix attribute
Port '<port>' in leaf module does not specify pad or etype attribute
```

## Root Cause
A leaf module (a module with no body items representing a physical component) must declare hardware linkage attributes:
1. **Module Prefix**: The module header must declare `(* prefix = "..." *)` (e.g. `"R"`, `"C"`, `"U"`), which defines the default reference designator prefix.
2. **Port Pad & Electrical Type**: Every port on the leaf module must declare both `pad` (physical pin number or comma-separated list of pin numbers) and `etype` (electrical pin type, such as `passive`, `input`, `output`, `power_in`, etc.).

Note: Missing footprint assignments are validated separately under [Rule: footprint-missing](footprint-missing.md).

## Remediation

1. Add `prefix` to the leaf module header if missing:
   ```verilog
   (* prefix = "R" *)
   module Resistor (
       (* pad = "1", etype = "passive" *) inout A,
       (* pad = "2", etype = "passive" *) inout B
   );
   endmodule
   ```
2. Ensure every port declares both `pad` and `etype` attributes:
   ```verilog
   (* prefix = "U", footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm" *)
   module NE555 (
       (* pad = "1", etype = "power_in" *) inout GND,
       (* pad = "2", etype = "input" *)    inout TRIG,
       (* pad = "3", etype = "output" *)   inout OUT,
       (* pad = "4", etype = "input" *)    inout RESET,
       (* pad = "5", etype = "passive" *)  inout CONT,
       (* pad = "6", etype = "input" *)    inout THRES,
       (* pad = "7", etype = "output" *)   inout DISCH,
       (* pad = "8", etype = "power_in", decouple = "required" *) inout VCC
   );
   endmodule
   ```
3. Use `kinema gen-leaf <NAME>` to scaffold a complete leaf module template.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
- [Rule: footprint-missing](footprint-missing.md)
