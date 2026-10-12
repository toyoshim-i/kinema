---
id: diffpair-invalid
title: "Rule: diffpair-invalid"
summary: Differential pair signals do not follow required naming or attribute conventions.
category: rule
---

# Rule: `diffpair-invalid`

## Problem / Symptom
Static lint found a differential pair specification where net names do not form a valid positive/negative pair or polarity is mismatched.

## Root Cause
Differential pair nets must use matching base names with `_P`/`_N` or `+`/`-` polarity suffixes and equal bit widths.

## Remediation

1. Ensure both positive and negative nets are declared with matching naming conventions (e.g. `DIFF_P` and `DIFF_N`).
2. Verify that both branches are connected to corresponding differential ports on the transceiver or connector.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
