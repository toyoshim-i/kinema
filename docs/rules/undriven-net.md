---
id: undriven-net
title: "Rule: undriven-net"
summary: Net has only input/passive pins and no driving source.
category: rule
---

# Rule: `undriven-net`

## Problem / Symptom
Static lint detected a multi-pin net that has no driving pins:
```text
Net '<net>' has no driving pins (only input pins)
```

## Root Cause
A declared wire connects two or more component pins, but all connected pins are marked as `input` or `passive`. The net has no source pin with a driving electrical type (`output`, `power_out`, `bidirectional`, `power_in`, `open_collector`, `open_emitter`, or `tri_state`), and the net is not connected to a module port.

## Remediation

1. Connect the net to an output, power, or driver pin:
   ```verilog-fragment
   // Source driver: output or power
   MCU u1 (.TX_OUT(tx_net), ...);
   // Destination receiver: input
   Transceiver u2 (.RX_IN(tx_net), ...);
   ```
2. If the net is driven from an external off-board signal, connect it to a port on the module, or ensure the input connector leaf definition has appropriate driving electrical types on its output pins.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
- [Rule: single-pin-net](single-pin-net.md)
