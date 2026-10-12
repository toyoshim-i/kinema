---
id: hub-direct-pad
title: "Rule: hub-direct-pad"
summary: Component pad is directly connected to a nearby hub wire.
category: rule
---

# Rule: `hub-direct-pad`

## Problem / Symptom
Static lint detected a component instance port directly wired to a nearby hub wire rather than through an intermediate pin-wire and join node:
```text
Pad of <instance>.<port> is directly connected to nearby hub '<hub>'
```

## Root Cause
In kinema connection architecture, a wire declared with `(* nearby *)` represents a local star-topology hub. Component pads cannot connect directly to a nearby hub wire. Instead, every pad that participates in the hub must connect to its own dedicated pin-wire (named `<instance>_<port>`), which is then attached to the hub using a `join` primitive (`j_<instance>_<port>`).

Connecting an instance port directly to a nearby hub wire (e.g. `u1 (.VCC(vcc_hub))`) bypasses this topology and triggers `hub-direct-pad`.

## Remediation

1. Introduce a dedicated pin-wire for the component port.
2. Join the pin-wire to the nearby hub wire using a `join` instance:
   ```verilog-fragment
   // NG: Direct connection to nearby hub
   // (* nearby *) wire vcc_hub;
   // MCU u1 (.VCC(vcc_hub));

   // OK: Connect via pin-wire and join
   (* nearby *)
   wire vcc_hub;
   wire u1_VCC;

   join j_vcc_hub (.P(VCC), .C(vcc_hub));
   join j_u1_VCC  (.P(vcc_hub), .C(u1_VCC));

   MCU u1 (
       .VCC(u1_VCC)
   );
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
- [Rule: pin-wire-name](pin-wire-name.md)
- [Rule: pin-wire-shape](pin-wire-shape.md)
