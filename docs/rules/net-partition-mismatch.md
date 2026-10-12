---
id: net-partition-mismatch
title: "Rule: net-partition-mismatch"
summary: Net connectivity or partitioning does not match between circuit description and PCB layout.
category: rule
---

# Rule: `net-partition-mismatch`

## Problem / Symptom
During equivalence checking, the group of component pins connected together on the PCB does not match the netlist defined in Verilog.

## Root Cause
1. Traces on the PCB connect pins to the wrong net, or missing traces leave pins unrouted.
2. Short circuits between different nets exist on the PCB layout.

## Remediation

1. Run `kinema check` to inspect the exact pin differences reported for the mismatched nets.
2. Update the PCB routing in KiCad PCB Editor to match the circuit description, or fix the net connection in Verilog.

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
