---
id: width-mismatch
title: "Rule: width-mismatch"
summary: Bit width mismatch between port and connection, or bit index out of bounds.
category: rule
---

# Rule: `width-mismatch`

## Problem / Symptom
Static lint detected a bit width mismatch or an out-of-bounds bit index/slice:
```text
Bit width mismatch for port '<instance>.<port>' (port width <W1>, connection width <W2>)
Bit index [<index>] out of bounds for net '<net>' (declared [<msb>:<lsb>])
Bit slice [<msb>:<lsb>] out of bounds for net '<net>' (declared [<decl_msb>:<decl_lsb>])
```

## Root Cause
Kinema validates multi-bit port and wire widths strictly:
1. **Width Mismatch**: The declared bit width of a module port does not match the total bit width of the connected net, bit slice, or concatenation (e.g. connecting an 8-bit port to a 1-bit wire or a 4-bit bus).
2. **Index Out of Bounds**: An indexed reference (`net[i]`) or bit slice (`net[msb:lsb]`) references indices outside the declared `[msb:lsb]` range of the wire or port.

## Remediation

1. Match wire bus width to the connected port width:
   ```verilog-fragment
   wire [7:0] data_bus;
   MCU u1 (
       .DATA(data_bus) // DATA is declared as inout [7:0]
   );
   ```
2. When concatenating multiple signals, ensure the sum of widths matches the port width:
   ```verilog-fragment
   wire [3:0] high_nibble;
   wire [3:0] low_nibble;
   MCU u1 (
       .DATA({high_nibble, low_nibble})
   );
   ```
3. Ensure bit indices fall within the declared `[msb:lsb]` bounds:
   ```verilog-fragment
   wire [7:0] bus;
   // bus[8] is invalid; highest valid index is 7
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
