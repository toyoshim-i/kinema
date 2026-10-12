---
id: footprint-missing
title: "Rule: footprint-missing"
summary: Footprint specified in leaf module not found in library table or search directories.
category: rule
---

# Rule: `footprint-missing`

## Problem / Symptom
Static lint failed to locate the `.kicad_mod` footprint file referenced by a component leaf module.

## Root Cause
1. The footprint identifier `Library:FootprintName` is misspelled.
2. The library is not registered in the project or global `fp-lib-table`.
3. The footprint path or directory is not included in `kinema.toml` search paths.

## Remediation

1. Verify the footprint name against your KiCad footprint library:
   ```verilog-fragment
   (* footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm", prefix = "U" *)
   module NE555 (...);
   ```
2. Add the library directory or search path to `kinema.toml`:
   ```toml
   [kicad]
   search_paths = ["path/to/footprints"]
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
