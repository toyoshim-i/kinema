---
id: nc-connected
title: "Rule: nc-connected"
summary: Port declared with etype = "no_connect" is connected to an active net.
category: rule
---

# Rule: `nc-connected`

## Problem / Symptom
Static lint detected a component port declared with `(* etype = "no_connect" *)` that has a net connected to it in a module instance:
```text
no_connect pin '<instance>.<port>' is connected to a net
```

## Root Cause
When an IC or connector pin is defined in its leaf module with `etype = "no_connect"` (such as internal test pins, factory pins, or package no-connects), kinema enforces that the pin must remain completely unconnected in the circuit. Supplying a net expression (e.g. `.NC(my_net)`) violates this electrical constraint.

## Remediation

1. If the pin should remain unconnected, write the port with empty parentheses `.PORT()`:
   ```verilog-fragment
   // NG: Connected to a net
   // IC u1 (.NC(gnd_net));

   // OK: Explicit deliberate unconnection
   IC u1 (
       .VCC(u1_VCC),
       .GND(GND),
       .NC() // Left unconnected
   );
   ```
2. If the pin is actually meant to be connected to a circuit net, update the port's `etype` in its leaf module definition from `no_connect` to an appropriate electrical type (such as `passive`, `input`, etc.).

See also:
- [Circuit Description Syntax](../guides/syntax.md)
- [Rule: missing-port](missing-port.md)
