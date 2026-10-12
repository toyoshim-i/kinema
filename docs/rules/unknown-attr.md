---
id: unknown-attr
title: "Rule: unknown-attr"
summary: Unknown or misplaced attribute key.
category: rule
---

# Rule: `unknown-attr`

## Problem / Symptom
Static lint found an attribute key that is not recognized by kinema on this syntactic element.

## Root Cause
Typo in attribute name or using an attribute on an unsupported construct (e.g. `footprint` on a wire).

## Remediation

1. Check the spelling of the attribute key.
2. Review valid attributes in the [Circuit Description Syntax Guide](../guides/syntax.md).

See also:
- [Circuit Description Syntax](../guides/syntax.md)
