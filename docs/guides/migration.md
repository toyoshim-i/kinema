---
id: migration
title: KiCad Project Migration & Import Guide
aliases: [import, porting, convert, legacy, kicad]
summary: Step 0: How to safely migrate existing KiCad schematics and PCBs to Kinema without breaking PCB layout or UUIDs.
category: getting-started
---

# KiCad Project Migration & Import Guide

When introducing Kinema to an existing project, migration is the critical **Step 0**. 

Because writing structural Verilog feels familiar, developers and AI agents often attempt an ad-hoc transcription of the KiCad schematic directly into `.v`. Without understanding how KiCad tracks footprint identities, this frequently corrupts existing PCB layouts and erases routed copper tracks.

This guide details the exact playbook to migrate any existing KiCad project safely into Kinema while preserving 100% of your PCB layout and routing.

---

## The Trap: Why Ad-Hoc Migration Fails

In KiCad:
1. Every schematic symbol and PCB footprint has a unique **Timestamp / UUID** (e.g. `uuid "7c25c65d-..."` in `*.kicad_sch` and `tstamp "7c25c65d-..."` in `board.kicad_pcb`).
2. Routed copper traces, vias, and footprint coordinates on the PCB layout are bound to these UUIDs.
3. By default, Kinema assigns each module instance a deterministic UUID v5 derived from its Verilog instance hierarchy path (e.g. `top.U1`).

If you author Verilog without pinning original UUIDs:
- Kinema generates new, different UUIDs.
- When you import `board.net` into KiCad using the standard **Match by: Timestamp / UUID**, KiCad assumes every single existing footprint was deleted and instantiates new, unplaced footprints outside your board outline. All existing copper routing is detached.
- If you instead import matching by **Reference Designator**, any slight renumbering, pin swap, or footprint name discrepancy creates silent netlist mismatches or disconnected pads.

---

## The 6-Step Migration Playbook

Follow these steps in order before modifying the circuit.

### Step 1: Inspect the Existing KiCad Project

Inspect your target `board.kicad_pcb` (and `*.kicad_sch` if available):
1. **Identify Footprint Libraries**:
   Check what footprint libraries are used. For example:
   - `Package_SO:SOIC-8_3.9x4.9mm_P1.27mm`
   - `Resistor_SMD:R_0603_1608Metric`
   - `Connector_PinHeader_2.54mm:PinHeader_1x04_P2.54mm_Vertical`
2. **Inspect Pad Numbering**:
   Confirm whether pin numbers are numeric (`1, 2, 3...`) or alphanumeric (`A1, B1...`). KiCad footprints bind electrical connections to pad numbers.

### Step 2: Define Leaf Modules (`lib/*.v`)

For every unique IC, connector, and passive type in your board, ensure a leaf module definition exists in `lib/`.

Use `kinema gen-leaf` to generate standard leaf module templates:
```bash
kinema gen-leaf NE555 --prefix U --footprint Package_SO:SOIC-8_3.9x4.9mm_P1.27mm
```

Ensure pad numbers in the leaf module match the physical KiCad footprint pads:
```verilog
(* footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm", prefix = "U" *)
module NE555 (
    (* pin = "1", etype = "power_in" *) input wire GND,
    (* pin = "2", etype = "input" *)    input wire TRIG,
    (* pin = "3", etype = "output" *)   output wire OUT,
    (* pin = "4", etype = "input" *)    input wire RESET,
    (* pin = "5", etype = "passive" *)  input wire CTRL,
    (* pin = "6", etype = "input" *)    input wire THRES,
    (* pin = "7", etype = "output" *)   output wire DISCH,
    (* pin = "8", etype = "power_in" *) input wire VCC
);
endmodule
```

### Step 3: Extract & Pin Existing UUIDs

To protect your existing PCB placement and copper tracks, extract the existing UUIDs from `board.kicad_pcb` or `*.kicad_sch`:

```bash
# Search for footprint timestamp in board.kicad_pcb
grep -B 2 -A 5 '"Reference" "U1"' board.kicad_pcb
```

You will find a block like:
```lisp
(footprint "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm"
  (layer "F.Cu")
  (tstamp "7c25c65d-64b6-56cb-a17c-ffb79789f143")
  ...
  (property "Reference" "U1" ...)
)
```

In your Verilog description (`src/top.v`), pin this UUID using the `(* id = "..." *)` attribute:
```verilog
// Pinned UUID guarantees KiCad matches this instance to the existing PCB footprint
(* id = "7c25c65d-64b6-56cb-a17c-ffb79789f143" *)
NE555 U1 (
    .GND(GND),
    .TRIG(net_trig),
    .OUT(clk_out),
    .RESET(VCC),
    .CTRL(net_ctrl),
    .THRES(net_trig),
    .DISCH(net_trig),
    .VCC(VCC)
);
```

> [!TIP]
> Pinning the UUID gives you complete freedom: you can later refactor, rename, or renumber instances in Verilog without disturbing the PCB footprint placement or routed traces. See [Component Identity & Refactoring](identity.md).

### Step 4: Transcribe Schematic to Structural Verilog

Write the circuit connections in `src/top.v`:
- Use meaningful wire names for internal interconnects.
- For decoupling capacitors, place them directly adjacent to the IC power pins and declare `(* decouple *)`:
  ```verilog
  (* decouple, id = "b1284a1e-1284-4820-9118-84201948291a" *)
  C_0603 C1 (.P1(VCC), .P2(GND));
  ```
- Format and run static lint checks:
  ```bash
  kinema fmt
  kinema check --stage lint
  ```

### Step 5: Visual Schematic Review

Before synchronizing with KiCad PCB Editor, verify the structural topology visually:
```bash
kinema graph src/top.v > target/graph.json
netlistsvg target/graph.json -o target/schematic.svg
```

Open `target/schematic.svg` and compare against your original KiCad schematic. Confirm that power nets, IC pins, and pull-up/pull-down resistors match the design intent.

### Step 6: Synchronize with KiCad PCB

1. Export the new netlist from Kinema:
   ```bash
   kinema netlist -o board.net
   ```
2. Open `board.kicad_pcb` in KiCad PCB Editor.
3. Select **File → Import → Netlist...**.
4. Choose `board.net`.
5. Under **Match Method**, select **Timestamp / UUID**.
6. Review the changes list in the preview:
   - **Crucial**: Ensure there are **0 Footprints to delete**.
   - If footprints show up for deletion, stop! A UUID was missed or mistyped in `(* id = "..." *)`.
7. Click **Update PCB** and save `board.kicad_pcb`.

---

## Step 7: Unified Verification & Sign-Off

Run Kinema's unified verification engine:
```bash
kinema check --strict
```

This performs three checks in one command:
1. **Verilog Lint**: Confirms strict syntax, typed port directions, and design rules.
2. **Equivalence Check**: Compares every pad connection in `board.kicad_pcb` against the Verilog netlist to ensure zero discrepancies (`net-partition-mismatch`).
3. **Physical DRC**: Runs KiCad's Design Rule Check (clearances, unrouted nets, track widths).

When `kinema check --strict` exits with code 0, the migration is complete and fully verified.

---

## Common Migration Issues & Remedies

| Symptom | Root Cause | Remediation |
| :--- | :--- | :--- |
| KiCad netlist import wants to delete all footprints | UUIDs were not pinned in Verilog | Inspect `(tstamp ...)` in `board.kicad_pcb` and add `(* id = "<uuid>" *)` to each instance in `.v`. |
| Net connections are inverted / swapped on an IC | Leaf module port `(* pin = "..." *)` does not match footprint pad numbers | Open the footprint in KiCad Footprint Editor, verify pad numbers, and update the leaf module in `lib/*.v`. |
| `kinema check` reports `net-partition-mismatch` | `board.net` was not imported into KiCad or not saved | Re-export with `kinema netlist -o board.net`, import in KiCad, and save `board.kicad_pcb`. |
| Decoupling capacitor warning | Power pin lacks a nearby bypass capacitor | Connect a capacitor across `VCC` and `GND` with `(* decouple *)`. See [Connection Model](connection.md). |

See also:
- [Core Design Principles](principles.md)
- [Design Workflow](workflow.md)
- [Component Identity & Refactoring](identity.md)
- [Language Syntax & Attributes](syntax.md)
