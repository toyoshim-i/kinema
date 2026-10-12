---
id: missing-port
title: "Rule: missing-port"
summary: Port is missing from instance connection list.
category: rule
---

# Rule: `missing-port`

## Problem / Symptom
Static lint detected an instantiated module where one of its declared ports was omitted from the connection list:
```text
Port '<port>' connection missing (use .<port>() for deliberate unconnected)
```

## Root Cause
Kinema requires all module ports to be explicitly accounted for at every instantiation point. Omitting a port connection is not permitted, even if the pin is left physically unconnected on the board.

## Remediation

1. If the port connects to a net or pin-wire, add the named port connection:
   ```verilog-fragment
   MyModule u1 (
       .CLK(clk_net),
       .DATA(data_net)
   );
   ```
2. If the port is intentionally left unconnected, write the port connection with empty parentheses `.PORT()`:
   ```verilog-fragment
   MyModule u1 (
       .CLK(clk_net),
       .DATA(data_net),
       .NC_PIN() // Deliberately unconnected
   );
   ```

Note: Do not connect unused pins to dummy wires (which triggers `single-pin-net` or `undriven-net`), and do not use non-existent attributes like `(* nc *)`. Use `.PORT()`.

See also:
- [Circuit Description Syntax](../guides/syntax.md)
- [Rule: nc-connected](nc-connected.md)
