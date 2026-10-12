---
id: pin-wire-shape
title: "Rule: pin-wire-shape"
summary: Pin-wire is connected to multiple component ports or has irregular topology.
category: rule
---

# Rule: `pin-wire-shape`

## Problem / Symptom
Static lint detected a pin-wire that connects to more than one component port or has invalid branching.

## Root Cause
A pin-wire must connect to exactly one component port. Sharing signals across components must be done via interconnect nets and join nodes.

## Remediation

1. Declare separate pin-wires for each component port.
2. Use `join` primitives to connect multiple pin-wires to a shared net.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
