---
id: width-mismatch
title: "Rule: width-mismatch"
summary: Bit width mismatch between connected port and wire.
category: rule
---

# Rule: `width-mismatch`

## Problem / Symptom
Static lint detected a width mismatch when connecting a wire bus to a module port.

## Root Cause
A multi-bit port was connected to a wire with a different bit width, or a single-bit signal was connected to a bus.

## Remediation

1. Ensure wire bit widths match port bit widths:
   ```verilog-fragment
   wire [7:0] data_bus;
   MySubmodule u1 (.DATA(data_bus)); // DATA declared as [7:0]
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
