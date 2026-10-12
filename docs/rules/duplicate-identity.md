---
id: duplicate-identity
title: "Rule: duplicate-identity"
summary: Multiple component instances share the same identity key or UUID.
category: rule
---

# Rule: `duplicate-identity`

## Problem / Symptom
Static lint detected multiple component instances with conflicting `(* id = "..." *)` attributes or colliding default keys.

## Root Cause
Two or more component instances in the module hierarchy were assigned identical ID attributes.

## Remediation

1. Assign distinct `(* id = "..." *)` attributes to each component instance.
2. Or remove the explicit `id` attribute to let kinema derive unique deterministic UUIDs automatically from instance hierarchy paths.

See also:
- [Component Identity & Refactoring](../guides/identity.md)
