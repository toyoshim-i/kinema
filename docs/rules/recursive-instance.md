---
id: recursive-instance
title: "Rule: recursive-instance"
summary: Recursive module instantiation detected (cycle in module hierarchy).
category: rule
---

# Rule: `recursive-instance`

## Problem / Symptom
Static lint found a module that instantiates itself directly or indirectly.

## Root Cause
Hardware circuits cannot be recursively instantiated without infinite hardware expansion.

## Remediation

1. Break the recursion by refactoring the repeated structure into parameterized iteration or separate submodules.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
