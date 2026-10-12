---
id: power-conflict
title: "Rule: power-conflict"
summary: Multiple power output pins connected together without isolation.
category: rule
---

# Rule: `power-conflict`

## Problem / Symptom
Static lint detected multiple pins with electrical type `power_out` driving the same net.

## Root Cause
Connecting multiple voltage regulators or power supplies directly together can cause damaging contention currents.

## Remediation

1. Ensure each power rail is driven by exactly one `power_out` source.
2. Use diodes, OR-ing controllers, or power multiplexers when multiple supplies share a rail.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
