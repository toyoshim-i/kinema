---
id: power-unconnected
title: "Rule: power-unconnected"
summary: Port with etype = "power_in" is left unconnected in instance.
category: rule
---

# Rule: `power-unconnected`

## Problem / Symptom
Static lint detected a component instance where an electrical power input pin was left unconnected:
```text
power_in pin '<instance>.<port>' is unconnected
```

## Root Cause
A component port declared with `(* etype = "power_in" *)` in its leaf module (such as `VCC`, `VDD`, `GND`, or `VSS`) was instantiated with empty parentheses `.PORT()`, leaving the device unpowered or ungrounded.

## Remediation

1. Connect the power input port to an appropriate power rail, ground rail, or decoupling pin-wire:
   ```verilog-fragment
   // NG: power_in pin left unconnected
   // NE555 u1 (
   //     .VCC(),
   //     .GND(GND),
   //     ...
   // );

   // OK: Connected to power pin-wire or net
   NE555 u1 (
       .VCC(u1_VCC),
       .GND(GND),
       ...
   );
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
- [Connection Model & Decoupling](../guides/connection.md)
