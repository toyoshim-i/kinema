---
id: footprint-missing
title: "Rule: footprint-missing"
summary: Component footprint not specified or not found in footprint libraries.
category: rule
---

# Rule: `footprint-missing`

## Problem / Symptom
Static lint detected a component without an assigned footprint, or a footprint identifier that cannot be resolved:
```text
Footprint not specified for component '<instance>'
Footprint '<footprint>' does not exist in footprint libraries
```

## Root Cause
1. **Unassigned Footprint**: A leaf module was instantiated without specifying a `footprint` attribute on either the leaf module definition or the component instance.
2. **Unresolved Footprint**: The assigned footprint string (e.g. `Package_SO:SOIC-8_3.9x4.9mm_P1.27mm`) cannot be found in the global or project `fp-lib-table`, or within configured search directories.

## Remediation

1. Specify a footprint on the leaf module or component instance:
   ```verilog-fragment
   // On instance:
   (* footprint = "Resistor_SMD:R_0603_1608Metric" *)
   Resistor r1 (.A(net_a), .B(net_b));

   // Or on leaf definition:
   (* footprint = "Resistor_SMD:R_0603_1608Metric", prefix = "R" *)
   module Resistor (
       (* pad = "1", etype = "passive" *) inout A,
       (* pad = "2", etype = "passive" *) inout B
   );
   endmodule
   ```
2. If the footprint exists on disk but is not found, verify the library prefix and add the directory to `kinema.toml`:
   ```toml
   [kicad]
   search_paths = ["path/to/footprint/libraries"]
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
