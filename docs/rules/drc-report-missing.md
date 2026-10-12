---
id: drc-report-missing
title: "Rule: drc-report-missing"
summary: kicad-cli finished but did not produce the expected DRC JSON report file.
category: rule
---

# Rule: `drc-report-missing`

## Problem / Symptom
The `kicad-cli` process exited successfully, but the output JSON report file was not found.

## Root Cause
Invalid output path, missing write permissions, or execution was aborted before writing output.

## Remediation

1. Verify write permissions in the working directory.
2. Test running `kicad-cli pcb drc -o /tmp/drc.json <board.kicad_pcb>` directly from terminal.

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
