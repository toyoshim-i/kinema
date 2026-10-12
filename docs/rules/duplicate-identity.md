---
id: duplicate-identity
title: "Rule: duplicate-identity"
summary: Multiple component instances share the same ID or reference designator.
category: rule
---

# Rule: `duplicate-identity`

## Problem / Symptom
Static lint detected multiple component instances with conflicting identity keys or reference designators:
```text
Duplicate component ID '<id>'
Duplicate reference designator '<ref>'
```

## Root Cause
Kinema enforces uniqueness on component identity through three checks:
1. **Explicit ID**: Two or more instances declare identical `(* id = "..." *)` attributes.
2. **Explicit Reference**: Two or more instances declare identical `(* ref = "..." *)` attributes.
3. **Implicit Reference**: An instance name matches its leaf prefix followed by numbers (such as `R1` or `C2`) without an explicit `ref`, and collides with another component sharing that reference designator.

## Remediation

1. For duplicate `id` attributes, assign unique identifiers:
   ```verilog-fragment
   (* id = "u1_primary" *) MCU u1 (...);
   (* id = "u2_secondary" *) MCU u2 (...);
   ```
2. For duplicate `ref` attributes or instance names, assign unique reference designators:
   ```verilog-fragment
   (* ref = "R1" *) Resistor r1 (...);
   (* ref = "R2" *) Resistor r2 (...);
   ```

See also:
- [Component Identity & Refactoring](../guides/identity.md)
