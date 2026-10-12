---
id: pcb-parse-error
title: "Rule: pcb-parse-error"
summary: Failed to parse KiCad PCB file (.kicad_pcb) S-expression format.
category: rule
---

# Rule: `pcb-parse-error`

## Problem / Symptom
Kinema failed to parse the KiCad board layout file.

## Root Cause
The `.kicad_pcb` file is corrupted, truncated, or uses an unsupported syntax version.

## Remediation

1. Open the board file in KiCad PCB Editor and re-save it to regenerate clean S-expressions.
2. Verify that the board file is not empty or corrupted.

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
