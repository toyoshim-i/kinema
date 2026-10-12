---
id: pin-wire-name
title: "Rule: pin-wire-name"
summary: Pin-wire name does not follow canonical <instance>_<port> convention.
category: rule
---

# Rule: `pin-wire-name`

## Problem / Symptom
Static lint detected a pin-wire whose name does not match the canonical `<instance>_<port>` naming format of the component pin it connects to:
```text
Pin-wire name '<wire>' does not match connected pin '<instance>.<port>'
```

## Root Cause
When a wire is joined as a child to a nearby hub via a `join` primitive, kinema requires its name to be strictly `<instance>_<port>` (matching the component instance name and port name). This strict naming ensures physical layout traceability between KiCad pads, netlist traces, and Verilog descriptions.

For example, connecting port `VCC` of instance `u1` must use a wire named `u1_VCC`. Using any other name (such as `vcc_u1`, `u1_pwr`, or `U1_VCC` when instance is `u1`) triggers this rule.

## Remediation

1. Rename the wire and its corresponding `join` instance to match `<instance>_<port>`:
   ```verilog-fragment
   // NG: Pin-wire name does not match instance 'u1' and port 'VCC'
   // wire vcc_u1;
   // join j_vcc_u1 (.P(vcc_hub), .C(vcc_u1));
   // NE555 u1 (.VCC(vcc_u1), ...);

   // OK: Canonical <instance>_<port> naming
   wire u1_VCC;
   join j_u1_VCC (.P(vcc_hub), .C(u1_VCC));
   NE555 u1 (
       .VCC(u1_VCC),
       ...
   );
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
- [Rule: join-name](join-name.md)
- [Rule: pin-wire-shape](pin-wire-shape.md)
