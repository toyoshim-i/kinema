---
id: constraint-conflict
title: "Rule: constraint-conflict"
summary: Conflicting width, current, or netclass constraints applied to the same net.
category: rule
---

# Rule: `constraint-conflict`

## Problem / Symptom
Static lint detected conflicting constraint attributes on nets that resolve to the same canonical electrical connection:
```text
Conflicting <kind> constraints specified for net '<net>'
```

## Root Cause
Kinema validates routing and electrical constraints across connected subnets, pin-wires, and `join` hierarchies. When different values for any of the three supported constraint attributes:
1. `width` (track width, e.g. `"0.5mm"` vs `"0.25mm"`)
2. `current` (current carrying capacity, e.g. `"1.5A"` vs `"500mA"`)
3. `netclass` (KiCad netclass name, e.g. `"Power"` vs `"Default"`)

are attached to wires or ports that are electrically joined together into the same canonical net, kinema reports a constraint conflict.

## Remediation

1. Review attributes on the connected nets or ports and unify them to use identical constraint values:
   ```verilog-fragment
   // NG: Conflicting track width constraints on connected wires
   // (* width = "0.5mm" *) wire pwr_a;
   // (* width = "0.25mm" *) wire pwr_b;
   // join j_pwr_b (.P(pwr_a), .C(pwr_b));

   // OK: Consistent constraint values
   (* width = "0.5mm" *) wire pwr_a;
   (* width = "0.5mm" *) wire pwr_b;
   join j_pwr_b (.P(pwr_a), .C(pwr_b));
   ```
2. Or declare the constraint attribute only on the parent distribution net or hub, allowing connected pin-wires to inherit it without conflict.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
- [Circuit Description Syntax](../guides/syntax.md)
