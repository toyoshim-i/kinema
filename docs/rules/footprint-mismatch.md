---
id: footprint-mismatch
title: "Rule: footprint-mismatch"
summary: Component footprint assignment differs between circuit description and PCB layout.
category: rule
---

# Rule: `footprint-mismatch`

## Problem / Symptom
During equivalence checking, a component on the PCB uses a different footprint than specified in the Verilog `(* footprint = "..." *)` attribute.

## Root Cause
The footprint attribute was changed in Verilog without updating the PCB, or the footprint was changed directly in KiCad PCB Editor.

## Remediation

1. If the Verilog footprint is authoritative, re-export netlist and update footprints in KiCad:
   ```bash
   kinema netlist -o board.net
   # In KiCad PCB Editor: Tools -> Update Footprints from Library...
   ```
2. If the PCB footprint is authoritative, update the `(* footprint = "..." *)` attribute in your leaf module definition.

See also:
- [Component Identity & Refactoring](../guides/identity.md)
