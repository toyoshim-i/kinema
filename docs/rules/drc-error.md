---
id: drc-error
title: "Rule: drc-error"
summary: KiCad Design Rule Check (DRC) reported layout or clearance violations.
category: rule
---

# Rule: `drc-error`

## Problem / Symptom
KiCad DRC executed via `kicad-cli` detected clearance, unrouted net, or physical design rule violations on the PCB.

## Root Cause
Traces, pads, vias, or courtyards violate clearance or design rules specified in KiCad board settings.

## Remediation

1. Open the board in KiCad PCB Editor and run **Inspect -> Design Rules Checker**.
2. Resolve layout violations (clearance, unrouted airwires, silk overlaps) and save the board.

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
