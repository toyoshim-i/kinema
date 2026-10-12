---
id: syntax-error
title: "Rule: syntax-error"
summary: Verilog syntax error during parsing.
category: rule
---

# Rule: `syntax-error`

## Problem / Symptom
Kinema failed to parse the Verilog circuit description file due to syntax errors.

## Root Cause
Missing semicolons, unmatched parentheses/brackets, or unsupported Verilog constructs.

## Remediation

1. Check the line number and column reported in the diagnostic message.
2. Ensure code conforms to kinema structural Verilog subset rules described in the [Syntax Guide](../guides/syntax.md).

See also:
- [Circuit Description Syntax](../guides/syntax.md)
