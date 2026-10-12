---
id: unit-invalid
title: "Rule: unit-invalid"
summary: Invalid or missing physical engineering unit on attribute.
category: rule
---

# Rule: `unit-invalid`

## Problem / Symptom
Static lint detected an attribute value with an invalid or unrecognized physical unit (e.g. current, voltage, resistance).

## Root Cause
Attribute value format does not match required engineering unit strings (e.g. "1.5A", "10k", "100nF").

## Remediation

1. Specify standard engineering units with numerical values (e.g. `"1.5A"`, `"100mA"`, `"10k"`, `"0.1uF"`).

See also:
- [Circuit Description Syntax](../guides/syntax.md)
