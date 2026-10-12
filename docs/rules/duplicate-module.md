---
id: duplicate-module
title: "Rule: duplicate-module"
summary: Module defined multiple times across loaded source files.
category: rule
---

# Rule: `duplicate-module`

## Problem / Symptom
Static lint detected two or more module definitions sharing the exact same module name.

## Root Cause
A module with the same name was defined in multiple source files or defined more than once in the same file.

## Remediation

1. Rename one of the conflicting modules to a unique name.
2. Ensure library files and project source files do not redundantly define the same module.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
