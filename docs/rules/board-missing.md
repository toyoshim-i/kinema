---
id: board-missing
title: "Rule: board-missing"
summary: Required KiCad board file (.kicad_pcb) was not specified or not found.
category: rule
---

# Rule: `board-missing`

## Problem / Symptom
During equivalence checking or DRC execution, the specified KiCad PCB layout file (.kicad_pcb) could not be located.

## Root Cause
1. The board path specified on the command line or in `kinema.toml` under `project.board` points to a nonexistent file.
2. No board file was provided when running `kinema check` in equivalence or DRC stage.

## Remediation

1. Verify that your PCB file exists and the path in `kinema.toml` or the CLI argument is correct:
   ```toml
   [project]
   board = "board.kicad_pcb"
   ```
2. Provide the board file explicitly via CLI:
   ```bash
   kinema check board.kicad_pcb
   ```

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
