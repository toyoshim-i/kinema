---
id: missing-port
title: "Rule: missing-port"
summary: Required port of module instance was not connected in port list.
category: rule
---

# Rule: `missing-port`

## Problem / Symptom
Static lint detected an instantiated submodule where a declared port was omitted from the connection list.

## Root Cause
A module instantiation failed to connect one or more declared ports.

## Remediation

1. Connect all ports explicitly in the module instance port list:
   ```verilog-fragment
   MySubmodule u1 (
       .CLK(clk),
       .DATA(data)
   );
   ```
2. For unused pins, mark the corresponding port with `(* nc *)` or connect to an explicit unconnected wire.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
