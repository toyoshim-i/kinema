---
id: unit-invalid
title: "Rule: unit-invalid"
summary: Invalid or missing unit on width or current attribute.
category: rule
---

# Rule: `unit-invalid`

## Problem / Symptom
Static lint detected an attribute value with an invalid or missing physical unit:
```text
Length '<value>' is missing unit or has invalid unit
Current '<value>' is missing unit or has invalid unit
```

## Root Cause
Kinema validates units strictly on two specific physical constraint attributes:
1. `width` (track width on wires or ports): Must be a valid floating-point number ending with `mm` or `mil` (e.g. `"0.5mm"`, `"20mil"`).
2. `current` (current carrying capacity on wires or ports): Must be a valid floating-point number ending with `mA` or `A` (e.g. `"500mA"`, `"1.5A"`).

Values without units (e.g. `"0.5"`) or with unsupported unit strings trigger this diagnostic. Kinema does not check units on component parameters or other attributes with this rule.

## Remediation

1. For `width` attributes on wires or ports, specify either `mm` or `mil`:
   ```verilog-fragment
   (* width = "0.5mm" *) wire vcc_main;
   (* width = "15mil" *) wire sig_trace;
   ```
2. For `current` attributes on wires or ports, specify either `A` or `mA`:
   ```verilog-fragment
   (* current = "1.5A" *) wire pwr_rail;
   (* current = "100mA" *) wire led_drive;
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
