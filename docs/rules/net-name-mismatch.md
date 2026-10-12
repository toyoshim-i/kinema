---
id: net-name-mismatch
title: "Rule: net-name-mismatch"
summary: Net connectivity partition matches, but net name differs between circuit and PCB.
category: rule
---

# Rule: `net-name-mismatch`

## Problem / Symptom
During equivalence checking, the connected pins on a net match between circuit and PCB, but the net names do not match.

## Root Cause
The net was renamed in Verilog without updating the netlist in KiCad, or renamed manually in KiCad.

## Remediation

1. Export the updated netlist from kinema:
   ```bash
   kinema netlist -o board.net
   ```
2. In KiCad PCB Editor, re-import the netlist to synchronize net names.

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
