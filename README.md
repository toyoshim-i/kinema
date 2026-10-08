# kinema

> **Circuit Description Language & KiCad Equivalence Verification Tool**

[![CI](https://github.com/toyoshim-i/kinema/actions/workflows/ci.yml/badge.svg)](https://github.com/toyoshim-i/kinema/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online_manual-blue)](https://toyoshim-i.github.io/kinema/)
[![Release](https://img.shields.io/github/v/release/toyoshim-i/kinema)](https://github.com/toyoshim-i/kinema/releases)

`kinema` is an offline hardware description toolchain. It enables engineers and AI coding agents to design printed circuit boards (PCBs) by writing circuits in a **strict structural subset of Verilog**, bypassing graphical schematics entirely.

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

## Philosophy

- **Code as Single Ground Truth**: Electrical connectivity originates purely in code. Graphical schematics are generated views, not authoritative models.
- **Verification Over Presumption**: Automated pad-partition equivalence and physical DRC enforce zero discrepancy between intent and layout.
- **Context-Engineering-Free**: Tooling must be self-documenting. Rather than external context stuffing, guidance and diagnostic remediations are discovered on-demand just-in-time.

## Getting Started

- **[Online Manual](https://toyoshim-i.github.io/kinema/)**: Complete architectural guides, workflows, and rule references.
- **In-CLI Guides**: Run `kinema guide` to browse the interactive documentation index, or `kinema explain <CODE>` for contextual diagnostic remediation.
- **CLI Reference**: Run `kinema --help` for available subcommands and flags.

## Installation

Pre-built binaries are available on [GitHub Releases](https://github.com/toyoshim-i/kinema/releases).

To build from source:
```bash
cargo install --path crates/kinema-cli
```

## License

MIT OR Apache-2.0
