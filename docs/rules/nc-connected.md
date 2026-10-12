---
id: nc-connected
title: "Rule: nc-connected"
summary: Port or pin marked as no-connect (nc) is connected to an active net.
category: rule
---

# Rule: `nc-connected`

## Problem / Symptom
Static lint detected a pin marked with `(* nc *)` or `etype = "no_connect"` that is connected to an active signal or power net.

## Root Cause
A pin designated as physically unconnected was wired to a circuit net.

## Remediation

1. Disconnect the pin from the net in Verilog source.
2. If the pin is actually used, remove the `(* nc *)` attribute from its leaf port definition.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
