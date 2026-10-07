# kinema Documentation

> **Circuit Description Language & KiCad Equivalence Verification Tool**

`kinema` is an offline hardware description toolchain written in Rust. It enables engineers and AI coding agents to design printed circuit boards (PCBs) by writing circuits in a **strict structural subset of Verilog**, bypassing graphical schematics entirely.

```
       Circuit Description (.v)
                 │
                 ▼
          kinema fmt & lint
                 │
                 ▼
       KiCad Netlist (.net) ──► KiCad PCB Editor (Sync)
                                       │
                                       ▼
                               Placement & Routing
                                       │
                                       ▼
  .kicad_pcb ◄──────────────► kinema check (--strict)
                               (Equiv & KiCad DRC)
```

---

## Getting Started

If you are migrating an existing KiCad project, start with the migration playbook:
- **[KiCad Project Migration](guides/migration.md)**: How to safely import existing schematics and boards without breaking PCB footprint placement or routed copper tracks.

## Core Architectural Guides

- **[Core Design Principles](guides/principles.md)**: Ground truth commitments, verification sign-off, and review checkpoints.
- **[End-to-End PCB Workflow](guides/workflow.md)**: The 7-stage workflow from requirements to manufacturing sign-off.
- **[Language Syntax & Attributes](guides/syntax.md)**: Strict structural Verilog grammar and attribute specifications.
- **[Component Identity & Refactoring](guides/identity.md)**: Preserving PCB footprints and tracks using UUID v5 when refactoring or renumbering.
- **[Connection Model & Decoupling](guides/connection.md)**: Sensitive routing topologies, bypass capacitors, and differential pairs.

## Diagnostics & Rule References

- **[identity-missing](rules/identity-missing.md)**: PCB footprint timestamp does not match circuit UUID v5.
- **[decouple-missing](rules/decouple-missing.md)**: Required bypass capacitor is missing from an IC power pin.
