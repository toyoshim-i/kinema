---
name: kinema-pcb-design
description: Design, modify, route, and verify printed circuit boards (PCBs) in KiCad using kinema, or work on KiCad PCB layouts using the kinema toolchain.
---

# kinema PCB Design Guide

Design printed circuit boards by writing circuit descriptions in a strict structural Verilog subset (`.v`) and validating them against KiCad PCB layouts using the `kinema` toolchain.

## Getting Started

Kinema is self-documenting. Before starting any design or modification task, run:

```bash
kinema guide agent
```

This retrieves your full operational commitments, guardrails, human review gates, and 7-stage workflow directly from the installed tool.

## Key Subcommands

- `kinema guide`: Browse available architectural guides (`syntax`, `connection`, `identity`, `workflow`).
- `kinema explain <CODE>`: View root causes and concrete code remediation for any diagnostic code.
- `kinema check`: Run iterative syntax and static lint checks.
- `kinema check --strict`: Run full manufacturing sign-off (equivalence and DRC).
