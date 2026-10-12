---
id: pad-duplicate
title: "Rule: pad-duplicate"
summary: Duplicate pad number declared within the same leaf module.
category: rule
---

# Rule: `pad-duplicate`

## Problem / Symptom
Static lint detected multiple ports assigned the same physical pad number within a leaf module definition.

## Root Cause
Two or more ports share identical `(* pad = "..." *)` attribute values.

## Remediation

1. Ensure each port in the leaf module has a unique physical pad number matching the footprint pad designation.
2. If multiple internal pins connect to the same external pad, declare a single port.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
