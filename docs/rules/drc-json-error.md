---
id: drc-json-error
title: "Rule: drc-json-error"
summary: Failed to parse JSON report produced by kicad-cli DRC.
category: rule
---

# Rule: `drc-json-error`

## Problem / Symptom
The DRC report output by `kicad-cli pcb drc --format json` was malformed or unexpected.

## Root Cause
Incompatible KiCad version or corrupted DRC output format.

## Remediation

1. Ensure a supported version of KiCad (version 8 or newer) is installed.
2. Run `kicad-cli pcb drc --help` to verify CLI capabilities.

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
