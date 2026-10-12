---
id: undeclared-net
title: "Rule: undeclared-net"
summary: Wire or net used without declaration.
category: rule
---

# Rule: `undeclared-net`

## Problem / Symptom
Static lint found a signal identifier used in a module port connection that was not declared as a `wire` or port.

## Root Cause
A wire name was misspelled or the `wire <name>;` declaration was omitted.

## Remediation

1. Declare the wire explicitly before using it in port connections:
   ```verilog-fragment
   wire my_signal;
   ```
2. Check for typos in wire names.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
