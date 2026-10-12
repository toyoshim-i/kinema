---
id: power-unconnected
title: "Rule: power-unconnected"
summary: Power input pin is unconnected.
category: rule
---

# Rule: `power-unconnected`

## Problem / Symptom
Static lint detected an IC port marked with `etype = "power_in"` that has no connection.

## Root Cause
An IC power supply pin (VCC, VDD, GND, VSS) was left unconnected in the module instantiation.

## Remediation

1. Connect all power input pins to appropriate power and ground rails:
   ```verilog-fragment
   join j_u1_vcc (.P(VCC), .C(U1_VCC));
   join j_u1_gnd (.P(GND), .C(U1_GND));
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
