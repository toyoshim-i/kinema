---
id: pin-wire-shape
title: "Rule: pin-wire-shape"
summary: Pin-wire does not have exactly 1 pad connection and 1 join to a hub.
category: rule
---

# Rule: `pin-wire-shape`

## Problem / Symptom
Static lint detected an invalid topology on a pin-wire:
```text
Pin-wire '<wire>' must have exactly 1 pad (currently <pad_count>) and 1 join to hub (currently <join_count>)
```

## Root Cause
In kinema connection architecture, every pin-wire represents an isolated, 1-to-1 connection between exactly one physical component pin and a `join` node attached to a hub.

A pin-wire must satisfy both topological invariants:
1. **Exactly 1 Component Pad**: It cannot connect to multiple pads (no daisy-chaining pin-wires directly across components), nor can it connect to zero pads.
2. **Exactly 1 Join Instance**: It must be attached as child `C` to exactly one parent hub via a `join` primitive. It cannot participate in multiple joins, nor can it omit its join.

## Remediation

1. Ensure the pin-wire is connected to exactly one component port:
   ```verilog-fragment
   wire u1_VCC;
   join j_u1_VCC (.P(vcc_hub), .C(u1_VCC));
   NE555 u1 (.VCC(u1_VCC), ...);
   ```
2. If another component pin connects to the same hub (such as a bypass capacitor), declare a separate pin-wire and join for that pin:
   ```verilog-fragment
   wire c1_A;
   join j_c1_A (.P(vcc_hub), .C(c1_A));
   Capacitor c1 (.A(c1_A), .B(GND));
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
- [Rule: pin-wire-name](pin-wire-name.md)
- [Rule: hub-direct-pad](hub-direct-pad.md)
