---
id: io-error
title: "Rule: io-error"
summary: I/O error reading input file or accessing file system.
category: rule
---

# Rule: `io-error`

## Problem / Symptom
Kinema encountered an error when trying to open or read a source file or board file.

## Root Cause
The file does not exist, the path is incorrect, or read permissions are denied.

## Remediation

1. Check that the file path is correct and accessible.
2. Verify file permissions.

See also:
- [Design Workflow & Toolchain](../guides/workflow.md)
