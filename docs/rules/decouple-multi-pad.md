---
id: decouple-multi-pad
title: "Rule: decouple-multi-pad"
summary: Leaf module port with decouple = "required" maps to multiple physical pads.
category: rule
---

# Rule: `decouple-multi-pad`

## Problem / Symptom
Static lint detected a leaf module port marked with `(* decouple = "required" *)` whose `(* pad = "..." *)` attribute contains multiple comma-separated pad numbers:
```text
Port '<port>' with decouple = "required" has multiple pads; split into per-pad ports
```

## Root Cause
In kinema, decoupling constraints apply to specific physical power pins on an IC package so that bypass capacitors can be placed right next to each individual pin. When multiple physical pads are grouped into a single port (e.g. `(* pad = "1, 8", decouple = "required" *) inout VCC`), individual per-pad pin-wires and nearby star hubs cannot be constructed in the circuit description.

## Remediation

1. Split the grouped port into distinct individual ports in the leaf module definition:
   ```verilog
   // NG: Multiple pads on a decoupling-required port
   // (* pad = "1, 8", etype = "power_in", decouple = "required" *) inout VCC

   // OK: Separate per-pad ports
   (* footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm", prefix = "U" *)
   module DualSupplyIC (
       (* pad = "1", etype = "power_in", decouple = "required" *) inout VCC1,
       (* pad = "8", etype = "power_in", decouple = "required" *) inout VCC2,
       (* pad = "4", etype = "power_in" *) inout GND
   );
   endmodule
   ```
2. Each split port can then have its own dedicated pin-wire and local nearby decoupling hub.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
- [Rule: decouple-missing](decouple-missing.md)
