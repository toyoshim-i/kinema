---
id: identity
title: Component Identity & Refactoring
aliases: [uuid, refactor, renumber, tstamp]
summary: How to preserve PCB footprint placement and tracks when renaming or renumbering components.
category: guide
---

# Component Identity & Refactoring

KiCad tracks PCB footprint instances on the board layout by **Timestamp / UUID**, not by their reference designator (`refdes`).
When authoring circuits in Kinema, each instance is assigned a deterministic UUID v5 derived from the instance path (e.g. `top.J1`).

If you renumber components in `.v` (for example, sorting connector terminals `J1, J2, ...` to match physical layout order) without fixing their identity, Kinema generates a different UUID. Upon importing the netlist into KiCad with Timestamp matching, KiCad interprets this as a deleted footprint and a newly placed footprint, which can disconnect routed copper tracks and reset footprint placement.

See [Rule: identity-missing](../rules/identity-missing.md) for related diagnostic checks.

## Recommended Workflow

### 1. Inspect Original KiCad UUID
When importing from KiCad or preparing to refactor existing footprints:
- In `board.kicad_pcb`: search for the reference designator to find its timestamp:
  ```bash
  grep -B 2 -A 5 '"Reference" "J1"' board.kicad_pcb
  # Look for (tstamp "<uuid>")
  ```
- In schematic files (`*.kicad_sch`): search for the symbol UUID:
  ```bash
  grep -B 2 -A 5 '"Reference" "J1"' *.kicad_sch
  # Look for (uuid "<uuid>")
  ```

### 2. Pin the Identity in Verilog
Explicitly assign the original UUID (or a stable hash string) to the `id` attribute:
```verilog
// Frozen identity guarantees invariant UUID v5 even if instance name is renamed
(* id = "7c25c65d-64b6-56cb-a17c-ffb79789f143" *)
CONN_1X04 J2 (.P1(VCC), .P2(SDA), .P3(SCL), .P4(GND)); // Renamed/renumbered from J1
```

### 3. Synchronize with KiCad
1. Export the updated netlist: `kinema netlist -o board.net`
2. In KiCad PCB Editor: **File → Import → Netlist...**
3. Set **Match Method** to **Timestamp / UUID**
4. Click **Update PCB**

KiCad updates the reference designator in-place on the PCB without moving the footprint or disturbing any routed copper traces.

See also:
- [Connection Model & Decoupling](../guides/connection.md)
- [Rule: identity-missing](../rules/identity-missing.md)
