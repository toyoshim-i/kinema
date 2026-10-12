---
id: top-module
title: "Rule: top-module"
summary: No top module or multiple top modules found in project.
category: rule
---

# Rule: `top-module`

## Problem / Symptom
Static lint requires exactly one top-level module (a module that is not instantiated by any other module and has no footprint).

## Root Cause
1. No module in the project qualified as a top-level module.
2. Multiple candidate top modules exist without a specified top module in `kinema.toml`.

## Remediation

1. Define a top module encapsulating your circuit design.
2. If multiple modules exist, specify the top module in `kinema.toml`:
   ```toml
   [project]
   top = "my_board_top"
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
