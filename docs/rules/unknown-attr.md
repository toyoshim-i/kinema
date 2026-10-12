---
id: unknown-attr
title: "Rule: unknown-attr"
summary: Unknown attribute, invalid target, unsupported etype, or malformed property.
category: rule
---

# Rule: `unknown-attr`

## Problem / Symptom
Static lint detected an unrecognized attribute key, an attribute attached to an unsupported syntactic target, an invalid `etype`, or a malformed `property` attribute:
```text
Unknown attribute '<attr>' or attribute attached to invalid target
Invalid etype '<value>'. Must be one of ["input", "output", ...]
Invalid property attribute format. Must be "Name=Value" or "Name:Value"
```

## Root Cause
Kinema strictly validates attributes against supported targets, allowable values, and expected syntax:

1. **Allowed Attributes by Target**:
   - Leaf module headers: `footprint`, `mpn`, `prefix`, `property`.
   - Non-leaf module headers: none.
   - Component instances: `footprint`, `mpn`, `id`, `ref`, `dnp`, `property`.
   - Ports: `etype`, `width`, `current`, `netclass` (and `pad`, `decouple` on leaf ports only).
   - Wires: `nearby`, `diffpair`, `width`, `current`, `netclass`.
2. **Valid `etype` Values**:
   `input`, `output`, `bidirectional`, `tri_state`, `passive`, `power_in`, `power_out`, `open_collector`, `open_emitter`, `no_connect`.
3. **Valid `property` Format**:
   Must follow `"Name=Value"` or `"Name:Value"` format (e.g. `"LCSC PN=C546649"`).

## Remediation

1. Verify that the attribute name is spelled correctly and attached to an allowed target:
   ```verilog-fragment
   (* nearby *) wire vcc_hub; // nearby is only valid on wires
   (* dnp *) Resistor r1 (...); // dnp is only valid on instances
   ```
2. For port electrical types, use one of the 10 supported `etype` values:
   ```verilog-fragment
   (* pad = "1", etype = "power_in" *) inout VCC
   ```
3. For custom footprint properties, format as `"Name=Value"`:
   ```verilog-fragment
   (* property = "LCSC PN=C546649" *)
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
