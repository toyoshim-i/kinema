---
id: pad-count
title: "Rule: pad-count"
summary: Component instance pin count does not match footprint pad count.
category: rule
---

# Rule: `pad-count`

## Problem / Symptom
Static lint detected a mismatch between the number of ports defined on a leaf module and the physical pad count of the assigned KiCad footprint.

## Root Cause
The assigned footprint has more or fewer pads than the module definition (e.g. SOT-23-5 assigned to a 3-pin transistor module).

## Remediation

1. Verify that the correct footprint is assigned in `(* footprint = "..." *)`.
2. Ensure all physical pads (including thermal pads, NC pins, or shield pins) are declared in the leaf module port list.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
