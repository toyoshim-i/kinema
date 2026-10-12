---
id: nc-pad-connected
title: "Rule: nc-pad-connected"
summary: No-connect pad on PCB is routed or connected to a net.
category: rule
---

# Rule: `nc-pad-connected`

## Problem / Symptom
During equivalence checking, a pad designated as no-connect in the circuit description was found connected to a copper net on the PCB.

## Root Cause
Copper traces or planes on the PCB connect to a pad marked as unconnected in the Verilog model.

## Remediation

1. In KiCad PCB Editor, delete traces or copper zones connected to the no-connect pad.
2. If the connection is intentional, update the port definition in the Verilog leaf module to remove the `no_connect` type.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
