---
id: syntax
title: Language Syntax, Grammar & Essential Attributes
aliases: [grammar, attributes, language]
summary: The strict structural Verilog subset and essential attributes for components and nets.
category: guide
---

# Language Syntax, Grammar & Essential Attributes

Kinema uses a strict structural subset of Verilog tailored specifically for printed circuit boards:
- Only `module`, `endmodule`, `inout`, `wire`, and `parameter`.
- All ports must be `inout`. Directions are defined by the `(* etype = "..." *)` attribute.
- Connections must be named explicitly (`.PORT(net)`). Positional connections and `.*` are disallowed.
- Deliberate unconnections must be explicit: `.PORT()`.
- Every net must be declared with `wire`.
- One declaration per line, formatted via `kinema fmt`.

## Essential Attributes

| Attribute | Target | Description | Example |
| :--- | :--- | :--- | :--- |
| `footprint` | Leaf, instance | KiCad footprint identifier | `(* footprint = "Resistor_SMD:R_0603_1608Metric" *)` |
| `mpn` | Leaf, instance | Manufacturer part number | `(* mpn = "NE555DR" *)` |
| `prefix` | Leaf module | Reference prefix | `(* prefix = "U" *)` |
| `pad` | Leaf port | Physical pad number(s) | `(* pad = "1" *)` |
| `etype` | Port | Electrical pin type | `(* etype = "power_in" *)` |
| `decouple` | Leaf port | Requires decoupling capacitor | `(* decouple = "required" *)` |
| `id` | Instance | Stable identity key | `(* id = "u1_core" *)` |
| `ref` | Instance | Fixed refdes override | `(* ref = "U1" *)` |
| `dnp` | Instance | Do Not Populate flag | `(* dnp *)` |
| `property` | Leaf, instance | Custom footprint property | `(* property = "LCSC PN=C546649" *)` |
| `nearby` | Hub wire | Proximity hint | `(* nearby *) wire vcc_u1;` |
| `width` | Wire, port | Min track width constraint | `(* width = "0.5mm" *)` |
| `current` | Wire, port | Current capacity hint | `(* current = "1.5A" *)` |
| `netclass` | Wire, port | KiCad netclass name | `(* netclass = "Power" *)` |
| `diffpair` | Wire | Differential pair pair | `(* diffpair = "USB_D" *)` |

## Code Examples

### Leaf Module
```verilog
(* footprint = "LED_SMD:LED_0603_1608Metric", prefix = "D", property = "LCSC PN=C546649" *)
module LED_RED (
    (* pad = "1", etype = "passive" *) inout A,
    (* pad = "2", etype = "passive" *) inout K
);
endmodule
```

### Circuit Instances with DNP and Overrides
```verilog
(* property = "LCSC PN=C546649" *)
LED_RED D1 (.A(PWR_IND), .K(GND));

(* dnp *)
R #(.value("10k")) R_TEST (.A(VCC), .B(TP1));
```

See also:
- [Connection Model & Decoupling](../guides/connection.md)
- [Component Identity & Refactoring](../guides/identity.md)
- [Design Workflow](../guides/workflow.md)
