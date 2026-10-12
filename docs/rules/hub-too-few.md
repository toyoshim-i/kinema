---
id: hub-too-few
title: "Rule: hub-too-few"
summary: Connection hub has fewer than two child branches.
category: rule
---

# Rule: `hub-too-few`

## Problem / Symptom
Static lint detected a hub wire or join node that does not connect multiple child signals.

## Root Cause
A hub wire was declared with `(* nearby *)` or `(* hub *)` but has only one connection, serving no grouping or distribution purpose.

## Remediation

1. Remove redundant single-connection hubs and connect directly.
2. Or ensure all intended branch signals are joined to the hub.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
