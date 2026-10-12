---
id: net-extra
title: "Rule: net-extra"
summary: Extraneous net exists on PCB that is not defined in circuit description.
category: rule
---

# Rule: `net-extra`

## Problem / Symptom
During equivalence checking, an extra net was detected on the PCB layout that has no counterpart in the Verilog netlist IR.

## Root Cause
A net was added manually in KiCad PCB Editor or remained after being deleted from Verilog.

## Remediation

1. Remove the extra net traces from the PCB layout.
2. Or define the missing net and connections in your Verilog source, then re-export the netlist.

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
