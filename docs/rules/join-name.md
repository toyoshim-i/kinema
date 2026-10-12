---
id: join-name
title: "Rule: join-name"
summary: Join instance name does not follow canonical naming convention.
category: rule
---

# Rule: `join-name`

## Problem / Symptom
Static lint found a `join` primitive instance whose name violates naming rules.

## Root Cause
Join instances should use descriptive, canonical names such as `j_<net>_<pin>` to maintain traceability.

## Remediation

1. Name join instances systematically based on the connected parent net and child branch:
   ```verilog-fragment
   join j_vcc_u1 (.P(VCC), .C(U1_VCC));
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
