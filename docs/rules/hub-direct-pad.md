---
id: hub-direct-pad
title: "Rule: hub-direct-pad"
summary: Direct pin-wire connection bypasses required connection hub.
category: rule
---

# Rule: `hub-direct-pad`

## Problem / Symptom
Static lint detected an IC pin-wire connected directly to a bus or global net without passing through a join node or designated hub.

## Root Cause
In kinema connection architecture, pin-wires must connect to dedicated join nodes rather than tapping arbitrary interconnect nets directly.

## Remediation

1. Read the [Connection Model & Decoupling Guide](../guides/connection.md).
2. Declare a join node to attach the pin-wire to the destination net:
   ```verilog-fragment
   join j_u1_in (.P(SIGNAL_NET), .C(U1_IN));
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
