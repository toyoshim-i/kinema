---
id: not-formatted
title: "Rule: not-formatted"
summary: Source file is not canonically formatted according to kinema fmt rules.
category: rule
---

# Rule: `not-formatted`

## Problem / Symptom
Running `kinema check --stage fmt` or `kinema fmt --check` found formatting deviations.

## Root Cause
Indentation, spacing, or attribute layout differs from canonical Kinema formatting standards.

## Remediation

1. Run `kinema fmt` to automatically format files in-place:
   ```bash
   kinema fmt <file.v>
   ```
2. Use `kinema fmt --check` in CI to ensure all files conform to canonical style.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
