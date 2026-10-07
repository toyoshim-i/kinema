---
id: workflow
title: End-to-End PCB Design & Verification Workflow
aliases: [stages, process, steps]
summary: The 7-stage operational workflow from requirements to manufacturing sign-off.
category: guide
---

# End-to-End PCB Design & Verification Workflow

Kinema completely bypasses schematic capture: your Verilog description in `src/` is the single source of truth.

## Stage 1: Requirements & Part Selection
- Confirm voltages, interfaces, dimensions, and physical constraints with the human user.
- Prepare leaf modules in `lib/`. If a module is missing, draft it with:
  ```bash
  kinema gen-leaf <PART_NAME> --prefix <PREFIX> --footprint <FOOTPRINT>
  ```
- **Human Gate**: Confirm pin names, pad numbers, `etype`, `decouple` needs, and MPN before proceeding.

## Stage 2: Circuit Description
- Write modules in `src/`.
- Format and run static lint:
  ```bash
  kinema fmt
  kinema check --stage lint
  ```

## Stage 3: Visual Review
- Generate netlist graph and render with `netlistsvg`:
  ```bash
  kinema graph src/top.v > target/graph.json
  netlistsvg target/graph.json -o target/schematic.svg
  ```
- **Human Gate**: Present the schematic SVG and bill of materials (BOM) to the human user for visual confirmation.

## Stage 4: Netlist Synchronization
- Export KiCad S-expression netlist and design rules:
  ```bash
  kinema netlist -o board.net
  kinema rules
  ```
- In KiCad PCB Editor: Select **File → Import → Netlist...**, choose `board.net`, select **Match by: Timestamp / UUID**, and click **Update PCB**.
- Save `board.kicad_pcb`.

## Stage 5: Placement & Routing
- Inspect components, net connectivity, and proximity groupings:
  ```bash
  kinema ir --json
  ```
- Place fixed connectors first, ICs second, bypass capacitors next to power pins third, and passives last.
- Route tracks respecting `width` constraints.

## Stage 6: Unified Verification
Run verification across lint, pad-to-net equivalence, and physical DRC:
```bash
kinema check --strict
```
- If syntax/lint fails: fix `.v`.
- If equivalence fails (`net-partition-mismatch`): re-export netlist and re-import into KiCad.
- If physical DRC fails: adjust copper routing in KiCad PCB Editor.

## Stage 7: Manufacturing Deliverables
Once `kinema check --strict` passes with 0 errors:
```bash
kicad-cli pcb export gerbers -o build/gerber/ board.kicad_pcb
kicad-cli pcb export drill -o build/gerber/ board.kicad_pcb
kicad-cli pcb export pos --format csv -o build/cpl.csv board.kicad_pcb
```

See also:
- [AI Agent Guidelines](../guides/agent.md)
- [Language Syntax & Attributes](../guides/syntax.md)
- [Component Identity & Refactoring](../guides/identity.md)
