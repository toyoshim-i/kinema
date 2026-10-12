---
id: decouple-multi-pad
title: "Rule: decouple-multi-pad"
summary: Multiple pads on the same component reference incompatible decoupling roles.
category: rule
---

# Rule: `decouple-multi-pad`

## Problem / Symptom
Static lint detected multiple decoupling capacitors or pins joined in an invalid decoupling topology.

## Root Cause
A single bypass capacitor was connected across incompatible rails or a decoupling hub connects multiple distinct power domains.

## Remediation

1. Provide dedicated decoupling capacitors per power pin pair.
2. Ensure each decoupling hub connects exactly one IC power pin and one bypass capacitor to the power rail.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
