---
id: hub-too-few
title: "Rule: hub-too-few"
summary: Nearby hub wire has fewer than two child pin-wires attached.
category: rule
---

# Rule: `hub-too-few`

## Problem / Symptom
Static lint detected a wire declared with `(* nearby *)` that has fewer than two child pin-wires joined to it via `join` instances:
```text
Nearby hub '<hub>' has fewer than 2 pin-wires attached (currently <count>)
```

## Root Cause
A wire marked with `(* nearby *)` designates a local star-topology hub (such as a local decoupling point or power distribution node). A hub exists to group multiple child pin-wires together before connecting to a parent net. If fewer than 2 pin-wires are attached as children (`.P(<hub>), .C(<pin_wire>)`), the hub serves no grouping purpose.

Note: In kinema, hubs are designated exclusively by the `(* nearby *)` attribute on wire declarations. There is no `(* hub *)` attribute.

## Remediation

1. Attach at least two child pin-wires to the nearby hub via `join` instances:
   ```verilog-fragment
   (* nearby *)
   wire vcc_u1;
   wire u1_VCC;
   wire c1_A;

   join j_vcc_u1 (.P(VCC), .C(vcc_u1));
   join j_u1_VCC (.P(vcc_u1), .C(u1_VCC));
   join j_c1_A   (.P(vcc_u1), .C(c1_A));
   ```
2. If the wire is an ordinary point-to-point or distribution net rather than a local star-topology hub, remove the `(* nearby *)` attribute from the wire declaration:
   ```verilog-fragment
   wire vcc_u1;
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
