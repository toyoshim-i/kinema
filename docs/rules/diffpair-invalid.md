---
id: diffpair-invalid
title: "Rule: diffpair-invalid"
summary: Wires with diffpair attribute do not form a valid complementary 2-wire pair.
category: rule
---

# Rule: `diffpair-invalid`

## Problem / Symptom
Static lint detected wires declared with `(* diffpair = "<name>" *)` that do not form a valid 2-wire differential pair:
```text
Differential pair '<name>' does not form a valid 2-wire pair
```

## Root Cause
Kinema groups wires sharing the same `(* diffpair = "<name>" *)` attribute value. A valid differential pair must satisfy two conditions:
1. Exactly 2 wires must share the same `diffpair` value.
2. The two wire names must end with complementary polarity suffixes:
   - `_P` and `_N` (e.g. `usb_dp` and `usb_dn` is invalid; must be `usb_d_P` and `usb_d_N`), OR
   - `+` and `-` (e.g. `clk+` and `clk-`).

If there is only 1 wire, more than 2 wires, or if the name suffixes do not match `_P`/`_N` or `+`/`-`, this diagnostic is emitted.

## Remediation

1. Ensure exactly two wires share the `diffpair` attribute and use matching polarity suffixes:
   ```verilog-fragment
   (* diffpair = "USB_D" *) wire usb_d_P;
   (* diffpair = "USB_D" *) wire usb_d_N;
   ```
2. Or use `+` and `-` suffixes:
   ```verilog-fragment
   (* diffpair = "LVDS_CLK" *) wire clk+;
   (* diffpair = "LVDS_CLK" *) wire clk-;
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
