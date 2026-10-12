---
id: single-pin-net
title: "Rule: single-pin-net"
summary: Net has only a single connection (floating net).
category: rule
---

# Rule: `single-pin-net`

## Problem / Symptom
Static lint detected a declared net that is connected to only one component pin or port.

## Root Cause
A net was partially routed, a typo in a wire name created an isolated net, or a required connection was forgotten.

## Remediation

1. Connect the net to its intended destination pins.
2. If the pin is meant to be unconnected, mark the port with `(* nc *)` instead of connecting to a wire.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
