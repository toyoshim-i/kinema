---
id: pin-wire-attr
title: "Rule: pin-wire-attr"
summary: Pin-wire declaration has attributes attached.
category: rule
---

# Rule: `pin-wire-attr`

## Problem / Symptom
Static lint detected attributes attached to a wire that serves as a pin-wire:
```text
Pin-wire '<wire>' cannot have attributes
```

## Root Cause
In kinema connection architecture, a pin-wire is a dedicated, zero-resistance local connection wire that links an individual component port directly to a `join` primitive (e.g. `wire u1_VCC; join j_u1_VCC (.P(vcc_hub), .C(u1_VCC));`).

Pin-wires cannot have any attributes attached to them. Electrical properties (such as `etype`) are declared on the leaf module ports, and routing constraints (such as `width`, `current`, or `netclass`) belong on the parent hub or distribution nets. Attaching attributes to a pin-wire declaration triggers `pin-wire-attr`.

## Remediation

1. Remove all attributes from the pin-wire declaration:
   ```verilog-fragment
   // NG: Pin-wire has attributes
   // (* width = "0.5mm" *) wire u1_VCC;

   // OK: Clean pin-wire declaration without attributes
   wire u1_VCC;
   ```
2. Move any routing or electrical constraints (such as `width`, `current`, or `netclass`) to the parent hub wire or the main net:
   ```verilog-fragment
   (* nearby, width = "0.5mm" *)
   wire vcc_hub;
   wire u1_VCC;

   join j_vcc_hub (.P(VCC), .C(vcc_hub));
   join j_u1_VCC  (.P(vcc_hub), .C(u1_VCC));
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
- [Rule: pin-wire-name](pin-wire-name.md)
- [Rule: pin-wire-shape](pin-wire-shape.md)
