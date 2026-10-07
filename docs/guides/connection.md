---
id: connection
title: Connection Model, Nearby Pattern, and Decoupling
aliases: [nearby, join, decouple, bypass]
summary: How to model sensitive routing topology such as bypass capacitors and differential pairs using nearby and join.
category: guide
---

# Connection Model: `nearby` & `join`

In KiCad and physical PCB design, all pads on a power rail (e.g. `VCC` or `GND`) belong to the same logical net. However, in physical layout, high-frequency bypass capacitors must be placed immediately adjacent to specific IC power pins before the rail connects to the rest of the plane.

Kinema models this physical routing proximity using the `(* nearby *)` attribute and structural `join` primitives.

See [Rule: decouple-missing](../rules/decouple-missing.md) when an IC requires decoupling caps.

## Structural Topology

1. Declare a hub wire with `(* nearby *)`.
2. Connect each participating component pin to a dedicated pin-wire named `<instance>_<port>`.
3. Connect each pin-wire to the hub using `join j_<pin-wire> (.P(hub), .C(pin-wire));`.
4. Connect the hub to the main power net using `join j_<hub> (.P(main_net), .C(hub));`.

## Code Example

```verilog
// Bypass capacitor C1 placed immediately adjacent to U1 VCC pin
(* nearby *)
wire vcc_u1;
wire U1_VCC;
wire C1_A;

join j_vcc_u1 (.P(VCC), .C(vcc_u1));
join j_U1_VCC (.P(vcc_u1), .C(U1_VCC));
join j_C1_A   (.P(vcc_u1), .C(C1_A));

NE555 U1 (
    .VCC(U1_VCC),
    .GND(GND),
    ...
);

C #(.value("100n")) C1 (
    .A(C1_A),
    .B(GND)
);
```

See also:
- [Component Identity & Refactoring](../guides/identity.md)
- [Rule: decouple-missing](../rules/decouple-missing.md)
