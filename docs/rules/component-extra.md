---
id: component-extra
title: "Rule: component-extra"
summary: Extraneous component footprint found on PCB that does not exist in circuit description.
category: rule
---

# Rule: `component-extra`

## Problem / Symptom
During equivalence checking, a component footprint on the PCB layout has no matching instance in the Verilog netlist IR.

## Root Cause
1. A component was added directly in KiCad PCB Editor instead of being described in Verilog.
2. A component was removed from the Verilog source but its footprint remains on the PCB layout.

## Remediation

1. Add the component definition and instance to your Verilog source files, then re-export netlist:
   ```bash
   kinema netlist -o board.net
   ```
2. Alternatively, if the component was intentionally removed from the design, delete the footprint from the KiCad PCB layout.

See also:
- [Component Identity & Refactoring](../guides/identity.md)
