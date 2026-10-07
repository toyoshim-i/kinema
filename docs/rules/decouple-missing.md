---
id: decouple-missing
title: "Rule: decouple-missing"
aliases: [decouple, missing-bypass]
summary: Port with decouple = "required" attribute is not joined to a nearby decoupling hub.
category: rule
---

# Rule: `decouple-missing`

## Problem / Symptom
Static lint detected an IC power pin marked with `decouple = "required"` that is connected directly to a global power rail without routing through a local decoupling hub.

## Root Cause
IC leaf definitions in `lib/*.v` require bypass capacitors near high-current power pins to ensure electrical stability and prevent noise on shared power rails. Connecting the pin directly to `VCC` without a dedicated `(* nearby *)` hub violates this structural constraint.

## Remediation

1. Read the [Connection Model & Decoupling Guide](../guides/connection.md).
2. Declare a local hub wire marked with `(* nearby *)`.
3. Route the IC power pin and capacitor pin through dedicated pin-wires and join nodes:
   ```verilog
   (* nearby *)
   wire vcc_u1;
   wire U1_VCC;
   wire C1_A;

   join j_vcc_u1 (.P(VCC), .C(vcc_u1));
   join j_U1_VCC (.P(vcc_u1), .C(U1_VCC));
   join j_C1_A   (.P(vcc_u1), .C(C1_A));
   ```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
