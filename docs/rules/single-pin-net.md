---
id: single-pin-net
title: "Rule: single-pin-net"
summary: Declared wire has only a single component pad connected.
category: rule
---

# Rule: `single-pin-net`

## Problem / Symptom
Static lint detected a declared wire connected to only one component pad:
```text
Net '<net>' has only 1 component pad connected
```

## Root Cause
An internal wire connects to a single component pin and is neither a top-level port nor connected to any other pins or `join` nodes. This usually indicates an unfinished circuit connection, a typographical error in a net name, or a floating test point.

## Remediation

1. Connect the other end of the net to its intended destination port or pin.
2. If the component pin is intentionally left unconnected on the circuit board, remove the wire and specify `.PORT()` with empty parentheses on the instance:
   ```verilog-fragment
   // NG: Unconnected pin assigned to an isolated wire
   // wire unused_wire;
   // IC u1 (.UNUSED(unused_wire));

   // OK: Deliberately unconnected port
   IC u1 (
       .UNUSED(),
       ...
   );
   ```

See also:
- [Circuit Description Syntax](../guides/syntax.md)
- [Rule: missing-port](missing-port.md)
