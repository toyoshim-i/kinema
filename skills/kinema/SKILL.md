---
name: kinema-pcb-design
description: Design, modify, and verify printed circuit boards (PCBs) in KiCad using kinema, a hardware description language based on a strict structural Verilog subset and offline equivalence verification toolchain. Use whenever designing circuits, authoring .v circuit files, importing netlists into KiCad, or verifying PCB layouts against kinema.toml.
---

# kinema PCB Design Guide

Design printed circuit boards by writing circuit descriptions in a strict structural Verilog subset (`.v`) and validating them against KiCad PCB layouts using the `kinema` toolchain. Schematics are completely bypassed: the circuit description is the ground truth, and correctness of pad-to-net assignments is verified by automated pad partition equivalence verification, with physical copper connectivity verified by KiCad DRC.

---

## Core Principles & Commitments

1. **Circuit Description is the Sole Ground Truth**: All electrical connectivity originates in `.v` files. Never manually reassign nets or pad numbers on the PCB layout.
2. **Definition of Done is `kinema check --strict` Passing**: Never claim a design is complete until `kinema check --strict` exits with code 0 (all lint, board pad-to-net equivalence, and DRC checks pass).
3. **Verified Leaf Modules**: Only use component leaf modules that have been agreed upon with human review or verified from manufacturer datasheets. Never guess pinouts.
4. **Human Review Gates**: Request human confirmation at defined checkpoints (Part Selection / Leaf Definition and Visual Schematic Review).
5. **Loop Limit**: If an automated fix loop fails 5 times consecutively on the same issue, stop and ask the user for guidance.

---

## Prerequisites & Project Layout

### Tools
- `kinema` CLI installed and available on PATH (`kinema --version`).
- KiCad 10 (with IPC API enabled under Preferences → Plugins, if using MCP automation).
- PCB layout MCP server (recommended: [Konnect](https://github.com/mixelpixx/Konnect) for KiCad 10).

### Repository Structure
```text
kinema.toml        # Project configuration (sources, libraries, board file)
src/*.v            # Circuit implementations
lib/*.v            # Component library (leaf module definitions)
board.kicad_pcb    # KiCad board layout
```

When beginning a task, first inspect `kinema.toml`, `git log`, existing `.v` files, and run `kinema check --stage lint` to establish baseline state.

---

## Operational Workflow

### Stage 1: Requirements & Part Selection (Human Interaction)
- Clarify design requirements: voltages, power sources, external interfaces, board dimensions, and physical constraints.
- Select components and prepare leaf modules in `lib/`. If a module does not exist, draft it with `kinema gen-leaf`:
  ```bash
  kinema gen-leaf <PART_NAME> --prefix <PREFIX> --footprint <FOOTPRINT>
  ```
- Verify and confirm the following with the human user before proceeding:
  - Pin names to physical pad numbers (especially packages with variable pinouts like SOT-23).
  - Electrical pin types (`etype`).
  - Power pins requiring decoupling marked with `decouple = "required"` (each placed on its own port).
  - Reference prefix, footprint, and MPN.

### Stage 2: Circuit Description
- Author modules in `src/` following the grammar and connection model.
- Format and run static verification:
  ```bash
  kinema fmt
  kinema check --stage lint --json
  ```
- Inspect JSON diagnostics:
  - `location`: Exact file and line to edit.
  - `subject`: Entity violating the rule (`hub`, `pin-wire`, `join`, `pin`, `net`, `component`).
  - `related`: Associated declarations or connections.
  - `fix`: Deterministic suggested fix (apply directly if available).

### Stage 3: Visual Review
- Generate Yosys-compatible JSON and render with `netlistsvg`:
  ```bash
  kinema graph src/top.v > target/graph.json
  netlistsvg target/graph.json -o target/schematic.svg
  ```
- Present the schematic diagram and bill of materials (BOM) to the human user for review before moving to layout.

### Stage 4: Netlist Synchronization
- Export KiCad S-expression netlist and design rules:
  ```bash
  kinema netlist -o board.net
  kinema rules
  ```
- In KiCad PCB Editor: Select **File → Import → Netlist...**, choose `board.net`, ensure match method is set to **Timestamp / UUID**, and click **Update PCB**.
- Save `board.kicad_pcb`.

### Stage 5: Placement & Routing (via MCP or Layout Editor)
- Review flat components, netlists, and proximity groupings:
  ```bash
  kinema ir --json
  ```
- **Placement order**:
  1. Fixed connectors, mounting holes, and edge components.
  2. Primary ICs.
  3. `nearby` groupings: Bypass capacitors placed immediately adjacent to the associated IC power/GND pin pairs.
  4. Remaining passive components and pullups.
- **Routing**:
  - Honor `width` attributes (enforced by DRC).
  - Size track widths according to `current` hints.
  - Save `board.kicad_pcb` when complete.

### Stage 6: Verification
Run the unified verification suite:
```bash
# During iterative design (checks syntax, formatting, and static lint):
kinema check --json

# For manufacturing sign-off (requires board file, verifies pad equivalence and DRC):
kinema check --strict --json
```

Routing and remediation decision tree:

| Failure Type | Example Codes | Action |
| :--- | :--- | :--- |
| **Circuit Syntax / Lint** | `missing-port`, `decouple-missing`, `undeclared-net` | Return to **Stage 2** (fix `.v`). |
| **Equivalence Discrepancy** | `component-missing`, `net-partition-mismatch` | Return to **Stage 4** (re-export and re-import netlist). |
| **Physical DRC Violation** | Unrouted net, track clearance, custom rule mismatch | Return to **Stage 5** (adjust tracks in PCB editor). |

*Never attempt to resolve an equivalence failure by manually rewiring pad net assignments on the PCB.*

### Stage 7: Final Deliverables
- Once `kinema check` passes with 0 errors:
  - Export manufacturing data via `kicad-cli`:
    ```bash
    kicad-cli pcb export gerbers -o build/gerber/ board.kicad_pcb
    kicad-cli pcb export drill -o build/gerber/ board.kicad_pcb
    kicad-cli pcb export pos --format csv -o build/cpl.csv board.kicad_pcb
    ```
  - Commit all `.v` sources, `kinema.toml`, and `board.kicad_pcb`.

---

## Language Highlights & Reference

The language is a strict structural subset of Verilog:
- Only `module`, `endmodule`, `inout`, `wire`, and `parameter`.
- All ports must be `inout`. Directions are defined by the `(* etype = "..." *)` attribute.
- Connections must be named explicitly (`.PORT(net)`). Positional connections and `.*` are disallowed.
- Deliberate unconnections must be explicit: `.PORT()`.
- Every net must be declared with `wire`.
- One declaration per line, formatted via `kinema fmt`.

### Essential Attributes

| Attribute | Target | Description | Example |
| :--- | :--- | :--- | :--- |
| `footprint` | Leaf, instance | KiCad footprint identifier | `(* footprint = "Resistor_SMD:R_0603_1608Metric" *)` |
| `mpn` | Leaf, instance | Manufacturer part number | `(* mpn = "NE555DR" *)` |
| `prefix` | Leaf module | Reference prefix | `(* prefix = "U" *)` |
| `pad` | Leaf port | Physical pad number(s) | `(* pad = "1" *)` |
| `etype` | Port | Electrical type | `(* etype = "power_in" *)` |
| `decouple` | Leaf port | Requires decoupling cap | `(* decouple = "required" *)` |
| `id` | Instance | Stable identity key | `(* id = "u1_core" *)` |
| `ref` | Instance | Fixed refdes override | `(* ref = "U1" *)` |
| `dnp` | Instance | Do Not Populate flag | `(* dnp *)` |
| `nearby` | Hub wire | Proximity hint | `(* nearby *) wire vcc_u1;` |
| `width` | Wire, port | Min track width constraint | `(* width = "0.5mm" *)` |
| `current` | Wire, port | Current capacity hint | `(* current = "1.5A" *)` |
| `netclass` | Wire, port | KiCad netclass name | `(* netclass = "Power" *)` |
| `diffpair` | Wire | Differential pair pair | `(* diffpair = "USB_D" *)` |

---

## Connection Model (`nearby` & `join`)

For sensitive nets (decoupling capacitors, crystal oscillators, high-speed differential pairs), use the `nearby` pattern:
1. Declare a hub wire with `(* nearby *)`.
2. Connect each participating component pin to a dedicated **pin-wire** (`<instance>_<port>`).
3. Connect each pin-wire to the hub using `join j_<pin-wire> (.P(hub), .C(pin-wire));`.
4. Connect the hub to the main power net using `join j_<hub> (.P(main_net), .C(hub));`.

```verilog
// Decoupling capacitor example: C1 placed next to U1 VCC pin
(* nearby *)
wire vcc_u1;
wire U1_VCC;
wire C1_A;

join j_vcc_u1 (.P(VCC), .C(vcc_u1));
join j_U1_VCC (.P(vcc_u1), .C(U1_VCC));
join j_C1_A (.P(vcc_u1), .C(C1_A));

NE555 U1 (
    .VCC(U1_VCC),
    ...
);

C #(.value("100n")) C1 (
    .A(C1_A),
    .B(GND)
);
```

---

## Diagnostics & Troubleshooting

| Diagnostic Code | Root Cause | Remediation |
| :--- | :--- | :--- |
| `undeclared-net` | Net identifier used without a `wire` declaration. | Declare `wire <name>;` (or fix typo). |
| `missing-port` | Port missing on instance. | Add `.PORT(net)` or explicit `.PORT()`. |
| `width-mismatch` | Port bit-width differs from connection bit-width. | Match bus widths or slice index `bus[0]`. |
| `pin-wire-name` | Pin-wire name does not follow `<instance>_<port>`. | Rename wire to `<instance>_<port>`. |
| `hub-direct-pad` | A pad connects directly to a `(* nearby *)` hub wire. | Insert dedicated pin-wire and `join`. |
| `decouple-missing` | Port with `decouple = "required"` not joined to hub. | Connect via pin-wire and join to decoupling hub. |
| `component-missing` | Component in `.v` not found on PCB. | Re-export `kinema netlist` and re-import into PCB editor. |
| `net-partition-mismatch` | PCB copper connectivity does not match circuit netlist. | Check for missing tracks, shorts, or un-synced netlist. |
| `identity-missing` | PCB footprint timestamp does not match UUID v5. | Ensure netlist importer matches by Timestamp / UUID. |
