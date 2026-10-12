---
id: constraint-conflict
title: "Rule: constraint-conflict"
summary: Conflicting routing or electrical constraints applied to the same net.
category: rule
---

# Rule: `constraint-conflict`

## Problem / Symptom
Static lint detected contradictory constraint attributes (e.g. conflicting width or class specifications) on a net or between joined nets.

## Root Cause
Multiple conflicting constraint attributes were attached to interconnected pin-wires or joined subnets.

## Remediation

1. Review attribute annotations on joined nets and ensure routing constraints (such as `width` or netclass) are consistent.
2. Read the [Connection Model & Decoupling Guide](../guides/connection.md) for constraint propagation rules.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
