---
id: kicad-cli-not-found
title: "Rule: kicad-cli-not-found"
summary: kicad-cli executable was not found on PATH or standard install locations.
category: rule
---

# Rule: `kicad-cli-not-found`

## Problem / Symptom
Kinema attempted to run `kicad-cli` for DRC verification, but the executable could not be launched.

## Root Cause
`kicad-cli` is not installed or its directory is not included in the system PATH environment variable.

## Remediation

1. Install KiCad (version 8 or newer).
2. Add `kicad-cli` to your PATH, or specify the KiCad installation directory:
   - macOS: `/Applications/KiCad/KiCad.app/Contents/MacOS`
   - Linux: `/usr/bin/kicad-cli`
   - Windows: `C:\Program Files\KiCad\bin`

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
