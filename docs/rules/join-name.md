---
id: join-name
title: "Rule: join-name"
summary: Join instance name does not match j_<child_wire> canonical convention.
category: rule
---

# Rule: `join-name`

## Problem / Symptom
Static lint detected a `join` primitive instance whose instance name does not follow the required `j_<child_wire>` convention:
```text
Join instance name must be 'j_<child_wire>'
```

## Root Cause
Kinema requires deterministic, canonical names for all `join` instances. The instance name of a `join` must be exactly `j_` followed by the name of the child wire connected to its `C` port.

For example:
- If a join connects `.P(VCC), .C(vcc_hub)`, the instance name must be `j_vcc_hub`.
- If a join connects `.P(vcc_hub), .C(u1_VCC)`, the instance name must be `j_u1_VCC`.

Any other instance name (such as arbitrary names like `j1`, `join_u1`, or names based on the parent net) violates this rule.

## Remediation

1. Rename the `join` instance to match `j_<child_wire>`:
   ```verilog-fragment
   // NG: Instance name does not match child wire u1_VCC
   // join j_power (.P(vcc_hub), .C(u1_VCC));

   // OK: Instance name matches j_<C_wire>
   join j_u1_VCC (.P(vcc_hub), .C(u1_VCC));
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
