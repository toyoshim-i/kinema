---
id: undriven-net
title: "Rule: undriven-net"
summary: Net has only input pins and no driving signal source.
category: rule
---

# Rule: `undriven-net`

## Problem / Symptom
Static lint detected a net connected only to input ports (`etype = "input"`), with no output or power driver.

## Root Cause
An input pin was connected to a net that is not driven by any component output or module input port.

## Remediation

1. Connect an output, bidirectional, or power pin to drive the net.
2. If the net is driven by an external connector, declare the driving port with `etype = "input"` or `etype = "passive"`.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
