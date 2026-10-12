---
id: join-cycle
title: "Rule: join-cycle"
summary: Cycle detected in join graph hierarchy.
category: rule
---

# Rule: `join-cycle`

## Problem / Symptom
Static lint detected a circular reference among interconnected `join` nodes.

## Root Cause
Join instances formed a closed loop (`P -> C -> ... -> P`), which violates the strict tree hierarchy required for deterministic netlist generation.

## Remediation

1. Inspect the join connections between parent (`.P`) and child (`.C`) ports.
2. Ensure join relationships form a directed acyclic tree from root net to leaf pin-wires.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
