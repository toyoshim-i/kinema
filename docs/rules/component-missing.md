---
id: component-missing
title: "Rule: component-missing"
summary: Component defined in circuit description does not exist on the PCB layout.
category: rule
---

# Rule: `component-missing`

## Problem / Symptom
During equivalence checking, a component defined in the Verilog circuit description could not be found on the PCB layout.

## Root Cause
1. A new component was instantiated in Verilog, but the updated netlist has not yet been imported into KiCad PCB Editor.
2. The component was unplaced or deleted from the PCB layout.

## Remediation

1. Export the netlist from kinema:
   ```bash
   kinema netlist -o board.net
   ```
2. In KiCad PCB Editor, select **File -> Import -> Netlist...** and place the new component footprints on the board.

See also:
- [Component Identity & Refactoring](../guides/identity.md)
